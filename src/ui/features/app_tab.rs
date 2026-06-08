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
        tracing::info!(
            "[app_render] apps cache is empty — loading apps.json for manager={}",
            app.active_package_manager()
        );
        let raw = read_apps_json().unwrap_or_else(|e| {
            tracing::error!("[app_render] Failed to load apps.json: {}", e);
            vec![]
        });
        app.apps = Some(filter_apps_by_platform(&raw, app.active_package_manager()));
        tracing::debug!(
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
            tracing::debug!("[app_render] drained {} new install log lines", added);
        }
    }

    // Drain search result channel (winget only)
    if let Some(rx) = &app.search_rx {
        if let Ok(results) = rx.try_recv() {
            tracing::info!("[app_render] received {} search results", results.len());
            app.search_results = results;
            app.search_loading = false;
            app.search_selected = 0;
            app.search_rx = None;
        }
    }

    render_sidebar(frame, sidebar, apps, app);

    // If in Search focus, replace the apps list panel with search results
    if app.app_focus == AppFocus::Search {
        render_search_panel(frame, body_chunks[0], app);
    } else {
        render_app_list(frame, body_chunks[0], apps, app);
    }

    render_custom_input(frame, body_chunks[1], app);

    if app.app_sudo_pending {
        tracing::debug!("[app_render] rendering sudo confirmation modal");
        render_sudo_confirmation(frame, app);
    } else if app.app_installing {
        tracing::debug!("[app_render] rendering install modal (log_lines={})", app.app_install_log.len());
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
        tracing::warn!(
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

// ─── Search panel (winget only) ──────────────────────────────────────────────

fn render_search_panel(frame: &mut Frame, area: Rect, app: &App) {
    let mgr = app.active_package_manager();

    // Title shows query and result count
    let title = if app.search_loading {
        format!(" Search [{}] — searching… ", mgr)
    } else if app.search_results.is_empty() && !app.search_query.is_empty() {
        format!(" Search [{}] — no results ", mgr)
    } else {
        format!(
            " Search [{}] — {} result(s) ",
            mgr,
            app.search_results.len()
        )
    };

    if app.search_loading {
        let loading = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  Searching, please wait…",
                Style::default().fg(Color::Yellow),
            )),
        ])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(Color::Yellow)),
            );
        frame.render_widget(loading, area);
        return;
    }

    if app.search_results.is_empty() {
        let empty_msg = if app.search_query.is_empty() {
            "  Type a query in the input below and press Enter to search."
        } else {
            "  No results found. Try a different query."
        };
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                empty_msg,
                Style::default().fg(Color::DarkGray),
            )),
        ])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        frame.render_widget(empty, area);
        return;
    }

    // Compute visible column widths — name gets ~50%, id gets ~35%, version ~15%
    let inner_w = area.width.saturating_sub(2); // subtract borders
    let name_w = (inner_w as f32 * 0.40) as usize;
    let id_w   = (inner_w as f32 * 0.40) as usize;
    let ver_w  = inner_w.saturating_sub((name_w + id_w) as u16) as usize;

    let items: Vec<ListItem> = app
        .search_results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let is_cursor = i == app.search_selected;
            let is_picked = app.app_selected_ids.contains(&result.id);

            let checkbox = if is_picked { "[✓]" } else { "[ ]" };
            let arrow    = if is_cursor { "▶ " } else { "  " };

            // Truncate each column to its allotted width
            let name = truncate(&result.name, name_w);
            let id   = truncate(&result.id,   id_w);
            let ver  = truncate(&result.version, ver_w);

            let style = match (is_cursor, is_picked) {
                (true, _) => Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
                (_, true) => Style::default().fg(Color::Green),
                _         => Style::default().fg(Color::White),
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("{}{} ", arrow, checkbox), style),
                Span::styled(format!("{:<width$}  ", name, width = name_w), style),
                Span::styled(
                    format!("{:<width$}  ", id, width = id_w),
                    Style::default().fg(if is_cursor { Color::Cyan } else { Color::Yellow }),
                ),
                Span::styled(
                    ver,
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(list, area);
}

fn truncate(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", &s.chars().take(max.saturating_sub(1)).collect::<String>())
    }
}

