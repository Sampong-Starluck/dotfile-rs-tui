use crate::{
    app::App,
    enumerate::AppFocus,
    models::Apps,
    service::{
        filter_apps_by_platform, get_install_command, install_command, remove_command,
        is_root, read_apps_json, requires_interactive, requires_sudo,
        search_command, decode_search_output, parse_search_output,
        list_command, parse_list_output,
        search_hint,
        SearchResult,
    },
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use std::{
    collections::HashSet,
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    sync::{Arc, Mutex, mpsc},
    thread,
};
use crate::models::{AppEntry, PackageManager};
use crate::utils::{decode_winget_output, is_noise_line, sanitize_line, strip_ansi};

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

    // Drain search result channel
    if let Some(rx) = &app.search_rx
        && let Ok(results) = rx.try_recv()
    {
        tracing::info!("[app_render] received {} search results", results.len());
        app.search_results = results;
        app.search_loading = false;
        app.search_selected = 0;
        app.search_rx = None;
    }

    // Drain installed packages channel
    if let Some(rx) = &app.installed_rx
        && let Ok(results) = rx.try_recv()
    {
        tracing::info!("[app_render] received {} installed packages", results.len());
        app.installed_set = results.iter().map(|r| r.name.clone()).collect();
        app.installed_packages = results;
        app.installed_loading = false;
        app.installed_selected = 0;
        app.installed_rx = None;
    }

    // Auto-load installed packages in the background so the apps list can
    // show [I] markers without the user having to open the installed view first.
    if !app.installed_auto_loaded && !app.installed_loading {
        app.installed_auto_loaded = true;
        run_list_installed(app);
        tracing::info!("[app_render] triggered background installed-list load");
    }

    let body_list_offset = {
        let apps = app.apps.as_ref().unwrap();
        render_sidebar(frame, sidebar, apps, app);
        let offset = match app.app_focus {
            AppFocus::Search    => render_search_panel(frame, body_chunks[0], app),
            AppFocus::Installed => render_installed_panel(frame, body_chunks[0], app),
            _                   => render_app_list(frame, body_chunks[0], apps, app),
        };
        render_custom_input(frame, body_chunks[1], app);
        offset
        // `apps` borrow released here
    };
    app.body_list_offset = body_list_offset;

    if app.app_focus == AppFocus::PmPicker {
        render_pm_picker(frame, app);
    } else if app.app_sudo_pending {
        tracing::debug!("[app_render] rendering sudo confirmation modal");
        render_sudo_confirmation(frame, app);
    } else if app.app_installing {
        tracing::debug!(
            "[app_render] rendering install modal (log_lines={})",
            app.app_install_log.len()
        );
        render_install_modal(frame, app);
    }
}

fn render_pm_picker(frame: &mut Frame, app: &App) {
    let area = centered_rect(44, 60, frame.area());
    frame.render_widget(Clear, area);

    let inner_w = area.width.saturating_sub(2) as usize;

    let items: Vec<ListItem> = app
        .package_managers
        .iter()
        .enumerate()
        .map(|(i, pm)| {
            let is_cursor = i == app.pm_picker_selected;
            let is_active = i == app.selected_pm;

            let active_tag = if is_active { " ●" } else { "  " };
            let text = format!("{} {:10} {}", active_tag, pm.binary(), pm_description(pm));
            let padded = format!("{:<width$}", text, width = inner_w);

            let style = match (is_cursor, is_active) {
                (true, true)  => Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD),
                (true, false) => Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),
                (_, true)     => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                _             => Style::default().fg(Color::DarkGray),
            };

            ListItem::new(Line::from(Span::styled(padded, style)))
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.pm_picker_selected));

    frame.render_stateful_widget(
        List::new(items)
            .block(panel("⚙ Package Manager", true))
            .highlight_symbol(""),
        area,
        &mut list_state,
    );
}

fn render_sidebar(frame: &mut Frame, area: Rect, apps: &Apps, app: &App) {
    let focused  = app.app_focus == AppFocus::Section;
    let inner_w  = area.width.saturating_sub(2) as usize;

    let items: Vec<ListItem> = apps
        .iter()
        .enumerate()
        .map(|(i, section)| {
            let is_cur = i == app.app_selected_section;
            let style = match (is_cur, focused) {
                (true, true)  => Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),
                (true, false) => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                _             => Style::default().fg(Color::DarkGray),
            };
            let prefix = if is_cur && focused { "▶ " } else { "  " };
            let text   = format!("{}{}", prefix, section.section);
            let padded = format!("{:<width$}", text, width = inner_w);
            ListItem::new(Line::from(Span::styled(padded, style)))
        })
        .collect();

    let title = format!("≡ Sections [{}]", app.active_package_manager());
    frame.render_widget(List::new(items).block(panel(&title, focused)), area);
}

