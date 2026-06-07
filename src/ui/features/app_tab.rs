use std::collections::HashSet;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use crate::app::App;
use crate::models::Apps;
use crate::service::{filter_apps_by_platform, get_install_command, install_command, is_root, read_apps_json, requires_sudo};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use crate::enumerate::AppFocus;

pub fn app_render(frame: &mut Frame, sidebar: Rect, body: Rect, app: &mut App) {
    if app.apps.is_none() {
        let raw = read_apps_json().unwrap_or_else(|e| {
            log::error!("Failed to load apps.json: {}", e);
            vec![]
        });
        // Filter to current platform immediately after load
        app.apps = Some(filter_apps_by_platform(&raw, app.active_package_manager()));
    }

    let apps = app.apps.as_ref().unwrap();

    let body_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
        .split(body);

    // Drain install log channel
    if let Some(rx) = &app.install_rx {
        while let Ok(line) = rx.try_recv() {
            app.app_install_log.push(line);
        }
    }

    render_sidebar(frame, sidebar, apps, app);
    render_app_list(frame, body_chunks[0], apps, app);
    render_custom_input(frame, body_chunks[1], app);

    // Overlay install modal if installing
    if app.app_installing {
        render_install_modal(frame, app);
    }

    if app.app_sudo_pending {
        render_sudo_confirmation(frame, app);
    } else if app.app_installing {
        render_install_modal(frame, app);
    }
}

// src/ui/features/app_tab.rs
fn render_sidebar(frame: &mut Frame, area: Rect, apps: &Apps, app: &App) {
    let focused = app.app_focus == AppFocus::Section;

    let items: Vec<ListItem> = apps
        .iter()
        .enumerate()
        .map(|(i, section)| {
            let selected = i == app.app_selected_section;
            let style = match (selected, focused) {
                (true, true)  => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                (true, false) => Style::default().fg(Color::Yellow),
                _             => Style::default().fg(Color::DarkGray),
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
        frame.render_widget(
            Paragraph::new("No section selected")
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Applications ({})", app.active_package_manager()))), // ← here
            area,
        );
        return;
    };

    let items: Vec<ListItem> = section
        .apps
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_cursor   = focused && i == app.app_selected_app;
            let is_selected = app.app_selected_ids.contains(&entry.id);

            let checkbox = if is_selected { "[✓]" } else { "[ ]" };
            let arrow    = if is_cursor   { "▶ " } else { "  " };

            let style = match (is_cursor, is_selected) {
                (true, _)      => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                (_, true)      => Style::default().fg(Color::Green),
                _              => Style::default().fg(Color::White),
            };

            ListItem::new(Line::from(Span::styled(
                format!("{}{} {}", arrow, checkbox, entry.name),
                style,
            )))
        })
        .collect();

    let selected_count = app.app_selected_ids.len();
    // Section active
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
            Span::styled(
                "▌",
                Style::default().fg(Color::Magenta),
            )
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

    let input = Paragraph::new(Line::from(content))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Custom App [i]")
                .border_style(border_style),
        );

    frame.render_widget(input, area);
}

fn render_install_modal(frame: &mut Frame, app: &App) {
    // Center a modal over the whole terminal
    let area = centered_rect(70, 60, frame.area());

    // Clear background behind modal
    frame.render_widget(Clear, area);

    let log_text: Vec<Line> = app.app_install_log
        .iter()
        .map(|l| Line::from(Span::styled(l.clone(), Style::default().fg(Color::Green))))
        .collect();

    let modal = Paragraph::new(log_text)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Installing... (Esc to close) ")
                .border_style(Style::default().fg(Color::Yellow)),
        );

    frame.render_widget(modal, area);
}

/// Returns a centered Rect of given percentage width/height
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
    match app.app_focus {
        AppFocus::Section     => handle_section_keys(app, key),
        AppFocus::Apps        => handle_apps_keys(app, key),
        AppFocus::CustomInput => handle_custom_input_keys(app, key),
        AppFocus::Installing  => handle_installing_keys(app, key),
        AppFocus::SudoConfirm  => handle_sudo_confirm_keys(app, key),
    }
}

fn handle_sudo_confirm_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char(c) => {
            app.app_sudo_password.push(c);
        }
        KeyCode::Backspace => {
            app.app_sudo_password.pop();
        }
        KeyCode::Enter => {
            if app.app_sudo_password.is_empty() {
                return; // don't proceed without password
            }
            let password = app.app_sudo_password.clone();
            app.app_sudo_password.clear(); // clear immediately after use
            app.app_sudo_pending = false;
            app.app_focus = AppFocus::Installing;
            execute_install_with_password(app, password);
        }
        KeyCode::Esc => {
            app.app_sudo_pending = false;
            app.app_sudo_password.clear();
            app.app_sudo_command.clear();
            app.app_focus = AppFocus::Apps;
        }
        _ => {}
    }
}