fn render_custom_input(frame: &mut Frame, area: Rect, app: &App) {
    let focused = app.app_focus == AppFocus::CustomInput
        || app.app_focus == AppFocus::Search;

    let border_style = if focused {
        Style::default().fg(Color::Magenta)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let mgr = app.active_package_manager();
    let is_winget = mgr == "winget";

    let (title, content) = if app.app_focus == AppFocus::Search {
        // Search mode (winget): show the search query buffer
        let buf = &app.search_query;
        let display = if buf.is_empty() {
            Span::styled("▌", Style::default().fg(Color::Magenta))
        } else {
            Span::styled(
                format!("{}▌", buf),
                Style::default().fg(Color::White),
            )
        };
        (" Search (Enter to run, Esc to cancel) [i]", display)
    } else if focused {
        // CustomInput mode (Linux): plain name entry
        let buf = &app.app_custom_input;
        let display = if buf.is_empty() {
            Span::styled("▌", Style::default().fg(Color::Magenta))
        } else {
            Span::styled(
                format!("{}▌", buf),
                Style::default().fg(Color::White),
            )
        };
        (" Custom Package [i]", display)
    } else {
        // Idle hint — differs by manager
        let hint = if is_winget {
            "Press [i] to search winget"
        } else {
            "Press [i] to enter package name"
        };
        let display = Span::styled(hint.to_string(), Style::default().fg(Color::DarkGray));
        (" Search / Custom [i]", display)
    };

    let input = Paragraph::new(Line::from(content)).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
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
    tracing::debug!(
        "[handle_key] focus={:?} key={:?}",
        app.app_focus,
        key.code
    );
    match app.app_focus {
        AppFocus::Section     => handle_section_keys(app, key),
        AppFocus::Apps        => handle_apps_keys(app, key),
        AppFocus::CustomInput => handle_custom_input_keys(app, key),
        AppFocus::Search      => handle_search_keys(app, key),
        AppFocus::Installing  => handle_installing_keys(app, key),
        AppFocus::SudoConfirm => handle_sudo_confirm_keys(app, key),
    }
}

fn handle_sudo_confirm_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char(c) => {
            app.app_sudo_password.push(c);
            tracing::debug!(
                "[sudo_confirm] password input length={}",
                app.app_sudo_password.len()
            );
        }
        KeyCode::Backspace => {
            app.app_sudo_password.pop();
            tracing::debug!(
                "[sudo_confirm] backspace — password length={}",
                app.app_sudo_password.len()
            );
        }
        KeyCode::Enter => {
            if app.app_sudo_password.is_empty() {
                tracing::warn!("[sudo_confirm] Enter pressed but password is empty — ignoring");
                return;
            }
            tracing::info!(
                "[sudo_confirm] password confirmed — launching interactive install with sudo, manager={}",
                app.active_package_manager()
            );
            let password = app.app_sudo_password.clone();
            app.app_sudo_password.clear();
            app.app_sudo_pending = false;
            execute_install_interactive(app, Some(password));
        }
        KeyCode::Esc => {
            tracing::info!("[sudo_confirm] cancelled by user — clearing sudo state");
            app.app_sudo_pending = false;
            app.app_sudo_password.clear();
            app.app_sudo_command.clear();
            app.app_focus = AppFocus::Apps;
        }
        _ => {}
    }
}

// ─── Search key handler (winget) ─────────────────────────────────────────────

fn handle_search_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        // Navigate results
        KeyCode::Up | KeyCode::Char('k') => {
            if app.search_selected > 0 {
                app.search_selected -= 1;
                tracing::debug!("[search] navigate up — cursor={}", app.search_selected);
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.search_selected < app.search_results.len().saturating_sub(1) {
                app.search_selected += 1;
                tracing::debug!("[search] navigate down — cursor={}", app.search_selected);
            }
        }

        // Toggle-select the highlighted result (adds winget ID to install list)
        KeyCode::Char(' ') => {
            if let Some(result) = app.search_results.get(app.search_selected) {
                if app.app_selected_ids.contains(&result.id) {
                    app.app_selected_ids.remove(&result.id);
                    tracing::info!(
                        "[search] deselected id={} name={} — total_selected={}",
                        result.id, result.name, app.app_selected_ids.len()
                    );
                } else {
                    app.app_selected_ids.insert(result.id.clone());
                    tracing::info!(
                        "[search] selected id={} name={} — total_selected={}",
                        result.id, result.name, app.app_selected_ids.len()
                    );
                }
            }
        }

        // Re-run search with current query
        KeyCode::Enter => {
            if !app.search_query.is_empty() && !app.search_loading {
                tracing::info!("[search] re-running search for {:?}", app.search_query);
                run_winget_search(app);
            }
        }

        // Typing updates the query buffer — user presses Enter to search
        KeyCode::Char(c) => {
            app.search_query.push(c);
            tracing::debug!("[search] typed {:?} — query={:?}", c, app.search_query);
        }
        KeyCode::Backspace => {
            app.search_query.pop();
            tracing::debug!("[search] backspace — query={:?}", app.search_query);
        }

        // Close search panel, return to apps list
        KeyCode::Esc => {
            tracing::info!("[search] closed — returning to Apps focus");
            app.app_focus = AppFocus::Apps;
            app.search_results.clear();
            app.search_query.clear();
            app.search_loading = false;
            app.search_rx = None;
        }

        _ => {}
    }
}