fn render_app_list(frame: &mut Frame, area: Rect, apps: &Apps, app: &App) -> usize {
    let focused = app.app_focus == AppFocus::Apps;

    let Some(section) = apps.get(app.app_selected_section) else {
        tracing::warn!(
            "[render_app_list] no section at index {} — rendering empty state",
            app.app_selected_section
        );
        frame.render_widget(
            Paragraph::new("\n  No section selected.")
                .style(Style::default().fg(Color::DarkGray))
                .block(panel(&format!("≡ Apps [{}]", app.active_package_manager()), false)),
            area,
        );
        return 0;
    };

    let inner_w = area.width.saturating_sub(2) as usize;

    let mgr = app.active_package_manager();

    let items: Vec<ListItem> = section
        .apps
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_cursor    = focused && i == app.app_selected_app;
            let is_selected  = app.app_selected_ids.contains(&entry.id);
            let is_installed = entry_installed(entry, mgr, &app.installed_set);

            let checkbox = if is_selected  { "[✓]" }
                           else if is_installed { "[I]" }
                           else { "[ ]" };
            let arrow = if is_cursor { "▶ " } else { "  " };

            let style = match (is_cursor, is_selected, is_installed) {
                (true, true, _)   => Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD),
                (true, _, _)      => Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),
                (_, true, _)      => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                (_, _, true)      => Style::default().fg(Color::Green),
                _                 => Style::default().fg(Color::DarkGray),
            };

            let prefix     = format!("{}{} ", arrow, checkbox);
            let prefix_len = prefix.chars().count();
            let name_w     = inner_w.saturating_sub(prefix_len);
            let line       = format!("{}{:<name_w$}", prefix, truncate(&entry.name, name_w), name_w = name_w);

            ListItem::new(Line::from(Span::styled(line, style)))
        })
        .collect();

    let n_sel  = app.app_selected_ids.len();
    let sel_badge = if n_sel > 0 { format!("  ✓ {}  ", n_sel) } else { String::new() };
    let title  = format!("≡ {} — {}{}", section.section, app.active_package_manager(), sel_badge);

    let mut list_state = ListState::default();
    list_state.select(Some(app.app_selected_app));

    frame.render_stateful_widget(
        List::new(items).block(panel(&title, focused)).highlight_symbol(""),
        area,
        &mut list_state,
    );
    list_state.offset()
}

// ─── Search panel (winget only) ──────────────────────────────────────────────

const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

fn spinner(tick: u8) -> &'static str {
    SPINNER[tick as usize % SPINNER.len()]
}

/// Returns true if the entry's resolved package name is in the installed set.
fn entry_installed(entry: &AppEntry, mgr: &str, installed_set: &HashSet<String>) -> bool {
    let os = if cfg!(target_os = "windows") { "windows" }
             else if cfg!(target_os = "macos")   { "macos" }
             else { "linux" };
    // Use the exact package name the manager would install/list.
    entry.platforms
        .get(os)
        .and_then(|p| p.get(mgr))
        .map(|pkg| installed_set.contains(pkg.as_str()))
        // Fallback: match by entry id (e.g. for winget IDs).
        .unwrap_or_else(|| installed_set.contains(entry.id.as_str()))
}

/// Rounded panel block with a consistent title style.
fn panel(title: &str, focused: bool) -> Block<'static> {
    let border_color = if focused { Color::Cyan } else { Color::DarkGray };
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            format!(" {} ", title),
            Style::default()
                .fg(if focused { Color::Cyan } else { Color::DarkGray })
                .add_modifier(Modifier::BOLD),
        ))
}

fn render_search_panel(frame: &mut Frame, area: Rect, app: &App) -> usize {
    let mgr  = app.active_package_manager();
    let spin = spinner(app.loading_tick);

    let title = if app.search_loading {
        format!(" {} Search [{}] — searching… ", spin, mgr)
    } else if app.search_results.is_empty() && !app.search_query.is_empty() {
        format!(" Search [{}] — no results ", mgr)
    } else {
        format!(" Search [{}] — {} result(s) ", mgr, app.search_results.len())
    };

    if app.search_loading {
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    format!("  {} Searching, please wait…", spin),
                    Style::default().fg(Color::Yellow),
                )),
            ])
            .block(panel(&title, true)
                .border_style(Style::default().fg(Color::Yellow))),
            area,
        );
        return 0;
    }

    if app.search_results.is_empty() {
        let msg = if app.search_query.is_empty() {
            "  Type a query below and press Enter to search."
        } else {
            "  No results found — try a different query."
        };
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(msg, Style::default().fg(Color::DarkGray))),
            ])
            .block(panel(&title, false)),
            area,
        );
        return 0;
    }

    // prefix = arrow(2) + checkbox(3) + space(1) = 6 chars
    // columns fill remaining space with 2-char gaps between them
    let inner_w   = area.width.saturating_sub(2) as usize;
    let available = inner_w.saturating_sub(6 + 4); // 6 prefix + 4 gaps (2×"  ")
    let id_w      = (available as f32 * 0.40) as usize;
    let name_w    = (available as f32 * 0.40) as usize;
    let ver_w     = available.saturating_sub(id_w + name_w);

    let items: Vec<ListItem> = app
        .search_results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let is_cursor    = i == app.search_selected;
            let is_picked    = app.app_selected_ids.contains(&result.id);
            let is_installed = app.installed_set.contains(&result.name)
                || app.installed_set.contains(&result.id);

            let checkbox = if is_picked { "[✓]" } else if is_installed { "[I]" } else { "[ ]" };
            let arrow    = if is_cursor { "▶ " } else { "  " };

            let id   = truncate(&result.id,      id_w);
            let name = truncate(&result.name,    name_w);
            let ver  = truncate(&result.version, ver_w);

            let bg = if is_cursor && is_picked { Color::Green }
                     else if is_cursor         { Color::Cyan  }
                     else                      { Color::Reset };

            let base_style = match (is_cursor, is_picked) {
                (true, true)  => Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD),
                (true, false) => Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),
                (false, true) => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                (false, false) => Style::default().fg(Color::DarkGray),
            };

            // Each column span carries the same bg so the highlight fills the full row.
            let id_style   = Style::default().fg(if is_cursor { Color::Black } else { Color::Yellow }).bg(bg);
            let name_style = base_style;
            let ver_style  = Style::default().fg(if is_cursor { Color::Black } else { Color::DarkGray }).bg(bg);

            ListItem::new(Line::from(vec![
                Span::styled(format!("{}{} ", arrow, checkbox), base_style),
                Span::styled(format!("{:<id_w$}  ",   id,   id_w   = id_w),   id_style),
                Span::styled(format!("{:<name_w$}  ", name, name_w = name_w), name_style),
                Span::styled(format!("{:<ver_w$}",    ver,  ver_w  = ver_w),  ver_style),
            ]))
        })
        .collect();

    // let list = List::new(items).block(
    //     Block::default()
    //         .borders(Borders::ALL)
    //         .title(title)
    //         .border_style(Style::default().fg(Color::Cyan)),
    // );
    //
    // frame.render_widget(list, area);

    let mut list_state = ListState::default();
    list_state.select(Some(app.search_selected));

    frame.render_stateful_widget(
        List::new(items).block(panel(&title, true)).highlight_symbol(""),
        area,
        &mut list_state,
    );
    list_state.offset()
}

