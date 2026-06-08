use crate::app::App;
use crate::enumerate::AppFocus;
use crate::models::Apps;
use crate::service::{
    filter_apps_by_platform, get_install_command, install_command, is_root, read_apps_json,
    requires_interactive, requires_sudo,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use std::collections::HashSet;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

pub fn app_render(frame: &mut Frame, sidebar: Rect, body: Rect, app: &mut App) {
    if app.apps.is_none() {
        log::info!(
            "[app_render] apps cache is empty — loading apps.json for manager={}",
            app.active_package_manager()
        );
        let raw = read_apps_json().unwrap_or_else(|e| {
            log::error!("[app_render] Failed to load apps.json: {}", e);
            vec![]
        });
        app.apps = Some(filter_apps_by_platform(&raw, app.active_package_manager()));
        log::debug!(
            "[app_render] loaded {} sections after platform filter",
            app.apps.as_ref().map(|a| a.len()).unwrap_or(0)
        );
    }

    let apps = app.apps.as_ref().unwrap();

    let body_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(3)])
        .split(body);

    // Drain install log channel
    if let Some(rx) = &app.install_rx {
        let before = app.app_install_log.len();
        while let Ok(line) = rx.try_recv() {
            app.app_install_log.push(line);
        }
        let added = app.app_install_log.len() - before;
        if added > 0 {
            log::debug!("[app_render] drained {} new install log lines", added);
        }
    }

    render_sidebar(frame, sidebar, apps, app);
    render_app_list(frame, body_chunks[0], apps, app);
    render_custom_input(frame, body_chunks[1], app);

    if app.app_sudo_pending {
        log::debug!("[app_render] rendering sudo confirmation modal");
        render_sudo_confirmation(frame, app);
    } else if app.app_installing {
        log::debug!("[app_render] rendering install modal (log_lines={})", app.app_install_log.len());
        render_install_modal(frame, app);
    }
}

fn render_sidebar(frame: &mut Frame, area: Rect, apps: &Apps, app: &App) {
    let focused = app.app_focus == AppFocus::Section;

    let items: Vec<ListItem> = apps
        .iter()
        .enumerate()
        .map(|(i, section)| {
            let selected = i == app.app_selected_section;
            let style = match (selected, focused) {
                (true, true) => Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
                (true, false) => Style::default().fg(Color::Yellow),
                _ => Style::default().fg(Color::DarkGray),
            };
            let prefix = if selected && focused { "▶ " } else { "  " };
            ListItem::new(Line::from(Span::styled(
                format!("{}{}", prefix, section.section),
                style,
            )))
        })
        .collect();

    let border_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = format!("Sections [{}]", app.active_package_manager());

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style),
    );

    frame.render_widget(list, area);
}

fn render_app_list(frame: &mut Frame, area: Rect, apps: &Apps, app: &App) {
    let focused = app.app_focus == AppFocus::Apps;

    let Some(section) = apps.get(app.app_selected_section) else {
        log::warn!(
            "[render_app_list] no section at index {} — rendering empty state",
            app.app_selected_section
        );
        frame.render_widget(
            Paragraph::new("No section selected").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Applications ({})", app.active_package_manager())),
            ),
            area,
        );
        return;
    };

    let items: Vec<ListItem> = section
        .apps
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_cursor = focused && i == app.app_selected_app;
            let is_selected = app.app_selected_ids.contains(&entry.id);

            let checkbox = if is_selected { "[✓]" } else { "[ ]" };
            let arrow = if is_cursor { "▶ " } else { "  " };

            let style = match (is_cursor, is_selected) {
                (true, _) => Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
                (_, true) => Style::default().fg(Color::Green),
                _ => Style::default().fg(Color::White),
            };

            ListItem::new(Line::from(Span::styled(
                format!("{}{} {}", arrow, checkbox, entry.name),
                style,
            )))
        })
        .collect();

    let selected_count = app.app_selected_ids.len();
    let title = format!(
        "Apps — {} ({} selected) [{}]",
        section.section,
        selected_count,
        app.active_package_manager(),
    );

    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style),
    );

    frame.render_widget(list, area);
}