// ─── Winget search runner ─────────────────────────────────────────────────────

fn run_winget_search(app: &mut App) {
    let query = app.search_query.clone();
    tracing::info!("[run_winget_search] launching: winget search {}", query);

    app.search_loading = true;
    app.search_results.clear();

    let (tx, rx) = mpsc::channel::<Vec<SearchResult>>();
    app.search_rx = Some(rx);

    thread::spawn(move || {
        let output = Command::new("winget")
            // Pass query as a plain positional arg — no --query flag, no quoting.
            // --disable-interactivity is intentionally omitted: on many Windows
            // setups it suppresses all output, causing empty results.
            .args(["search", &query])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();

        match output {
            Ok(out) => {
                // Winget on Windows outputs UTF-16 LE in some environments.
                // Try UTF-8 first (works in most modern setups), then fall back
                // to a lossy decode that strips unrecognised bytes rather than
                // returning an empty string.
                let text = decode_winget_output(&out.stdout);
                tracing::debug!("[winget_search:thread] raw output ({} bytes):\n{}", out.stdout.len(), &text);
                let results = parse_winget_search(&text);
                tracing::info!(
                    "[winget_search:thread] parsed {} results for {:?}",
                    results.len(),
                    query
                );
                let _ = tx.send(results);
            }
            Err(e) => {
                tracing::error!("[winget_search:thread] failed to run winget: {}", e);
                let _ = tx.send(vec![]);
            }
        }
    });
}

/// Decode winget's stdout bytes to a String.
///
/// Winget can emit:
///   - Plain UTF-8 (modern Windows Terminal / PowerShell 7)
///   - UTF-16 LE with BOM (older CMD environments)
///   - UTF-8 with a BOM (0xEF 0xBB 0xBF)
///
/// We detect the BOM and decode accordingly, falling back to lossy UTF-8.
fn decode_winget_output(bytes: &[u8]) -> String {
    // UTF-16 LE BOM: FF FE
    if bytes.starts_with(&[0xFF, 0xFE]) {
        let words: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect();
        return String::from_utf16_lossy(&words);
    }
    // UTF-8 BOM: EF BB BF — strip it
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8_lossy(&bytes[3..]).into_owned();
    }
    // Plain UTF-8
    String::from_utf8_lossy(bytes).into_owned()
}