fn render_installed_panel(frame: &mut Frame, area: Rect, app: &App) -> usize {
    let mgr  = app.active_package_manager();
    let spin = spinner(app.loading_tick);

    let title = if app.installed_loading {
        format!(" {} Installed [{}] — loading… ", spin, mgr)
    } else {
        format!(" Installed [{}] — {} package(s) ", mgr, app.installed_packages.len())
    };

    if app.installed_loading {
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    format!("  {} Loading installed packages…", spin),
                    Style::default().fg(Color::Yellow),
                )),
            ])
            .block(panel(&title, true).border_style(Style::default().fg(Color::Yellow))),
            area,
        );
        return 0;
    }

    if app.installed_packages.is_empty() {
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  No installed packages found.  Press r to refresh.",
                    Style::default().fg(Color::DarkGray),
                )),
            ])
            .block(panel(&title, false)),
            area,
        );
        return 0;
    }

    let inner_w = area.width.saturating_sub(2) as usize;
    let ver_w   = 16usize;
    let name_w  = inner_w.saturating_sub(ver_w + 8);

    let items: Vec<ListItem> = app
        .installed_packages
        .iter()
        .enumerate()
        .map(|(i, pkg)| {
            let is_cursor  = i == app.installed_selected;
            let is_picked  = app.app_selected_ids.contains(&pkg.id);

            let checkbox = if is_picked { "[✓]" } else { "[ ]" };
            let arrow    = if is_cursor { "▶ " } else { "  " };

            let style = match (is_cursor, is_picked) {
                (true, true)  => Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD),
                (true, false) => Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),
                (false, true) => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                (false, false) => Style::default().fg(Color::DarkGray),
            };

            let name = truncate(&pkg.name, name_w);
            let ver  = truncate(&pkg.version, ver_w);

            // Pad to inner_w so the background fills the full row width.
            // name_w + ver_w + 8 == inner_w by construction.
            let line = format!("{}{} {:<name_w$}  {:<ver_w$}", arrow, checkbox, name, ver,
                               name_w = name_w, ver_w = ver_w);

            ListItem::new(Line::from(Span::styled(line, style)))
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.installed_selected));

    frame.render_stateful_widget(
        List::new(items)
            .block(panel(&title, true))
            .highlight_symbol(""),
        area,
        &mut list_state,
    );
    list_state.offset()
}

fn truncate(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!(
            "{}…",
            &s.chars().take(max.saturating_sub(1)).collect::<String>()
        )
    }
}

fn render_custom_input(frame: &mut Frame, area: Rect, app: &App) {
    let in_search = app.app_focus == AppFocus::Search;
    let focused   = app.app_focus == AppFocus::CustomInput || in_search;

    let (title, content, accent) = if in_search {
        let buf = &app.search_query;
        let display = if buf.is_empty() {
            Span::styled("▌", Style::default().fg(Color::Cyan))
        } else {
            Span::styled(format!("{}▌", buf), Style::default().fg(Color::White))
        };
        let search_title = if !buf.is_empty() && !app.search_loading {
            "⌕ Search  ↵ to search"
        } else {
            "⌕ Search"
        };
        (search_title, display, Color::Cyan)
    } else if focused {
        let buf = &app.app_custom_input;
        let display = if buf.is_empty() {
            Span::styled("▌", Style::default().fg(Color::Cyan))
        } else {
            Span::styled(format!("{}▌", buf), Style::default().fg(Color::White))
        };
        ("⌕ Custom package", display, Color::Cyan)
    } else {
        let hint = search_hint(app.active_package_manager());
        let display = Span::styled(hint.to_string(), Style::default().fg(Color::DarkGray));
        ("⌕ Search / Custom", display, Color::DarkGray)
    };

    frame.render_widget(
        Paragraph::new(Line::from(content)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(accent))
                .title(Span::styled(
                    format!(" {} ", title),
                    Style::default().fg(accent).add_modifier(Modifier::BOLD),
                )),
        ),
        area,
    );
}