fn render_custom_input(frame: &mut Frame, area: Rect, app: &App) {
    let focused = app.app_focus == AppFocus::CustomInput;

    let border_style = if focused {
        Style::default().fg(Color::Magenta)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let content = if focused {
        if app.app_custom_input.is_empty() {
            Span::styled("▌", Style::default().fg(Color::Magenta))
        } else {
            Span::styled(
                format!("{}▌", app.app_custom_input),
                Style::default().fg(Color::White),
            )
        }
    } else {
        Span::styled(
            "Press [i] to enter custom app id".to_string(),
            Style::default().fg(Color::DarkGray),
        )
    };

    let input = Paragraph::new(Line::from(content)).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Custom App [i]")
            .border_style(border_style),
    );

    frame.render_widget(input, area);
}

fn render_install_modal(frame: &mut Frame, app: &App) {
    let area = centered_rect(70, 70, frame.area());
    frame.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
        .split(area);

    let log_text: Vec<Line> = app
        .app_install_log
        .iter()
        .map(|l| {
            let style = if l.starts_with("✓") {
                Style::default().fg(Color::Green)
            } else if l.starts_with("✗") || l.starts_with("[err]") {
                Style::default().fg(Color::Red)
            } else if l.starts_with("▶") {
                Style::default().fg(Color::Cyan)
            } else if l.starts_with("═") {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            Line::from(Span::styled(l.clone(), style))
        })
        .collect();

    let scroll_offset = (log_text.len() as u16).saturating_sub(chunks[0].height.saturating_sub(2));

    let log = Paragraph::new(log_text)
        .scroll((scroll_offset, 0))
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Installing... ")
                .border_style(Style::default().fg(Color::Yellow)),
        );

    frame.render_widget(log, chunks[0]);

    let input_display = if app.install_input.is_empty() {
        Span::styled("▌", Style::default().fg(Color::DarkGray))
    } else {
        Span::styled(
            format!("{}▌", app.install_input),
            Style::default().fg(Color::White),
        )
    };

    let input_box = Paragraph::new(Line::from(input_display)).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Response (y/n/Enter) ")
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(input_box, chunks[1]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

// ─── Key handler ─────────────────────────────────────────────────────────────

pub fn handle_key(app: &mut App, key: KeyEvent) {
    log::debug!(
        "[handle_key] focus={:?} key={:?}",
        app.app_focus,
        key.code
    );
    match app.app_focus {
        AppFocus::Section     => handle_section_keys(app, key),
        AppFocus::Apps        => handle_apps_keys(app, key),
        AppFocus::CustomInput => handle_custom_input_keys(app, key),
        AppFocus::Installing  => handle_installing_keys(app, key),
        AppFocus::SudoConfirm => handle_sudo_confirm_keys(app, key),
    }
}

fn handle_sudo_confirm_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char(c) => {
            app.app_sudo_password.push(c);
            log::debug!(
                "[sudo_confirm] password input length={}",
                app.app_sudo_password.len()
            );
        }
        KeyCode::Backspace => {
            app.app_sudo_password.pop();
            log::debug!(
                "[sudo_confirm] backspace — password length={}",
                app.app_sudo_password.len()
            );
        }
        KeyCode::Enter => {
            if app.app_sudo_password.is_empty() {
                log::warn!("[sudo_confirm] Enter pressed but password is empty — ignoring");
                return;
            }
            log::info!(
                "[sudo_confirm] password confirmed — launching interactive install with sudo, manager={}",
                app.active_package_manager()
            );
            let password = app.app_sudo_password.clone();
            app.app_sudo_password.clear();
            app.app_sudo_pending = false;
            execute_install_interactive(app, Some(password));
        }
        KeyCode::Esc => {
            log::info!("[sudo_confirm] cancelled by user — clearing sudo state");
            app.app_sudo_pending = false;
            app.app_sudo_password.clear();
            app.app_sudo_command.clear();
            app.app_focus = AppFocus::Apps;
        }
        _ => {}
    }
}