fn execute_install_with_password(app: &mut App, password: String) {
    let commands = app.app_sudo_command
        .drain(..)
        .map(|cmd| format!("sudo -S {}", cmd)) // -S reads password from stdin
        .collect::<Vec<_>>();

    app.app_install_log.clear();
    app.app_install_log.push("Starting installation...".to_string());
    app.app_installing = true;

    let (tx, rx) = mpsc::channel::<String>();
    app.install_rx = Some(rx);

    thread::spawn(move || {
        for cmd in commands {
            let _ = tx.send(format!("▶ Running: {}", cmd));

            let mut parts = cmd.splitn(2, ' ');
            let binary = match parts.next() {
                Some(b) => b.to_string(),
                None => continue,
            };
            let args: Vec<String> = parts
                .next()
                .unwrap_or("")
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();

            match Command::new(&binary)
                .args(&args)
                .stdin(Stdio::piped())   // ← pipe password in
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(mut child) => {
                    // Write password to stdin for sudo -S
                    if let Some(mut stdin) = child.stdin.take() {
                        use std::io::Write;
                        let _ = writeln!(stdin, "{}", password);
                    }
                    if let Some(stdout) = child.stdout.take() {
                        for line in BufReader::new(stdout).lines().flatten() {
                            if tx.send(line).is_err() { return; }
                        }
                    }
                    if let Some(stderr) = child.stderr.take() {
                        for line in BufReader::new(stderr).lines().flatten() {
                            // Filter out sudo password prompt from output
                            if !line.contains("[sudo]") && !line.contains("password for") {
                                if tx.send(format!("[err] {}", line)).is_err() { return; }
                            }
                        }
                    }
                    match child.wait() {
                        Ok(s) if s.success() => {
                            let _ = tx.send(format!("✓ Done: {}", binary));
                        }
                        Ok(s) => {
                            let _ = tx.send(format!("✗ Failed (exit {})", s));
                        }
                        Err(e) => {
                            let _ = tx.send(format!("✗ Error: {}", e));
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(format!("✗ Could not run {}: {}", binary, e));
                }
            }
        }
        let _ = tx.send("═══ All done ═══".to_string());
    });
}

fn handle_section_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.app_selected_section > 0 {
                app.app_selected_section -= 1;
                app.app_selected_app = 0;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(apps) = &app.apps {
                if app.app_selected_section < apps.len().saturating_sub(1) {
                    app.app_selected_section += 1;
                    app.app_selected_app = 0;
                }
            }
        }
        // Enter app list for this section
        KeyCode::Char(' ') | KeyCode::Enter => {
            app.app_focus = AppFocus::Apps;
            app.app_selected_app = 0;
        }
        // Install all selected
        KeyCode::Char('d') => {
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
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(apps) = &app.apps {
                if let Some(section) = apps.get(app.app_selected_section) {
                    if app.app_selected_app < section.apps.len().saturating_sub(1) {
                        app.app_selected_app += 1;
                    }
                }
            }
        }
        // Toggle selection
        KeyCode::Char(' ') => {
            if let Some(apps) = &app.apps {
                if let Some(section) = apps.get(app.app_selected_section) {
                    if let Some(entry) = section.apps.get(app.app_selected_app) {
                        if app.app_selected_ids.contains(&entry.id) {
                            app.app_selected_ids.remove(&entry.id);
                        } else {
                            app.app_selected_ids.insert(entry.id.clone());
                        }
                    }
                }
            }
        }
        // Back to section
        KeyCode::Esc => {
            app.app_focus = AppFocus::Section;
        }
        // Custom input
        KeyCode::Char('i') => {
            app.app_focus = AppFocus::CustomInput;
        }
        // Install all selected
        KeyCode::Char('d') => {
            start_install(app);
        }
        _ => {}
    }
}

fn handle_custom_input_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Backspace => {
            app.app_custom_input.pop();
        }
        KeyCode::Enter => {
            let id = app.app_custom_input.trim().to_string();
            if !id.is_empty() {
                app.app_selected_ids.insert(id);
                app.app_custom_input.clear();
            }
            app.app_focus = AppFocus::Apps;
        }
        KeyCode::Esc => {
            app.app_custom_input.clear();
            app.app_focus = AppFocus::Apps;
        }
        KeyCode::Char(c) => {
            if c == ' ' {
                // Space adds current input to selected list and clears
                let id = app.app_custom_input.trim().to_string();
                if !id.is_empty() {
                    app.app_selected_ids.insert(id);
                    app.app_custom_input.clear();
                }
            } else {
                app.app_custom_input.push(c);
            }
        }
        _ => {}
    }
}