fn render_install_modal(frame: &mut Frame, app: &App) {
    let area = centered_rect(72, 72, frame.area());
    frame.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(3)])
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
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            Line::from(Span::styled(l.clone(), style))
        })
        .collect();

    let scroll_offset = (log_text.len() as u16).saturating_sub(chunks[0].height.saturating_sub(2));

    frame.render_widget(
        Paragraph::new(log_text)
            .scroll((scroll_offset, 0))
            .wrap(Wrap { trim: false })
            .block(panel("⬇ Installing", true).border_style(Style::default().fg(Color::Yellow))),
        chunks[0],
    );

    let cursor = if app.install_input.is_empty() {
        Span::styled("▌", Style::default().fg(Color::DarkGray))
    } else {
        Span::styled(format!("{}▌", app.install_input), Style::default().fg(Color::White))
    };

    frame.render_widget(
        Paragraph::new(Line::from(cursor))
            .block(panel("Response  y / n / Enter", true)),
        chunks[1],
    );
}

use super::super::centered_rect;

// ─── Key handler ─────────────────────────────────────────────────────────────

pub fn handle_key(app: &mut App, key: KeyEvent) {
    tracing::debug!("[handle_key] focus={:?} key={:?}", app.app_focus, key.code);
    match app.app_focus {
        AppFocus::Section     => handle_section_keys(app, key),
        AppFocus::Apps        => handle_apps_keys(app, key),
        AppFocus::CustomInput => handle_custom_input_keys(app, key),
        AppFocus::Search      => handle_search_keys(app, key),
        AppFocus::Installed   => handle_installed_keys(app, key),
        AppFocus::Installing  => handle_installing_keys(app, key),
        AppFocus::SudoConfirm => handle_sudo_confirm_keys(app, key),
        AppFocus::PmPicker    => handle_pm_picker_keys(app, key),
    }
}

fn handle_pm_picker_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.pm_picker_selected > 0 {
                app.pm_picker_selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.pm_picker_selected < app.package_managers.len().saturating_sub(1) {
                app.pm_picker_selected += 1;
            }
        }
        KeyCode::Enter => {
            if app.pm_picker_selected != app.selected_pm {
                app.selected_pm           = app.pm_picker_selected;
                app.apps                  = None;
                app.app_selected_section  = 0;
                app.app_selected_app      = 0;
                app.app_selected_ids.clear();
                app.search_query.clear();
                app.search_results.clear();
                app.search_loading        = false;
                app.search_rx             = None;
                app.installed_packages.clear();
                app.installed_set.clear();
                app.installed_loading      = false;
                app.installed_rx           = None;
                app.installed_selected     = 0;
                app.app_remove_mode        = false;
                app.installed_auto_loaded  = false;
                tracing::info!(
                    "[pm_picker] switched to manager={}",
                    app.active_package_manager()
                );
            }
            app.app_focus = AppFocus::Section;
        }
        KeyCode::Esc => {
            // discard — no change
            app.app_focus = AppFocus::Section;
        }
        _ => {}
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
                        result.id,
                        result.name,
                        app.app_selected_ids.len()
                    );
                } else {
                    app.app_selected_ids.insert(result.id.clone());
                    tracing::info!(
                        "[search] selected id={} name={} — total_selected={}",
                        result.id,
                        result.name,
                        app.app_selected_ids.len()
                    );
                }
            }
        }

        // Re-run search with current query
        KeyCode::Enter => {
            if !app.search_query.is_empty() && !app.search_loading {
                tracing::info!("[search] re-running search for {:?}", app.search_query);
                run_search(app);
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
            tracing::info!(
                "[search] closed — returning to {:?}",
                app.search_origin
            );
            app.app_focus = app.search_origin;  // ← was hardcoded AppFocus::Apps
            app.search_results.clear();
            app.search_query.clear();
            app.search_loading = false;
            app.search_rx = None;
        }

        _ => {}
    }
}

// ─── Winget search runner ─────────────────────────────────────────────────────