// ─── Helper: strip \r from lines emitted by Windows processes ────────────────
fn sanitize_line(line: String) -> String {
    line.trim_end_matches('\r').to_string()
}

// ─── Helper: split a command string into (binary, args) ─────────────────────
fn split_command(cmd: &str) -> (String, Vec<String>) {
    let mut tokens = cmd.split_whitespace();
    let binary = tokens.next().unwrap_or("").to_string();
    let args = tokens.map(|s| s.to_string()).collect();
    (binary, args)
}

// Kept for future Linux update work — do NOT remove.
fn execute_install_with_password(app: &mut App, password: String) {
    let commands = app
        .app_sudo_command
        .drain(..)
        .map(|cmd| format!("sudo -S {}", cmd))
        .collect::<Vec<_>>();

    log::info!(
        "[execute_install_with_password] starting — manager={} command_count={}",
        app.active_package_manager(),
        commands.len()
    );
    for cmd in &commands {
        log::debug!("[execute_install_with_password] queued: {}", cmd);
    }

    app.app_install_log.clear();
    app.app_install_log
        .push("Starting installation...".to_string());
    app.app_install_log
        .push("Type y/n or Enter to respond to prompts.".to_string());
    app.app_installing = true;

    let (out_tx, out_rx) = mpsc::channel::<String>();
    let (in_tx, in_rx) = mpsc::channel::<String>();

    app.install_rx = Some(out_rx);
    app.install_tx = Some(in_tx);

    let password_clone: Option<String> = Some(password.clone());

    thread::spawn(move || {
        let in_rx = Arc::new(Mutex::new(in_rx));

        for cmd in commands {
            let _ = out_tx.send(format!("▶ Running: {}", cmd));
            log::info!("[install_with_password:thread] running: {}", cmd);

            let (binary, args) = split_command(&cmd);
            if binary.is_empty() {
                log::warn!("[install_with_password:thread] empty binary — skipping");
                continue;
            }

            match Command::new(&binary)
                .args(&args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(mut child) => {
                    log::debug!(
                        "[install_with_password:thread] spawned {} pid={:?}",
                        binary,
                        child.id()
                    );
                    if let Some(mut stdin) = child.stdin.take() {
                        use std::io::Write;

                        if let Some(ref pwd) = password_clone {
                            log::debug!("[install_with_password:thread] writing password to stdin");
                            let _ = writeln!(stdin, "{}", pwd);
                        }

                        let in_rx_clone = Arc::clone(&in_rx);
                        thread::spawn(move || {
                            log::debug!("[install_with_password:stdin_relay] waiting for user input");
                            while let Ok(input) = in_rx_clone.lock().unwrap().recv() {
                                log::debug!(
                                    "[install_with_password:stdin_relay] forwarding input: {:?}",
                                    input
                                );
                                let _ = writeln!(stdin, "{}", input);
                            }
                            log::debug!("[install_with_password:stdin_relay] channel closed — exiting");
                        });
                    }

                    if let Some(stdout) = child.stdout.take() {
                        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                            let line = sanitize_line(line);
                            log::debug!("[install_with_password:stdout] {}", line);
                            if out_tx.send(line).is_err() {
                                log::warn!("[install_with_password:stdout] channel closed — stopping");
                                return;
                            }
                        }
                    }

                    if let Some(stderr) = child.stderr.take() {
                        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                            let line = sanitize_line(line);
                            if !line.contains("[sudo]") && !line.contains("password for") {
                                log::debug!("[install_with_password:stderr] {}", line);
                                if out_tx.send(format!("[err] {}", line)).is_err() {
                                    log::warn!("[install_with_password:stderr] channel closed — stopping");
                                    return;
                                }
                            }
                        }
                    }

                    match child.wait() {
                        Ok(s) if s.success() => {
                            log::info!("[install_with_password:thread] {} exited OK", binary);
                            let _ = out_tx.send(format!("✓ Done: {}", binary));
                        }
                        Ok(s) => {
                            log::warn!(
                                "[install_with_password:thread] {} exited with status {}",
                                binary, s
                            );
                            let _ = out_tx.send(format!("✗ Failed (exit {})", s));
                        }
                        Err(e) => {
                            log::error!("[install_with_password:thread] wait error for {}: {}", binary, e);
                            let _ = out_tx.send(format!("✗ Error: {}", e));
                        }
                    }
                }
                Err(e) => {
                    log::error!("[install_with_password:thread] could not spawn {}: {}", binary, e);
                    let _ = out_tx.send(format!("✗ Could not run {}: {}", binary, e));
                }
            }
        }
        log::info!("[install_with_password:thread] all commands finished");
        let _ = out_tx.send("═══ All done ═══".to_string());
    });
}