fn handle_installing_keys(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Esc {
        app.app_installing = false;
        app.app_focus = AppFocus::Section;
    }
}

fn handle_installing_keys1(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Esc {
        app.app_installing = false;
        app.app_installing = false;
        app.app_focus = AppFocus::Section;
        app.install_rx = None;          // drop channel
        app.app_selected_ids.clear();   // clear selection after install
        app.app_install_log.clear();
    }
}

// fn start_install(app: &mut App) {
//     if app.app_selected_ids.is_empty() {
//         return;
//     }
//     app.app_install_log.clear();
//     app.app_install_log.push("Starting installation...".to_string());
//     app.app_installing = true;
//     app.app_focus = AppFocus::Installing;
//
//     // TODO: spawn async install process and stream output into app.app_install_log
//     // e.g. std::thread::spawn + channel to push log lines each frame
//     for id in &app.app_selected_ids {
//         app.app_install_log.push(format!("▶ queued: {}", id));
//     }
// }

fn start_install(app: &mut App) {
    if app.app_selected_ids.is_empty() {
        return;
    }

    let mgr = app.active_package_manager().to_string();

    // If manager requires sudo and user is not root, show confirmation
    if requires_sudo(&mgr) && !is_root() {
        app.app_sudo_pending = true;
        // Store commands for later execution after confirmation
        app.app_sudo_command = build_commands(app);
        app.app_focus = AppFocus::SudoConfirm;  // ← add this
        return;
    }

    execute_install(app, false);
}

fn build_commands(app: &App) -> Vec<String> {
    let mgr = app.active_package_manager();
    let mut commands = vec![];

    if let Some(apps) = &app.apps {
        for id in &app.app_selected_ids {
            if let Some(entry) = apps.iter()
                .flat_map(|s| &s.apps)
                .find(|e| &e.id == id)
            {
                if let Some(cmd) = get_install_command(entry, mgr) {
                    commands.push(cmd);
                }
            }
        }
    }

    // Custom ids
    let known_ids: HashSet<String> = app.apps.as_ref()
        .map(|apps| apps.iter().flat_map(|s| &s.apps).map(|e| e.id.clone()).collect())
        .unwrap_or_default();

    for id in app.app_selected_ids.iter().filter(|id| !known_ids.contains(*id)) {
        commands.push(install_command(mgr, id));
    }

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

    app.app_install_log.clear();
    app.app_install_log.push("Starting installation...".to_string());
    app.app_installing = true;
    app.app_focus = AppFocus::Installing;

    let (tx, rx) = mpsc::channel::<String>();
    app.install_rx = Some(rx);

    thread::spawn(move || {
        for cmd in commands {
            let _ = tx.send(format!("▶ Running: {}", cmd));

            let mut parts = cmd.splitn(2, ' ');
            let binary = match parts.next() {
                Some(b) => b.to_string(),
                None => continue,
            };
            let args: Vec<String> = parts
                .next()
                .unwrap_or("")
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();

            match Command::new(&binary)
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(mut child) => {
                    if let Some(stdout) = child.stdout.take() {
                        for line in BufReader::new(stdout).lines().flatten() {
                            if tx.send(line).is_err() { return; }
                        }
                    }
                    if let Some(stderr) = child.stderr.take() {
                        for line in BufReader::new(stderr).lines().flatten() {
                            if tx.send(format!("[err] {}", line)).is_err() { return; }
                        }
                    }
                    match child.wait() {
                        Ok(s) if s.success() => { let _ = tx.send(format!("✓ Done: {}", binary)); }
                        Ok(s)                => { let _ = tx.send(format!("✗ Failed (exit {})", s)); }
                        Err(e)               => { let _ = tx.send(format!("✗ Error: {}", e)); }
                    }
                }
                Err(e) => { let _ = tx.send(format!("✗ Could not run {}: {}", binary, e)); }
            }
        }
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
            Span::styled("  [y] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("Run with sudo    ", Style::default().fg(Color::White)),
            Span::styled("[n] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("Cancel", Style::default().fg(Color::White)),
        ]),
    ];

    let modal = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Sudo Required ")
                .border_style(Style::default().fg(Color::Yellow)),
        );

    frame.render_widget(modal, area);
}