fn run_search(app: &mut App) {
    let query = app.search_query.clone();
    let mgr   = app.active_package_manager().to_string();
    tracing::info!("[run_search] launching: {} search {}", mgr, query);

    app.search_loading = true;
    app.search_results.clear();

    let (tx, rx) = mpsc::channel::<Vec<SearchResult>>();
    app.search_rx = Some(rx);

    thread::spawn(move || {
        let (cmd, args) = search_command(&mgr, &query);
        match Command::new(&cmd).args(&args).stdout(Stdio::piped()).stderr(Stdio::null()).output() {
            Ok(out) => {
                let text    = decode_search_output(&mgr, &out.stdout);
                let results = parse_search_output(&mgr, &text);
                tracing::info!("[run_search:thread] {} results for {:?} via {}", results.len(), query, mgr);
                let _ = tx.send(results);
            }
            Err(e) => {
                tracing::error!("[run_search:thread] failed to run {}: {}", cmd, e);
                let _ = tx.send(vec![]);
            }
        }
    });
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
                        binary,
                        child.id()
                    );
                    if let Some(mut stdin) = child.stdin.take() {
                        use std::io::Write;
                        if let Some(ref pwd) = password_clone {
                            tracing::debug!(
                                "[install_with_password:thread] writing password to stdin"
                            );
                            let _ = writeln!(stdin, "{}", pwd);
                        }
                        let in_rx_clone = Arc::clone(&in_rx);
                        thread::spawn(move || {
                            tracing::debug!(
                                "[install_with_password:stdin_relay] waiting for user input"
                            );
                            while let Ok(input) = in_rx_clone.lock().unwrap().recv() {
                                tracing::debug!(
                                    "[install_with_password:stdin_relay] forwarding: {:?}",
                                    input
                                );
                                let _ = writeln!(stdin, "{}", input);
                            }
                            tracing::debug!(
                                "[install_with_password:stdin_relay] channel closed — exiting"
                            );
                        });
                    }

                    if let Some(stdout) = child.stdout.take()
                        && !drain_stdout_to_log(stdout, &out_tx, "install_with_password") {
                        return;
                    }

                    if let Some(stderr) = child.stderr.take() {
                        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                            let line = sanitize_line(line);
                            if is_noise_line(&line) {
                                continue;
                            }
                            if !line.contains("[sudo]") && !line.contains("password for") {
                                tracing::debug!("[install_with_password:stderr] {}", line);
                                if out_tx.send(format!("[err] {}", line)).is_err() {
                                    tracing::warn!(
                                        "[install_with_password:stderr] channel closed — stopping"
                                    );
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
                            tracing::warn!(
                                "[install_with_password:thread] {} exited {}",
                                binary,
                                s
                            );
                            let _ = out_tx.send(format!("✗ Failed (exit {})", s));
                        }
                        Err(e) => {
                            tracing::error!(
                                "[install_with_password:thread] wait error for {}: {}",
                                binary,
                                e
                            );
                            let _ = out_tx.send(format!("✗ Error: {}", e));
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "[install_with_password:thread] could not spawn {}: {}",
                        binary,
                        e
                    );
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
                        binary,
                        child.id()
                    );
                    if let Some(mut stdin) = child.stdin.take() {
                        use std::io::Write;
                        if let Some(ref pwd) = password_clone {
                            tracing::debug!(
                                "[install_interactive:thread] writing password to stdin"
                            );
                            let _ = writeln!(stdin, "{}", pwd);
                        }
                        let in_rx_clone = Arc::clone(&in_rx);
                        thread::spawn(move || {
                            tracing::debug!(
                                "[install_interactive:stdin_relay] waiting for user input"
                            );
                            while let Ok(input) = in_rx_clone.lock().unwrap().recv() {
                                tracing::debug!(
                                    "[install_interactive:stdin_relay] forwarding: {:?}",
                                    input
                                );
                                let _ = writeln!(stdin, "{}", input);
                            }
                            tracing::debug!(
                                "[install_interactive:stdin_relay] channel closed — exiting"
                            );
                        });
                    }

                    if let Some(stdout) = child.stdout.take()
                        && !drain_stdout_to_log(stdout, &out_tx, "install_interactive") {
                            return;
                    }

                    if let Some(stderr) = child.stderr.take() {
                        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                            let line = sanitize_line(line);
                            if is_noise_line(&line) {
                                continue;
                            }
                            if !line.contains("[sudo]") && !line.contains("password for") {
                                tracing::debug!("[install_interactive:stderr] {}", line);
                                if out_tx.send(format!("[err] {}", line)).is_err() {
                                    tracing::warn!(
                                        "[install_interactive:stderr] channel closed — stopping"
                                    );
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
                            tracing::error!(
                                "[install_interactive:thread] wait error for {}: {}",
                                binary,
                                e
                            );
                            let _ = out_tx.send(format!("✗ Error: {}", e));
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "[install_interactive:thread] could not spawn {}: {}",
                        binary,
                        e
                    );
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
                    app.app_selected_section,
                    app.app_selected_app
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
                    app.app_selected_section,
                    app.app_selected_app
                );
            } else {
                tracing::debug!("[section] navigate down — already at bottom");
            }
        }
        KeyCode::Char(' ') | KeyCode::Enter => {
            tracing::info!(
                "[section] enter apps list — section={}",
                app.app_selected_section
            );
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
        KeyCode::Char('i') => {
            app.search_origin = AppFocus::Section;
            app.app_focus = AppFocus::Search;
            app.search_query.clear();
            app.search_results.clear();
            app.search_loading = false;
            tracing::info!("[section] 'i' — opening search for mgr={}", app.active_package_manager());
        }
        KeyCode::Char('l') => {
            app.search_origin = AppFocus::Section;
            app.app_focus = AppFocus::Installed;
            app.app_selected_ids.clear();
            app.app_remove_mode = true;
            run_list_installed(app);
            tracing::info!("[section] 'l' — opening installed view for mgr={}", app.active_package_manager());
        }
        KeyCode::Char('p') => {
            app.pm_picker_selected = app.selected_pm;
            app.app_focus = AppFocus::PmPicker;
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
                    app.app_selected_section,
                    app.app_selected_app
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
                    app.app_selected_section,
                    app.app_selected_app
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
                        entry.id,
                        entry.name,
                        app.app_selected_ids.len()
                    );
                } else {
                    app.app_selected_ids.insert(entry.id.clone());
                    tracing::info!(
                        "[apps] selected app id={} name={} — total_selected={}",
                        entry.id,
                        entry.name,
                        app.app_selected_ids.len()
                    );
                }
            }
        }
        KeyCode::Esc => {
            tracing::debug!("[apps] Esc — returning focus to section panel");
            app.app_focus = AppFocus::Section;
        }
        KeyCode::Char('i') => {
            app.search_origin = AppFocus::Apps;
            app.app_focus = AppFocus::Search;
            app.search_query.clear();
            app.search_results.clear();
            app.search_loading = false;
            tracing::info!("[apps] 'i' — opening search for mgr={}", app.active_package_manager());
        }
        KeyCode::Char('l') => {
            app.search_origin = AppFocus::Apps;
            app.app_focus = AppFocus::Installed;
            app.app_selected_ids.clear();
            app.app_remove_mode = true;
            run_list_installed(app);
            tracing::info!("[apps] 'l' — opening installed view for mgr={}", app.active_package_manager());
        }
        KeyCode::Char('d') => {
            tracing::info!(
                "[apps] install triggered — selected_count={} ids={:?}",
                app.app_selected_ids.len(),
                app.app_selected_ids
            );
            start_install(app);
        }
        KeyCode::Char('p') => {
            app.pm_picker_selected = app.selected_pm;
            app.app_focus = AppFocus::PmPicker;
        }
        _ => {}
    }
}