fn execute_install_interactive(app: &mut App, password: Option<String>) {
    let commands = if password.is_some() {
        app.app_sudo_command
            .drain(..)
            .map(|cmd| format!("sudo -S {}", cmd))
            .collect::<Vec<_>>()
    } else {
        build_commands(app)
    };

    log::info!(
        "[execute_install_interactive] starting — manager={} sudo={} command_count={}",
        app.active_package_manager(),
        password.is_some(),
        commands.len()
    );
    for cmd in &commands {
        log::debug!("[execute_install_interactive] queued: {}", cmd);
    }

    app.app_install_log.clear();
    app.app_install_log
        .push("Starting installation...".to_string());
    if requires_interactive(app.active_package_manager()) {
        app.app_install_log
            .push("Type y/n and press Enter to respond to prompts.".to_string());
    }
    app.app_installing = true;
    app.app_focus = AppFocus::Installing;

    let (out_tx, out_rx) = mpsc::channel::<String>();
    let (in_tx, in_rx) = mpsc::channel::<String>();

    app.install_rx = Some(out_rx);
    app.install_tx = Some(in_tx);

    let password_clone: Option<String> = password.clone();

    thread::spawn(move || {
        let in_rx = Arc::new(Mutex::new(in_rx));

        for cmd in commands {
            let _ = out_tx.send(format!("▶ Running: {}", cmd));
            log::info!("[install_interactive:thread] running: {}", cmd);

            let (binary, args) = split_command(&cmd);
            if binary.is_empty() {
                log::warn!("[install_interactive:thread] empty binary — skipping");
                continue;
            }

            match Command::new(&binary)
                .args(&args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(mut child) => {
                    log::debug!(
                        "[install_interactive:thread] spawned {} pid={:?}",
                        binary,
                        child.id()
                    );
                    if let Some(mut stdin) = child.stdin.take() {
                        use std::io::Write;

                        if let Some(ref pwd) = password_clone {
                            log::debug!("[install_interactive:thread] writing password to stdin");
                            let _ = writeln!(stdin, "{}", pwd);
                        }

                        let in_rx_clone = Arc::clone(&in_rx);
                        thread::spawn(move || {
                            log::debug!("[install_interactive:stdin_relay] waiting for user input");
                            while let Ok(input) = in_rx_clone.lock().unwrap().recv() {
                                log::debug!(
                                    "[install_interactive:stdin_relay] forwarding: {:?}",
                                    input
                                );
                                let _ = writeln!(stdin, "{}", input);
                            }
                            log::debug!("[install_interactive:stdin_relay] channel closed — exiting");
                        });
                    }

                    if let Some(stdout) = child.stdout.take() {
                        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                            let line = sanitize_line(line);
                            log::debug!("[install_interactive:stdout] {}", line);
                            if out_tx.send(line).is_err() {
                                log::warn!("[install_interactive:stdout] channel closed — stopping");
                                return;
                            }
                        }
                    }

                    if let Some(stderr) = child.stderr.take() {
                        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                            let line = sanitize_line(line);
                            if !line.contains("[sudo]") && !line.contains("password for") {
                                log::debug!("[install_interactive:stderr] {}", line);
                                if out_tx.send(format!("[err] {}", line)).is_err() {
                                    log::warn!("[install_interactive:stderr] channel closed — stopping");
                                    return;
                                }
                            }
                        }
                    }

                    match child.wait() {
                        Ok(s) if s.success() => {
                            log::info!("[install_interactive:thread] {} exited OK", binary);
                            let _ = out_tx.send(format!("✓ Done: {}", binary));
                        }
                        Ok(s) => {
                            log::warn!(
                                "[install_interactive:thread] {} exited with status {}",
                                binary, s
                            );
                            let _ = out_tx.send(format!("✗ Failed (exit {})", s));
                        }
                        Err(e) => {
                            log::error!("[install_interactive:thread] wait error for {}: {}", binary, e);
                            let _ = out_tx.send(format!("✗ Error: {}", e));
                        }
                    }
                }
                Err(e) => {
                    log::error!("[install_interactive:thread] could not spawn {}: {}", binary, e);
                    let _ = out_tx.send(format!("✗ Could not run {}: {}", binary, e));
                }
            }
        }
        log::info!("[install_interactive:thread] all commands finished");
        let _ = out_tx.send("═══ All done ═══".to_string());
    });
}