/// Parse `winget search` tabular output into structured results.
///
/// Winget output looks like:
/// ```
/// Name             Id                      Version   Source
/// ---------------------------------------------------------------
/// PowerShell       Microsoft.PowerShell    7.4.1     winget
/// ```
/// We locate the header row to find column offsets, then slice each
/// data row at those offsets.
fn parse_winget_search(output: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let mut header_offsets: Option<(usize, usize, usize)> = None;

    // Winget uses bare \r (carriage return) to overwrite its progress spinner
    // in place, so the entire output arrives as one giant "line" when split on
    // \n.  We must split on \r first, then also on \n, deduplicate blanks,
    // and strip ANSI codes from each segment before column detection.
    let segments: Vec<String> = output
        .split(|c| c == '\r' || c == '\n')
        .map(|s| strip_ansi(s))
        .collect();

    for line in &segments {
        let line = line.as_str();

        // Skip blank lines, pure-control lines, and winget progress bar lines.
        // Progress lines contain block-drawing chars (█ ▒) or KB/MB markers.
        if line.trim().is_empty() {
            continue;
        }
        if line.contains('█') || line.contains('▒') || line.contains("KB /") || line.contains("MB /") {
            continue;
        }
        // Skip the spinner frames winget emits before the table (-, \, |, /)
        let trimmed = line.trim();
        if trimmed == "-" || trimmed == "\\" || trimmed == "|" || trimmed == "/" {
            continue;
        }

        // Detect header row
        if header_offsets.is_none() {
            let lower = line.to_lowercase();
            if lower.contains("name") && lower.contains("id") && lower.contains("version") {
                let id_pos  = find_col(&line, "Id").or_else(|| find_col(&line, "id"));
                let ver_pos = find_col(&line, "Version").or_else(|| find_col(&line, "version"));
                let src_pos = find_col(&line, "Source").or_else(|| find_col(&line, "source"));

                if let (Some(id), Some(ver)) = (id_pos, ver_pos) {
                    if id > 0 && ver > id {
                        header_offsets = Some((id, ver, src_pos.unwrap_or(usize::MAX)));
                        tracing::debug!(
                            "[parse_winget] header: id_col={} ver_col={} src_col={:?} line={:?}",
                            id, ver, src_pos, line
                        );
                    }
                }
            }
            continue;
        }

        // Skip separator lines (all dashes/spaces)
        if line.chars().all(|c| c == '-' || c == ' ') {
            continue;
        }

        if let Some((id_start, ver_start, src_start)) = header_offsets {
            let chars: Vec<char> = line.chars().collect();
            let len = chars.len();

            if len < id_start {
                continue;
            }

            let name = chars[..id_start]
                .iter().collect::<String>().trim().to_string();
            let id = chars[id_start..ver_start.min(len)]
                .iter().collect::<String>().trim().to_string();
            let version = if len > ver_start {
                chars[ver_start..src_start.min(len)]
                    .iter().collect::<String>().trim().to_string()
            } else {
                String::new()
            };

            if !name.is_empty() && !id.is_empty() {
                tracing::debug!("[parse_winget] row: name={:?} id={:?} ver={:?}", name, id, version);
                results.push(SearchResult { name, id, version });
            }
        }
    }

    results
}