fn handle_custom_input_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Backspace => {
            app.app_custom_input.pop();
            tracing::debug!(
                "[custom_input] backspace — buffer={:?}",
                app.app_custom_input
            );
        }
        KeyCode::Enter => {
            let id = app.app_custom_input.trim().to_string();
            if !id.is_empty() {
                tracing::info!(
                    "[custom_input] Enter — adding custom id={:?} to selection",
                    id
                );
                app.app_selected_ids.insert(id);
                app.app_custom_input.clear();
            } else {
                tracing::debug!("[custom_input] Enter with empty buffer — ignoring");
            }
            app.app_focus = app.search_origin;
        }
        KeyCode::Esc => {
            tracing::debug!(
                "[custom_input] Esc — returning to {:?}, discarding buffer={:?}",
                app.search_origin, app.app_custom_input
            );
            app.app_custom_input.clear();
            app.app_focus = app.search_origin;  // ← was hardcoded AppFocus::Apps
        }
        KeyCode::Char(c) => {
            if c == ' ' {
                let id = app.app_custom_input.trim().to_string();
                if !id.is_empty() {
                    tracing::info!(
                        "[custom_input] Space — adding custom id={:?} to selection",
                        id
                    );
                    app.app_selected_ids.insert(id);
                    app.app_custom_input.clear();
                }
            } else {
                app.app_custom_input.push(c);
                tracing::debug!(
                    "[custom_input] typed {:?} — buffer={:?}",
                    c,
                    app.app_custom_input
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
            tracing::debug!(
                "[installing] typed {:?} — buffer={:?}",
                c,
                app.install_input
            );
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
                "[installing] Esc — user closed modal (log_lines={})",
                app.app_install_log.len()
            );
            app.app_installing = false;
            app.install_rx = None;
            app.install_tx = None;
            app.install_input.clear();
            app.app_selected_ids.clear();
            app.app_install_log.clear();
            if app.app_remove_mode {
                // go back to installed view and refresh
                app.app_focus = AppFocus::Installed;
                app.installed_packages.clear();
                app.installed_set.clear();
                run_list_installed(app);
            } else {
                app.app_focus = AppFocus::Section;
            }
        }
        _ => {}
    }
}

// ─── Installed packages key handler ──────────────────────────────────────────

fn handle_installed_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.installed_selected > 0 {
                app.installed_selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.installed_selected < app.installed_packages.len().saturating_sub(1) {
                app.installed_selected += 1;
            }
        }
        KeyCode::Char(' ') => {
            if let Some(pkg) = app.installed_packages.get(app.installed_selected) {
                let id = pkg.id.clone();
                if app.app_selected_ids.contains(&id) {
                    app.app_selected_ids.remove(&id);
                } else {
                    app.app_selected_ids.insert(id);
                }
            }
        }
        KeyCode::Char('d') => {
            tracing::info!(
                "[installed] remove triggered — selected_count={}",
                app.app_selected_ids.len()
            );
            start_remove(app);
        }
        KeyCode::Char('r') => {
            app.installed_packages.clear();
            app.installed_set.clear();
            app.app_selected_ids.clear();
            run_list_installed(app);
            tracing::info!("[installed] refreshing installed list");
        }
        KeyCode::Esc => {
            app.app_focus = app.search_origin;
            app.app_remove_mode = false;
            app.app_selected_ids.clear();
            tracing::info!("[installed] closed — returning to {:?}", app.search_origin);
        }
        _ => {}
    }
}