fn handle_section_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.app_selected_section > 0 {
                app.app_selected_section -= 1;
                app.app_selected_app = 0;
                log::debug!(
                    "[section] navigate up — section={} app={}",
                    app.app_selected_section, app.app_selected_app
                );
            } else {
                log::debug!("[section] navigate up — already at top");
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(apps) = &app.apps
                && app.app_selected_section < apps.len().saturating_sub(1)
            {
                app.app_selected_section += 1;
                app.app_selected_app = 0;
                log::debug!(
                    "[section] navigate down — section={} app={}",
                    app.app_selected_section, app.app_selected_app
                );
            } else {
                log::debug!("[section] navigate down — already at bottom");
            }
        }
        KeyCode::Char(' ') | KeyCode::Enter => {
            log::info!(
                "[section] enter apps list — section={}",
                app.app_selected_section
            );
            app.app_focus = AppFocus::Apps;
            app.app_selected_app = 0;
        }
        KeyCode::Char('d') => {
            log::info!(
                "[section] install triggered — selected_count={}",
                app.app_selected_ids.len()
            );
            start_install(app);
        }
        _ => {}
    }
}

fn handle_apps_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.app_selected_app > 0 {
                app.app_selected_app -= 1;
                log::debug!(
                    "[apps] navigate up — section={} app={}",
                    app.app_selected_section, app.app_selected_app
                );
            } else {
                log::debug!("[apps] navigate up — already at top");
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(apps) = &app.apps
                && let Some(section) = apps.get(app.app_selected_section)
                && app.app_selected_app < section.apps.len().saturating_sub(1)
            {
                app.app_selected_app += 1;
                log::debug!(
                    "[apps] navigate down — section={} app={}",
                    app.app_selected_section, app.app_selected_app
                );
            } else {
                log::debug!("[apps] navigate down — already at bottom");
            }
        }
        KeyCode::Char(' ') => {
            if let Some(apps) = &app.apps
                && let Some(section) = apps.get(app.app_selected_section)
                && let Some(entry) = section.apps.get(app.app_selected_app)
            {
                if app.app_selected_ids.contains(&entry.id) {
                    app.app_selected_ids.remove(&entry.id);
                    log::info!(
                        "[apps] deselected app id={} name={} — total_selected={}",
                        entry.id, entry.name, app.app_selected_ids.len()
                    );
                } else {
                    app.app_selected_ids.insert(entry.id.clone());
                    log::info!(
                        "[apps] selected app id={} name={} — total_selected={}",
                        entry.id, entry.name, app.app_selected_ids.len()
                    );
                }
            }
        }
        KeyCode::Esc => {
            log::debug!("[apps] Esc — returning focus to section panel");
            app.app_focus = AppFocus::Section;
        }
        KeyCode::Char('i') => {
            log::debug!("[apps] 'i' — switching to custom input mode");
            app.app_focus = AppFocus::CustomInput;
        }
        KeyCode::Char('d') => {
            log::info!(
                "[apps] install triggered — selected_count={} ids={:?}",
                app.app_selected_ids.len(),
                app.app_selected_ids
            );
            start_install(app);
        }
        _ => {}
    }
}

