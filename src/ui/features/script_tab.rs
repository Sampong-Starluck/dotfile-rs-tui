use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use crate::{
    app::App,
    service::script_service::{
        add_source_to_profile, chsh_command, clear_primary_shell, deploy_script,
        effective_primary_shell, load_shell_statuses, remove_source_from_profile,
        set_primary_shell, shell_binary, shell_profile_path, source_hint, undeploy_script,
    },
};

pub fn script_render(frame: &mut Frame, sidebar: Rect, body: Rect, app: &mut App) {
    if app.script_shells.is_none() {
        app.script_shells        = Some(load_shell_statuses());
        app.script_primary_shell = effective_primary_shell();
    }
    render_sidebar(frame, sidebar, app);
    render_body(frame, body, app);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    let count = app.script_shells.as_ref().map(|s| s.len()).unwrap_or(0);
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            if count > 0 {
                app.script_selected = (app.script_selected + 1).min(count - 1);
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.script_selected = app.script_selected.saturating_sub(1);
        }
        KeyCode::Enter => deploy_selected(app),
        KeyCode::Char('d') => undeploy_selected(app),
        KeyCode::Char('p') => set_primary_selected(app),
        KeyCode::Char('c') => clear_primary_selected(app),
        KeyCode::Char('r') => {
            app.script_shells = None;
            app.script_log.push("  Refreshed shell status.".into());
        }
        _ => {}
    }
}

// ── Actions ───────────────────────────────────────────────────────────────────

fn deploy_selected(app: &mut App) {
    let Some(shells) = app.script_shells.as_ref() else { return };
    let Some(status) = shells.get(app.script_selected) else { return };
    let id   = status.entry.id.clone();
    let name = status.entry.name.clone();

    match deploy_script(&id) {
        Ok(path) => app.script_log.push(format!("  ✓ Script → {}", path.display())),
        Err(e)   => {
            app.script_log.push(format!("  ✗ Deploy {} failed: {}", name, e));
            app.script_shells = None;
            return;
        }
    }

    match add_source_to_profile(&id) {
        Ok((true,  p)) => app.script_log.push(format!("  ✓ Added source line → {}", p.display())),
        Ok((false, p)) => app.script_log.push(format!("  — Already in {}", p.display())),
        Err(e)         => app.script_log.push(format!("  ✗ Profile update failed: {}", e)),
    }

    app.script_shells = None;
}

fn undeploy_selected(app: &mut App) {
    let Some(shells) = app.script_shells.as_ref() else { return };
    let Some(status) = shells.get(app.script_selected) else { return };
    if !status.deployed {
        app.script_log.push(format!("  — {} is not deployed.", status.entry.name));
        return;
    }
    let id   = status.entry.id.clone();
    let name = status.entry.name.clone();

    match undeploy_script(&id) {
        Ok(_)  => app.script_log.push(format!("  ✓ Removed {} script.", name)),
        Err(e) => {
            app.script_log.push(format!("  ✗ Remove {} failed: {}", name, e));
            app.script_shells = None;
            return;
        }
    }

    match remove_source_from_profile(&id) {
        Ok((true,  p)) => app.script_log.push(format!("  ✓ Removed source line from {}", p.display())),
        Ok((false, _)) => app.script_log.push("  — Source line not found in profile.".into()),
        Err(e)         => app.script_log.push(format!("  ✗ Profile cleanup failed: {}", e)),
    }

    app.script_shells = None;
}

fn set_primary_selected(app: &mut App) {
    let Some(shells) = app.script_shells.as_ref() else { return };
    let Some(status) = shells.get(app.script_selected) else { return };
    let id   = status.entry.id.clone();
    let name = status.entry.name.clone();

    match set_primary_shell(&id) {
        Ok(_) => {
            app.script_primary_shell = Some(id.clone());
            app.script_log.push(format!("  ★  {} set as primary shell.", name));
            match chsh_command(&id) {
                Some(cmd) => {
                    app.script_log.push(format!("  ▶  Running: {}", cmd));
                    app.run_external.push(cmd);
                }
                None => {
                    app.script_log.push("  — chsh not available on this platform.".into());
                }
            }
        }
        Err(e) => app.script_log.push(format!("  ✗ Set primary failed: {}", e)),
    }
}