/// Remove ANSI escape sequences from a string.
///
/// Winget emits cursor/erase sequences (ESC [ ... m/K/J/A/h/l) as part of
/// its progress spinner. These are invisible in a real terminal but appear
/// as raw bytes when stdout is piped, inflating column offsets.
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // ESC — consume the rest of the escape sequence
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                // Consume until a letter (the final byte of the sequence)
                for sc in chars.by_ref() {
                    if sc.is_ascii_alphabetic() {
                        break;
                    }
                }
            } else {
                // Two-char escape (ESC + single char) — skip next char
                chars.next();
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Find the char-index of the first occurrence of `needle` in `haystack`.
fn find_col(haystack: &str, needle: &str) -> Option<usize> {
    let h: Vec<char> = haystack.chars().collect();
    let n: Vec<char> = needle.chars().collect();
    let n_len = n.len();
    if n_len == 0 || h.len() < n_len { return None; }
    for i in 0..=(h.len() - n_len) {
        if h[i..i + n_len] == n[..] { return Some(i); }
    }
    None
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn sanitize_line(line: String) -> String {
    line.trim_end_matches('\r').to_string()
}

/// Returns true for lines that should be hidden in the install log.
///
/// Winget uses bare \r to animate a progress spinner and download bar in
/// place. When stdout is piped those \r sequences arrive inside a single
/// BufReader "line". We split on \r here and filter out the noise segments
/// so only meaningful status lines reach the UI.
fn is_noise_line(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() { return true; }
    // Spinner frames
    if t == "-" || t == "\\" || t == "|" || t == "/" { return true; }
    // Progress bar / download bar characters
    if t.contains('█') || t.contains('▒') { return true; }
    // KB/MB download progress markers
    if t.contains("KB /") || t.contains("MB /") || t.contains("GB /") { return true; }
    // Percentage-only lines (e.g. "  55%")
    if t.ends_with('%') && t.trim_end_matches('%').trim().chars().all(|c| c.is_ascii_digit()) { return true; }
    false
}

/// Read all stdout lines from a child process, split on both \n and \r,
/// strip ANSI codes, filter noise, and send each meaningful line to `tx`.
/// Returns false if the channel was closed (caller should abort).
fn drain_stdout_to_log(
    stdout: std::process::ChildStdout,
    tx: &mpsc::Sender<String>,
    tag: &str,
) -> bool {
    use std::io::Read;
    let mut raw = Vec::new();
    let mut reader = BufReader::new(stdout);
    // Read the entire stdout (blocks until process closes its end).
    // We use read_to_end so we capture everything including partial lines.
    if reader.read_to_end(&mut raw).is_err() {
        return true;
    }
    let text = String::from_utf8_lossy(&raw);
    for segment in text.split(|c| c == '\r' || c == '\n') {
        let clean = strip_ansi(segment);
        if is_noise_line(&clean) {
            tracing::debug!("[{}:stdout] (noise) {:?}", tag, clean);
            continue;
        }
        tracing::debug!("[{}:stdout] {}", tag, clean);
        if tx.send(clean).is_err() {
            tracing::warn!("[{}:stdout] channel closed — stopping", tag);
            return false;
        }
    }
    true
}

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

    tracing::info!(
        "[execute_install_with_password] starting — manager={} command_count={}",
        app.active_package_manager(),
        commands.len()
    );
    for cmd in &commands {
        tracing::debug!("[execute_install_with_password] queued: {}", cmd);
    }

    app.app_install_log.clear();
    app.app_install_log.push("Starting installation...".to_string());
    app.app_install_log.push("Type y/n or Enter to respond to prompts.".to_string());
    app.app_installing = true;

    let (out_tx, out_rx) = mpsc::channel::<String>();
    let (in_tx, in_rx)   = mpsc::channel::<String>();

    app.install_rx = Some(out_rx);
    app.install_tx = Some(in_tx);

    let password_clone: Option<String> = Some(password.clone());

    thread::spawn(move || {
        let in_rx = Arc::new(Mutex::new(in_rx));

        for cmd in commands {
            let _ = out_tx.send(format!("▶ Running: {}", cmd));
            tracing::info!("[install_with_password:thread] running: {}", cmd);

            let (binary, args) = split_command(&cmd);
            if binary.is_empty() {
                tracing::warn!("[install_with_password:thread] empty binary — skipping");
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
                    tracing::debug!(
                        "[install_with_password:thread] spawned {} pid={:?}",
                        binary, child.id()
                    );
                    if let Some(mut stdin) = child.stdin.take() {
                        use std::io::Write;
                        if let Some(ref pwd) = password_clone {
                            tracing::debug!("[install_with_password:thread] writing password to stdin");
                            let _ = writeln!(stdin, "{}", pwd);
                        }
                        let in_rx_clone = Arc::clone(&in_rx);
                        thread::spawn(move || {
                            tracing::debug!("[install_with_password:stdin_relay] waiting for user input");
                            while let Ok(input) = in_rx_clone.lock().unwrap().recv() {
                                tracing::debug!("[install_with_password:stdin_relay] forwarding: {:?}", input);
                                let _ = writeln!(stdin, "{}", input);
                            }
                            tracing::debug!("[install_with_password:stdin_relay] channel closed — exiting");
                        });
                    }

                    if let Some(stdout) = child.stdout.take() {
                        if !drain_stdout_to_log(stdout, &out_tx, "install_with_password") {
                            return;
                        }
                    }

                    if let Some(stderr) = child.stderr.take() {
                        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                            let line = sanitize_line(line);
                            if is_noise_line(&line) { continue; }
                            if !line.contains("[sudo]") && !line.contains("password for") {
                                tracing::debug!("[install_with_password:stderr] {}", line);
                                if out_tx.send(format!("[err] {}", line)).is_err() {
                                    tracing::warn!("[install_with_password:stderr] channel closed — stopping");
                                    return;
                                }
                            }
                        }
                    }

                    match child.wait() {
                        Ok(s) if s.success() => {
                            tracing::info!("[install_with_password:thread] {} exited OK", binary);
                            let _ = out_tx.send(format!("✓ Done: {}", binary));
                        }
                        Ok(s) => {
                            tracing::warn!("[install_with_password:thread] {} exited {}", binary, s);
                            let _ = out_tx.send(format!("✗ Failed (exit {})", s));
                        }
                        Err(e) => {
                            tracing::error!("[install_with_password:thread] wait error for {}: {}", binary, e);
                            let _ = out_tx.send(format!("✗ Error: {}", e));
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("[install_with_password:thread] could not spawn {}: {}", binary, e);
                    let _ = out_tx.send(format!("✗ Could not run {}: {}", binary, e));
                }
            }
        }
        tracing::info!("[install_with_password:thread] all commands finished");
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

    tracing::info!(
        "[execute_install_interactive] starting — manager={} sudo={} command_count={}",
        app.active_package_manager(),
        password.is_some(),
        commands.len()
    );
    for cmd in &commands {
        tracing::debug!("[execute_install_interactive] queued: {}", cmd);
    }

    app.app_install_log.clear();
    app.app_install_log.push("Starting installation...".to_string());
    if requires_interactive(app.active_package_manager()) {
        app.app_install_log.push("Type y/n and press Enter to respond to prompts.".to_string());
    }
    app.app_installing = true;
    app.app_focus = AppFocus::Installing;

    let (out_tx, out_rx) = mpsc::channel::<String>();
    let (in_tx, in_rx)   = mpsc::channel::<String>();

    app.install_rx = Some(out_rx);
    app.install_tx = Some(in_tx);

    let password_clone: Option<String> = password.clone();

    thread::spawn(move || {
        let in_rx = Arc::new(Mutex::new(in_rx));

        for cmd in commands {
            let _ = out_tx.send(format!("▶ Running: {}", cmd));
            tracing::info!("[install_interactive:thread] running: {}", cmd);

            let (binary, args) = split_command(&cmd);
            if binary.is_empty() {
                tracing::warn!("[install_interactive:thread] empty binary — skipping");
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
                    tracing::debug!(
                        "[install_interactive:thread] spawned {} pid={:?}",
                        binary, child.id()
                    );
                    if let Some(mut stdin) = child.stdin.take() {
                        use std::io::Write;
                        if let Some(ref pwd) = password_clone {
                            tracing::debug!("[install_interactive:thread] writing password to stdin");
                            let _ = writeln!(stdin, "{}", pwd);
                        }
                        let in_rx_clone = Arc::clone(&in_rx);
                        thread::spawn(move || {
                            tracing::debug!("[install_interactive:stdin_relay] waiting for user input");
                            while let Ok(input) = in_rx_clone.lock().unwrap().recv() {
                                tracing::debug!("[install_interactive:stdin_relay] forwarding: {:?}", input);
                                let _ = writeln!(stdin, "{}", input);
                            }
                            tracing::debug!("[install_interactive:stdin_relay] channel closed — exiting");
                        });
                    }

                    if let Some(stdout) = child.stdout.take() {
                        if !drain_stdout_to_log(stdout, &out_tx, "install_interactive") {
                            return;
                        }
                    }

                    if let Some(stderr) = child.stderr.take() {
                        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                            let line = sanitize_line(line);
                            if is_noise_line(&line) { continue; }
                            if !line.contains("[sudo]") && !line.contains("password for") {
                                tracing::debug!("[install_interactive:stderr] {}", line);
                                if out_tx.send(format!("[err] {}", line)).is_err() {
                                    tracing::warn!("[install_interactive:stderr] channel closed — stopping");
                                    return;
                                }
                            }
                        }
                    }

                    match child.wait() {
                        Ok(s) if s.success() => {
                            tracing::info!("[install_interactive:thread] {} exited OK", binary);
                            let _ = out_tx.send(format!("✓ Done: {}", binary));
                        }
                        Ok(s) => {
                            tracing::warn!("[install_interactive:thread] {} exited {}", binary, s);
                            let _ = out_tx.send(format!("✗ Failed (exit {})", s));
                        }
                        Err(e) => {
                            tracing::error!("[install_interactive:thread] wait error for {}: {}", binary, e);
                            let _ = out_tx.send(format!("✗ Error: {}", e));
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("[install_interactive:thread] could not spawn {}: {}", binary, e);
                    let _ = out_tx.send(format!("✗ Could not run {}: {}", binary, e));
                }
            }
        }
        tracing::info!("[install_interactive:thread] all commands finished");
        let _ = out_tx.send("═══ All done ═══".to_string());
    });
}

fn handle_section_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.app_selected_section > 0 {
                app.app_selected_section -= 1;
                app.app_selected_app = 0;
                tracing::debug!(
                    "[section] navigate up — section={} app={}",
                    app.app_selected_section, app.app_selected_app
                );
            } else {
                tracing::debug!("[section] navigate up — already at top");
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(apps) = &app.apps
                && app.app_selected_section < apps.len().saturating_sub(1)
            {
                app.app_selected_section += 1;
                app.app_selected_app = 0;
                tracing::debug!(
                    "[section] navigate down — section={} app={}",
                    app.app_selected_section, app.app_selected_app
                );
            } else {
                tracing::debug!("[section] navigate down — already at bottom");
            }
        }
        KeyCode::Char(' ') | KeyCode::Enter => {
            tracing::info!("[section] enter apps list — section={}", app.app_selected_section);
            app.app_focus = AppFocus::Apps;
            app.app_selected_app = 0;
        }
        KeyCode::Char('d') => {
            tracing::info!(
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
                tracing::debug!(
                    "[apps] navigate up — section={} app={}",
                    app.app_selected_section, app.app_selected_app
                );
            } else {
                tracing::debug!("[apps] navigate up — already at top");
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(apps) = &app.apps
                && let Some(section) = apps.get(app.app_selected_section)
                && app.app_selected_app < section.apps.len().saturating_sub(1)
            {
                app.app_selected_app += 1;
                tracing::debug!(
                    "[apps] navigate down — section={} app={}",
                    app.app_selected_section, app.app_selected_app
                );
            } else {
                tracing::debug!("[apps] navigate down — already at bottom");
            }
        }
        KeyCode::Char(' ') => {
            if let Some(apps) = &app.apps
                && let Some(section) = apps.get(app.app_selected_section)
                && let Some(entry) = section.apps.get(app.app_selected_app)
            {
                if app.app_selected_ids.contains(&entry.id) {
                    app.app_selected_ids.remove(&entry.id);
                    tracing::info!(
                        "[apps] deselected app id={} name={} — total_selected={}",
                        entry.id, entry.name, app.app_selected_ids.len()
                    );
                } else {
                    app.app_selected_ids.insert(entry.id.clone());
                    tracing::info!(
                        "[apps] selected app id={} name={} — total_selected={}",
                        entry.id, entry.name, app.app_selected_ids.len()
                    );
                }
            }
        }
        KeyCode::Esc => {
            tracing::debug!("[apps] Esc — returning focus to section panel");
            app.app_focus = AppFocus::Section;
        }
        KeyCode::Char('i') => {
            let mgr = app.active_package_manager();
            if mgr == "winget" {
                // Windows: open search panel
                tracing::info!("[apps] 'i' — opening winget search panel");
                app.app_focus = AppFocus::Search;
                app.search_query.clear();
                app.search_results.clear();
                app.search_loading = false;
            } else {
                // Linux: plain custom input
                tracing::debug!("[apps] 'i' — switching to custom input mode");
                app.app_focus = AppFocus::CustomInput;
            }
        }
        KeyCode::Char('d') => {
            tracing::info!(
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
            tracing::debug!("[custom_input] backspace — buffer={:?}", app.app_custom_input);
        }
        KeyCode::Enter => {
            let id = app.app_custom_input.trim().to_string();
            if !id.is_empty() {
                tracing::info!("[custom_input] Enter — adding custom id={:?} to selection", id);
                app.app_selected_ids.insert(id);
                app.app_custom_input.clear();
            } else {
                tracing::debug!("[custom_input] Enter with empty buffer — ignoring");
            }
            app.app_focus = AppFocus::Apps;
        }
        KeyCode::Esc => {
            tracing::debug!("[custom_input] Esc — discarding buffer={:?}", app.app_custom_input);
            app.app_custom_input.clear();
            app.app_focus = AppFocus::Apps;
        }
        KeyCode::Char(c) => {
            if c == ' ' {
                let id = app.app_custom_input.trim().to_string();
                if !id.is_empty() {
                    tracing::info!("[custom_input] Space — adding custom id={:?} to selection", id);
                    app.app_selected_ids.insert(id);
                    app.app_custom_input.clear();
                }
            } else {
                app.app_custom_input.push(c);
                tracing::debug!("[custom_input] typed {:?} — buffer={:?}", c, app.app_custom_input);
            }
        }
        _ => {}
    }
}

fn handle_installing_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char(c) => {
            app.install_input.push(c);
            tracing::debug!("[installing] typed {:?} — buffer={:?}", c, app.install_input);
        }
        KeyCode::Backspace => {
            app.install_input.pop();
            tracing::debug!("[installing] backspace — buffer={:?}", app.install_input);
        }
        KeyCode::Enter => {
            let input = app.install_input.trim().to_string();
            app.install_input.clear();
            tracing::info!("[installing] sending input to process stdin: {:?}", input);
            app.app_install_log.push(format!("> {}", input));
            if let Some(tx) = &app.install_tx {
                let _ = tx.send(input);
            } else {
                tracing::warn!("[installing] Enter pressed but install_tx is None — nothing sent");
            }
        }
        KeyCode::Esc => {
            tracing::info!(
                "[installing] Esc — user closed install modal (log_lines={})",
                app.app_install_log.len()
            );
            app.app_installing = false;
            app.app_focus = AppFocus::Section;
            app.install_rx = None;
            app.install_tx = None;
            app.install_input.clear();
            app.app_selected_ids.clear();
            app.app_install_log.clear();
        }
        _ => {}
    }
}

fn start_install(app: &mut App) {
    if app.app_selected_ids.is_empty() {
        tracing::warn!("[start_install] called with no apps selected — aborting");
        return;
    }

    let mgr = app.active_package_manager().to_string();
    tracing::info!(
        "[start_install] manager={} selected_ids={:?}",
        mgr, app.app_selected_ids
    );

    if requires_sudo(&mgr) && !is_root() {
        tracing::info!(
            "[start_install] {} requires sudo and user is not root — showing sudo confirmation",
            mgr
        );
        app.app_sudo_pending = true;
        app.app_sudo_command = build_commands(app);
        tracing::debug!("[start_install] staged sudo commands: {:?}", app.app_sudo_command);
        app.app_focus = AppFocus::SudoConfirm;
        return;
    }

    tracing::info!("[start_install] no sudo required — running directly");
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
                tracing::debug!("[build_commands] id={} → cmd={}", id, cmd);
                commands.push(cmd);
            } else {
                // May be a custom id (from search results or manual input) —
                // fall through to the generic install_command() below
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
        tracing::debug!("[build_commands] custom/search id={} → cmd={}", id, cmd);
        commands.push(cmd);
    }

    tracing::info!(
        "[build_commands] built {} command(s) for manager={}",
        commands.len(), mgr
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

    tracing::info!(
        "[execute_install] starting — use_sudo={} manager={} command_count={}",
        use_sudo, app.active_package_manager(), commands.len()
    );
    for cmd in &commands {
        tracing::debug!("[execute_install] queued: {}", cmd);
    }

    app.app_install_log.clear();
    app.app_install_log.push("Starting installation...".to_string());
    app.app_installing = true;
    app.app_focus = AppFocus::Installing;

    let (tx, rx) = mpsc::channel::<String>();
    app.install_rx = Some(rx);

    thread::spawn(move || {
        for cmd in commands {
            let _ = tx.send(format!("▶ Running: {}", cmd));
            tracing::info!("[execute_install:thread] running: {}", cmd);

            let (binary, args) = split_command(&cmd);
            if binary.is_empty() {
                tracing::warn!("[execute_install:thread] empty binary — skipping");
                continue;
            }

            match Command::new(&binary)
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(mut child) => {
                    tracing::debug!(
                        "[execute_install:thread] spawned {} pid={:?}",
                        binary, child.id()
                    );
                    if let Some(stdout) = child.stdout.take() {
                        if !drain_stdout_to_log(stdout, &tx, "execute_install") {
                            return;
                        }
                    }
                    if let Some(stderr) = child.stderr.take() {
                        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                            let line = sanitize_line(line);
                            if is_noise_line(&line) { continue; }
                            tracing::debug!("[execute_install:stderr] {}", line);
                            if tx.send(format!("[err] {}", line)).is_err() {
                                tracing::warn!("[execute_install:stderr] channel closed — stopping");
                                return;
                            }
                        }
                    }
                    match child.wait() {
                        Ok(s) if s.success() => {
                            tracing::info!("[execute_install:thread] {} exited OK", binary);
                            let _ = tx.send(format!("✓ Done: {}", binary));
                        }
                        Ok(s) => {
                            tracing::warn!("[execute_install:thread] {} exited {}", binary, s);
                            let _ = tx.send(format!("✗ Failed (exit {})", s));
                        }
                        Err(e) => {
                            tracing::error!("[execute_install:thread] wait error for {}: {}", binary, e);
                            let _ = tx.send(format!("✗ Error: {}", e));
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("[execute_install:thread] could not spawn {}: {}", binary, e);
                    let _ = tx.send(format!("✗ Could not run {}: {}", binary, e));
                }
            }
        }
        tracing::info!("[execute_install:thread] all commands finished");
        let _ = tx.send("═══ All done ═══".to_string());
    });
}

fn render_sudo_confirmation(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 30, frame.area());
    frame.render_widget(Clear, area);

    let mgr   = app.active_package_manager();
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
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
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

// ─── Public types ─────────────────────────────────────────────────────────────

/// A single result row from `winget search`.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub name:    String,
    pub id:      String,
    pub version: String,
}