fn run_list_installed(app: &mut App) {
    let mgr = app.active_package_manager().to_string();
    tracing::info!("[run_list_installed] querying installed packages via {}", mgr);

    app.installed_loading = true;
    app.installed_packages.clear();
    app.installed_rx = None;

    let (tx, rx) = mpsc::channel::<Vec<SearchResult>>();
    app.installed_rx = Some(rx);

    thread::spawn(move || {
        let (cmd, args) = list_command(&mgr);
        match Command::new(&cmd).args(&args).stdout(Stdio::piped()).stderr(Stdio::null()).output() {
            Ok(out) => {
                let text    = String::from_utf8_lossy(&out.stdout).into_owned();
                let results = parse_list_output(&mgr, &text);
                tracing::info!("[run_list_installed:thread] {} packages found via {}", results.len(), mgr);
                let _ = tx.send(results);
            }
            Err(e) => {
                tracing::error!("[run_list_installed:thread] failed to run {}: {}", cmd, e);
                let _ = tx.send(vec![]);
            }
        }
    });
}

fn start_remove(app: &mut App) {
    if app.app_selected_ids.is_empty() {
        tracing::warn!("[start_remove] called with no packages selected — aborting");
        return;
    }

    let mgr = app.active_package_manager().to_string();
    tracing::info!(
        "[start_remove] manager={} selected_ids={:?}",
        mgr,
        app.app_selected_ids
    );

    if requires_interactive(&mgr) {
        let commands = build_remove_commands(app);
        let ext_cmds = if requires_sudo(&mgr) && !is_root() {
            commands.into_iter().map(|c| format!("sudo {}", c)).collect()
        } else {
            commands
        };
        tracing::info!(
            "[start_remove] {} is interactive — queuing external execution: {:?}",
            mgr, ext_cmds
        );
        app.run_external          = ext_cmds;
        app.run_external_removing = true;
        return;
    }

    if requires_sudo(&mgr) && !is_root() {
        app.app_sudo_pending = true;
        app.app_sudo_command = build_remove_commands(app);
        app.app_focus        = AppFocus::SudoConfirm;
        return;
    }

    execute_remove(app, false);
}

fn build_remove_commands(app: &App) -> Vec<String> {
    let mgr = app.active_package_manager();
    app.app_selected_ids
        .iter()
        .map(|id| remove_command(mgr, id))
        .collect()
}

fn execute_remove(app: &mut App, use_sudo: bool) {
    let commands = if use_sudo {
        app.app_sudo_command
            .drain(..)
            .map(|cmd| format!("sudo {}", cmd))
            .collect::<Vec<_>>()
    } else {
        build_remove_commands(app)
    };

    tracing::info!(
        "[execute_remove] starting — use_sudo={} manager={} command_count={}",
        use_sudo,
        app.active_package_manager(),
        commands.len()
    );

    app.app_install_log.clear();
    app.app_install_log.push("Starting removal…".to_string());
    if requires_interactive(app.active_package_manager()) {
        app.app_install_log.push("Type y/n and press Enter to respond to prompts.".to_string());
    }
    app.app_installing = true;
    app.app_focus = AppFocus::Installing;

    let (out_tx, out_rx) = mpsc::channel::<String>();
    let (in_tx, in_rx)   = mpsc::channel::<String>();

    app.install_rx = Some(out_rx);
    app.install_tx = Some(in_tx);

    thread::spawn(move || {
        let in_rx = Arc::new(Mutex::new(in_rx));

        for cmd in commands {
            let _ = out_tx.send(format!("▶ Running: {}", cmd));
            tracing::info!("[execute_remove:thread] running: {}", cmd);

            let (binary, args) = split_command(&cmd);
            if binary.is_empty() { continue; }

            match Command::new(&binary)
                .args(&args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(mut child) => {
                    if let Some(mut stdin) = child.stdin.take() {
                        use std::io::Write;
                        let in_rx_clone = Arc::clone(&in_rx);
                        thread::spawn(move || {
                            while let Ok(input) = in_rx_clone.lock().unwrap().recv() {
                                let _ = writeln!(stdin, "{}", input);
                            }
                        });
                    }

                    if let Some(stdout) = child.stdout.take()
                        && !drain_stdout_to_log(stdout, &out_tx, "execute_remove") {
                            return;
                    }

                    if let Some(stderr) = child.stderr.take() {
                        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                            let line = sanitize_line(line);
                            if is_noise_line(&line) { continue; }
                            if out_tx.send(format!("[err] {}", line)).is_err() { return; }
                        }
                    }

                    match child.wait() {
                        Ok(s) if s.success() => { let _ = out_tx.send(format!("✓ Done: {}", binary)); }
                        Ok(s)                => { let _ = out_tx.send(format!("✗ Failed (exit {})", s)); }
                        Err(e)               => { let _ = out_tx.send(format!("✗ Error: {}", e)); }
                    }
                }
                Err(e) => { let _ = out_tx.send(format!("✗ Could not run {}: {}", binary, e)); }
            }
        }
        tracing::info!("[execute_remove:thread] all commands finished");
        let _ = out_tx.send("═══ All done ═══".to_string());
    });
}