fn handle_custom_input_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Backspace => {
            app.app_custom_input.pop();
            log::debug!(
                "[custom_input] backspace — buffer={:?}",
                app.app_custom_input
            );
        }
        KeyCode::Enter => {
            let id = app.app_custom_input.trim().to_string();
            if !id.is_empty() {
                log::info!(
                    "[custom_input] Enter — adding custom id={:?} to selection",
                    id
                );
                app.app_selected_ids.insert(id);
                app.app_custom_input.clear();
            } else {
                log::debug!("[custom_input] Enter with empty buffer — ignoring");
            }
            app.app_focus = AppFocus::Apps;
        }
        KeyCode::Esc => {
            log::debug!(
                "[custom_input] Esc — discarding buffer={:?}",
                app.app_custom_input
            );
            app.app_custom_input.clear();
            app.app_focus = AppFocus::Apps;
        }
        KeyCode::Char(c) => {
            if c == ' ' {
                let id = app.app_custom_input.trim().to_string();
                if !id.is_empty() {
                    log::info!(
                        "[custom_input] Space — adding custom id={:?} to selection",
                        id
                    );
                    app.app_selected_ids.insert(id);
                    app.app_custom_input.clear();
                }
            } else {
                app.app_custom_input.push(c);
                log::debug!(
                    "[custom_input] typed {:?} — buffer={:?}",
                    c, app.app_custom_input
                );
            }
        }
        _ => {}
    }
}

fn handle_installing_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char(c) => {
            app.install_input.push(c);
            log::debug!(
                "[installing] typed {:?} — buffer={:?}",
                c, app.install_input
            );
        }
        KeyCode::Backspace => {
            app.install_input.pop();
            log::debug!(
                "[installing] backspace — buffer={:?}",
                app.install_input
            );
        }
        KeyCode::Enter => {
            let input = app.install_input.trim().to_string();
            app.install_input.clear();
            log::info!("[installing] sending input to process stdin: {:?}", input);
            app.app_install_log.push(format!("> {}", input));
            if let Some(tx) = &app.install_tx {
                let _ = tx.send(input);
            } else {
                log::warn!("[installing] Enter pressed but install_tx is None — nothing sent");
            }
        }
        KeyCode::Esc => {
            log::info!(
                "[installing] Esc — user closed install modal (log_lines={})",
                app.app_install_log.len()
            );
            app.app_installing = false;
            app.app_focus = AppFocus::Section;
            app.install_rx = None;
            app.install_tx = None; // dropping this unblocks the stdin writer thread
            app.install_input.clear();
            app.app_selected_ids.clear();
            app.app_install_log.clear();
        }
        _ => {}
    }
}

fn start_install(app: &mut App) {
    if app.app_selected_ids.is_empty() {
        log::warn!("[start_install] called with no apps selected — aborting");
        return;
    }

    let mgr = app.active_package_manager().to_string();
    log::info!(
        "[start_install] manager={} selected_ids={:?}",
        mgr,
        app.app_selected_ids
    );

    if requires_sudo(&mgr) && !is_root() {
        log::info!(
            "[start_install] {} requires sudo and user is not root — showing sudo confirmation",
            mgr
        );
        app.app_sudo_pending = true;
        app.app_sudo_command = build_commands(app);
        log::debug!(
            "[start_install] staged sudo commands: {:?}",
            app.app_sudo_command
        );
        app.app_focus = AppFocus::SudoConfirm;
        return;
    }

    log::info!("[start_install] no sudo required — running directly");
    execute_install(app, false);
}

fn build_commands(app: &App) -> Vec<String> {
    let mgr = app.active_package_manager();
    let mut commands = vec![];

    if let Some(apps) = &app.apps {
        for id in &app.app_selected_ids {
            if let Some(entry) = apps.iter().flat_map(|s| &s.apps).find(|e| &e.id == id)
                && let Some(cmd) = get_install_command(entry, mgr)
            {
                log::debug!("[build_commands] id={} → cmd={}", id, cmd);
                commands.push(cmd);
            } else {
                log::warn!("[build_commands] id={} found in selection but no install command resolved for manager={}", id, mgr);
            }
        }
    }

    let known_ids: HashSet<String> = app
        .apps
        .as_ref()
        .map(|apps| {
            apps.iter()
                .flat_map(|s| &s.apps)
                .map(|e| e.id.clone())
                .collect()
        })
        .unwrap_or_default();

    for id in app
        .app_selected_ids
        .iter()
        .filter(|id| !known_ids.contains(*id))
    {
        let cmd = install_command(mgr, id);
        log::debug!("[build_commands] custom id={} → cmd={}", id, cmd);
        commands.push(cmd);
    }

    log::info!(
        "[build_commands] built {} command(s) for manager={}",
        commands.len(),
        mgr
    );
    commands
}