fn clear_primary_selected(app: &mut App) {
    match clear_primary_shell() {
        Ok(_) => {
            let detected = crate::service::script_service::detect_default_shell();
            app.script_primary_shell = detected.clone();
            match detected {
                Some(id) => app.script_log.push(format!("  ◇  Primary cleared. System default: {}.", id)),
                None     => app.script_log.push("  ◇  Primary cleared. No system default detected.".into()),
            }
        }
        Err(e) => app.script_log.push(format!("  ✗ Clear primary failed: {}", e)),
    }
}

// ── Sidebar ───────────────────────────────────────────────────────────────────

fn render_sidebar(frame: &mut Frame, area: Rect, app: &App) {
    let shells = match app.script_shells.as_ref() {
        Some(s) if !s.is_empty() => s,
        Some(_) => {
            frame.render_widget(
                Paragraph::new("\n  No shells with scripts.")
                    .style(Style::default().fg(Color::DarkGray))
                    .block(rounded_block("Shells", true)),
                area,
            );
            return;
        }
        None => {
            frame.render_widget(
                Paragraph::new("\n  Loading…")
                    .style(Style::default().fg(Color::DarkGray))
                    .block(rounded_block("Shells", true)),
                area,
            );
            return;
        }
    };

    let primary = app.script_primary_shell.as_deref();
    let narrow  = area.width < 18;

    let items: Vec<ListItem> = shells
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let is_sel  = i == app.script_selected;
            let is_prim = primary.map(|p| p == s.entry.id).unwrap_or(false);
            let (status_icon, status_color) = shell_icon(s.detected, s.deployed);
            let label   = if narrow { &s.entry.id } else { &s.entry.name };

            let sel_style = if is_sel {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let arrow = if is_sel { "▶ " } else { "  " };
            let prim  = if is_prim { "★ " } else { "  " };

            ListItem::new(Line::from(vec![
                Span::styled(arrow.to_string(), sel_style),
                Span::styled(prim.to_string(), Style::default().fg(Color::Yellow)),
                Span::styled(status_icon, Style::default().fg(status_color)),
                Span::raw(" "),
                Span::styled(label.clone(), sel_style),
            ]))
        })
        .collect();

    let mut state = ListState::default().with_selected(Some(app.script_selected));
    frame.render_stateful_widget(
        List::new(items)
            .block(rounded_block("Shells", true))
            .highlight_style(Style::default()),
        area,
        &mut state,
    );
}

// ── Body ──────────────────────────────────────────────────────────────────────

fn render_body(frame: &mut Frame, area: Rect, app: &App) {
    let log_height = (area.height / 3).clamp(5, 12);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(log_height)])
        .split(area);

    render_info(frame, chunks[0], app);
    render_log(frame, chunks[1], app);
}