fn start_install(app: &mut App) {
    if app.app_selected_ids.is_empty() {
        tracing::warn!("[start_install] called with no apps selected — aborting");
        return;
    }

    let mgr = app.active_package_manager().to_string();
    tracing::info!(
        "[start_install] manager={} selected_ids={:?}",
        mgr,
        app.app_selected_ids
    );

    if requires_interactive(&mgr) {
        // Interactive managers (pacman, apt, dnf, yay, paru, choco, …) write
        // prompts directly to the controlling terminal (/dev/tty on Unix,
        // ConPTY on Windows). Piped stdio cannot capture or forward those
        // prompts. Solution: leave the TUI, run in the real terminal, return.
        let commands = build_commands(app);
        let ext_cmds = if requires_sudo(&mgr) && !is_root() {
            commands.into_iter().map(|c| format!("sudo {}", c)).collect()
        } else {
            commands
        };
        tracing::info!(
            "[start_install] {} is interactive — queuing external execution: {:?}",
            mgr, ext_cmds
        );
        app.run_external          = ext_cmds;
        app.run_external_removing = false;
        return;
    }

    // Non-interactive managers (winget, brew, scoop): use the in-TUI modal.
    if requires_sudo(&mgr) && !is_root() {
        app.app_sudo_pending = true;
        app.app_sudo_command = build_commands(app);
        app.app_focus        = AppFocus::SudoConfirm;
        return;
    }

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

    tracing::info!(
        "[execute_install] starting — use_sudo={} manager={} command_count={}",
        use_sudo,
        app.active_package_manager(),
        commands.len()
    );
    for cmd in &commands {
        tracing::debug!("[execute_install] queued: {}", cmd);
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
                        binary,
                        child.id()
                    );
                    if let Some(stdout) = child.stdout.take()
                        && !drain_stdout_to_log(stdout, &tx, "execute_install") {
                            return;
                    }
                    if let Some(stderr) = child.stderr.take() {
                        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                            let line = sanitize_line(line);
                            if is_noise_line(&line) {
                                continue;
                            }
                            tracing::debug!("[execute_install:stderr] {}", line);
                            if tx.send(format!("[err] {}", line)).is_err() {
                                tracing::warn!(
                                    "[execute_install:stderr] channel closed — stopping"
                                );
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
                            tracing::error!(
                                "[execute_install:thread] wait error for {}: {}",
                                binary,
                                e
                            );
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

fn selected_display_names(app: &App) -> Vec<String> {
    app.app_selected_ids
        .iter()
        .map(|id| {
            if let Some(apps) = &app.apps {
                for section in apps {
                    if let Some(entry) = section.apps.iter().find(|e| &e.id == id) {
                        return entry.name.clone();
                    }
                }
            }
            id.clone()
        })
        .collect()
}

fn render_sudo_confirmation(frame: &mut Frame, app: &App) {
    let area = centered_rect(56, 50, frame.area());
    frame.render_widget(Clear, area);

    let mgr   = app.active_package_manager();
    let names = selected_display_names(app);
    let count = names.len();

    const MAX_SHOWN: usize = 10;
    let overflow = count.saturating_sub(MAX_SHOWN);

    let mut text = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  ⚠  {} requires sudo to continue.", mgr),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {} package(s) selected:", count),
            Style::default().fg(Color::DarkGray),
        )),
    ];

    for name in names.iter().take(MAX_SHOWN) {
        text.push(Line::from(Span::styled(
            format!("    • {}", name),
            Style::default().fg(Color::White),
        )));
    }

    if overflow > 0 {
        text.push(Line::from(Span::styled(
            format!("    … and {} more", overflow),
            Style::default().fg(Color::DarkGray),
        )));
    }

    text.push(Line::from(""));
    text.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(" y ", Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled("  Run with sudo      ", Style::default().fg(Color::White)),
        Span::styled(" n ", Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::styled("  Cancel", Style::default().fg(Color::White)),
    ]));

    frame.render_widget(
        Paragraph::new(text)
            .block(panel("⚠ Sudo Required", true).border_style(Style::default().fg(Color::Yellow))),
        area,
    );
}

// helper:
fn pm_description(pm: &PackageManager) -> &'static str {
    match pm {
        PackageManager::Winget => "Windows built-in, largest catalog",
        PackageManager::Scoop  => "portable installs, no admin needed",
        PackageManager::Choco  => "traditional installs, wide support",
        PackageManager::Apt    => "Debian/Ubuntu",
        PackageManager::Dnf    => "Red Hat/Fedora",
        PackageManager::Pacman => "Arch Linux default package manager",
        PackageManager::Yay    => "Yay (Yet Another Yogurt) AUR helper",
        PackageManager::Paru   => "Paru — feature-rich AUR helper",
        PackageManager::Xbps   => "Void Linux",
        _                      => "",
    }
}