fn execute_install(app: &mut App, use_sudo: bool) {
    let commands = if use_sudo {
        app.app_sudo_command
            .drain(..)
            .map(|cmd| format!("sudo {}", cmd))
            .collect::<Vec<_>>()
    } else {
        build_commands(app)
    };

    log::info!(
        "[execute_install] starting — use_sudo={} manager={} command_count={}",
        use_sudo,
        app.active_package_manager(),
        commands.len()
    );
    for cmd in &commands {
        log::debug!("[execute_install] queued: {}", cmd);
    }

    app.app_install_log.clear();
    app.app_install_log
        .push("Starting installation...".to_string());
    app.app_installing = true;
    app.app_focus = AppFocus::Installing;

    let (tx, rx) = mpsc::channel::<String>();
    app.install_rx = Some(rx);

    thread::spawn(move || {
        for cmd in commands {
            let _ = tx.send(format!("▶ Running: {}", cmd));
            log::info!("[execute_install:thread] running: {}", cmd);

            let (binary, args) = split_command(&cmd);
            if binary.is_empty() {
                log::warn!("[execute_install:thread] empty binary — skipping");
                continue;
            }

            match Command::new(&binary)
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(mut child) => {
                    log::debug!(
                        "[execute_install:thread] spawned {} pid={:?}",
                        binary,
                        child.id()
                    );
                    if let Some(stdout) = child.stdout.take() {
                        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                            let line = sanitize_line(line);
                            log::debug!("[execute_install:stdout] {}", line);
                            if tx.send(line).is_err() {
                                log::warn!("[execute_install:stdout] channel closed — stopping");
                                return;
                            }
                        }
                    }
                    if let Some(stderr) = child.stderr.take() {
                        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                            let line = sanitize_line(line);
                            log::debug!("[execute_install:stderr] {}", line);
                            if tx.send(format!("[err] {}", line)).is_err() {
                                log::warn!("[execute_install:stderr] channel closed — stopping");
                                return;
                            }
                        }
                    }
                    match child.wait() {
                        Ok(s) if s.success() => {
                            log::info!("[execute_install:thread] {} exited OK", binary);
                            let _ = tx.send(format!("✓ Done: {}", binary));
                        }
                        Ok(s) => {
                            log::warn!(
                                "[execute_install:thread] {} exited with status {}",
                                binary, s
                            );
                            let _ = tx.send(format!("✗ Failed (exit {})", s));
                        }
                        Err(e) => {
                            log::error!("[execute_install:thread] wait error for {}: {}", binary, e);
                            let _ = tx.send(format!("✗ Error: {}", e));
                        }
                    }
                }
                Err(e) => {
                    log::error!("[execute_install:thread] could not spawn {}: {}", binary, e);
                    let _ = tx.send(format!("✗ Could not run {}: {}", binary, e));
                }
            }
        }
        log::info!("[execute_install:thread] all commands finished");
        let _ = tx.send("═══ All done ═══".to_string());
    });
}

fn render_sudo_confirmation(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 30, frame.area());
    frame.render_widget(Clear, area);

    let mgr = app.active_package_manager();
    let count = app.app_selected_ids.len();

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {} requires sudo to install packages.", mgr),
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {} app(s) selected.", count),
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  [y] ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Run with sudo    ", Style::default().fg(Color::White)),
            Span::styled(
                "[n] ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled("Cancel", Style::default().fg(Color::White)),
        ]),
    ];

    let modal = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Sudo Required ")
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(modal, area);
}