fn render_info(frame: &mut Frame, area: Rect, app: &App) {
    let Some(shells) = app.script_shells.as_ref() else {
        frame.render_widget(
            Paragraph::new("\n  Loading…").block(rounded_block("Script Info", false)),
            area,
        );
        return;
    };
    let Some(status) = shells.get(app.script_selected) else {
        frame.render_widget(
            Paragraph::new("\n  No shell selected.").block(rounded_block("Script Info", false)),
            area,
        );
        return;
    };

    let primary          = app.script_primary_shell.as_deref();
    let is_explicit_prim = crate::service::script_service::read_config()
        .primary_shell.as_deref() == Some(&status.entry.id);
    let is_default_prim  = !is_explicit_prim
        && primary.map(|p| p == status.entry.id).unwrap_or(false);

    let (status_icon, status_color, status_label) = if !status.detected {
        ("○", Color::DarkGray, "not installed on this system")
    } else if status.deployed {
        ("✓", Color::Green, "deployed")
    } else {
        ("◆", Color::Yellow, "detected — not deployed")
    };

    let binary      = shell_binary(&status.entry.id).unwrap_or("—");
    let path_str    = status.target_path.as_ref()
        .map(|p| p.display().to_string()).unwrap_or_else(|| "—".into());
    let profile_str = shell_profile_path(&status.entry.id)
        .map(|p| p.display().to_string()).unwrap_or_else(|| "—".into());
    let hint        = source_hint(&status.entry.id).unwrap_or_default();
    let plat_spans  = platform_badges(&status.entry.platforms);

    let primary_line = if is_explicit_prim {
        Line::from(vec![
            Span::styled("  Primary    ", Style::default().fg(Color::DarkGray)),
            Span::styled("★  set as primary", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ])
    } else if is_default_prim {
        Line::from(vec![
            Span::styled("  Primary    ", Style::default().fg(Color::DarkGray)),
            Span::styled("◇  system default ($SHELL)", Style::default().fg(Color::Cyan)),
        ])
    } else {
        Line::from(vec![
            Span::styled("  Primary    ", Style::default().fg(Color::DarkGray)),
            Span::styled("—", Style::default().fg(Color::DarkGray)),
        ])
    };

    let mut lines: Vec<Line> = vec![
        Line::from(""),
        row("  Shell      ", &status.entry.name, Color::Cyan),
        row("  Binary     ", binary, Color::Yellow),
        row("  Desc       ", &status.entry.description, Color::White),
        Line::from(vec![
            Span::styled("  Status     ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}  {}", status_icon, status_label),
                Style::default().fg(status_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        {
            let mut spans = vec![Span::styled("  Platforms  ", Style::default().fg(Color::DarkGray))];
            spans.extend(plat_spans);
            Line::from(spans)
        },
        primary_line,
        row("  Script dir ", &path_str, Color::White),
        row("  Profile    ", &profile_str, Color::White),
        Line::from(""),
    ];

    if !hint.is_empty() && status.detected {
        lines.push(Line::from(Span::styled(
            "  ── Source line (added automatically on deploy) ────────────────────────────",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(hint, Style::default().fg(Color::Yellow)),
        ]));
    }

    if !status.entry.requires.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  Requires: {}", status.entry.requires.join(", ")),
            Style::default().fg(Color::DarkGray),
        )));
    }

    frame.render_widget(
        Paragraph::new(lines)
            .block(rounded_block("Script Info", false))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_log(frame: &mut Frame, area: Rect, app: &App) {
    let visible = area.height.saturating_sub(2) as usize;
    let log     = &app.script_log;
    let start   = log.len().saturating_sub(visible);

    let items: Vec<Line> = log[start..]
        .iter()
        .map(|s| {
            let color = if s.contains('✓') { Color::Green }
                else if s.contains('✗') { Color::Red }
                else if s.contains('★') { Color::Yellow }
                else { Color::DarkGray };
            Line::from(Span::styled(s.clone(), Style::default().fg(color)))
        })
        .collect();

    frame.render_widget(
        Paragraph::new(items).block(rounded_block("Log", false)),
        area,
    );
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn rounded_block(title: &str, focused: bool) -> Block<'static> {
    let color = if focused { Color::Cyan } else { Color::DarkGray };
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color))
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))
}

fn row<'a>(label: &'a str, value: &str, color: Color) -> Line<'a> {
    Line::from(vec![
        Span::styled(label.to_string(), Style::default().fg(Color::DarkGray)),
        Span::styled(value.to_string(), Style::default().fg(color)),
    ])
}

fn shell_icon(detected: bool, deployed: bool) -> (&'static str, Color) {
    if !detected     { ("○", Color::DarkGray) }
    else if deployed { ("✓", Color::Green) }
    else             { ("◆", Color::Yellow) }
}

fn platform_badges(platforms: &[String]) -> Vec<Span<'static>> {
    let current = if cfg!(target_os = "windows") { "windows" }
        else if cfg!(target_os = "macos") { "macos" }
        else { "linux" };

    let mut spans: Vec<Span<'static>> = Vec::new();
    for (i, p) in platforms.iter().enumerate() {
        if i > 0 { spans.push(Span::raw("  ")); }
        if p.as_str() == current {
            spans.push(Span::styled(
                p.clone(),
                Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(p.clone(), Style::default().fg(Color::DarkGray)));
        }
    }
    spans
}