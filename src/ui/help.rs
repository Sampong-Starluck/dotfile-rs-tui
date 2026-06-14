use ratatui::{
    Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};
use crate::app::App;
use crate::enumerate::AppFocus;
use crate::models::TabModel;
use super::{bindings_for, centered_rect};

pub fn render_help(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 70, frame.area());
    frame.render_widget(Clear, area);

    let mut lines: Vec<Line> = vec![Line::from("")];

    // Global bindings always shown
    lines.push(section_header("Global"));
    for (key, desc) in &[
        ("Tab / Shift+Tab", "switch tab"),
        ("q", "quit"),
        ("?", "close help"),
    ] {
        lines.push(binding_row(key, desc));
    }

    lines.push(Line::from(""));

    // Per-tab bindings
    match app.active_tab {
        TabModel::Home => {
            lines.push(section_header("Home"));
            for (k, d) in bindings_for(TabModel::Home, AppFocus::Section) {
                if !matches!(k, "Tab" | "q" | "?") {
                    lines.push(binding_row(k, d));
                }
            }
        }
        TabModel::Application => {
            let focus_groups: &[(&str, AppFocus)] = &[
                ("Apps — Section", AppFocus::Section),
                ("Apps — List",    AppFocus::Apps),
                ("Apps — Search",  AppFocus::Search),
                ("Apps — Installed", AppFocus::Installed),
            ];

            for (label, focus) in focus_groups {
                let pairs = bindings_for(TabModel::Application, *focus);
                if pairs.is_empty() { continue; }
                lines.push(section_header(label));
                for (k, d) in &pairs {
                    lines.push(binding_row(k, d));
                }
                lines.push(Line::from(""));
            }
        }
    }

    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(Span::styled(
                        "  ? Keybindings  ",
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    )),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn section_header(label: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("  ── {} ─────────────────────", label),
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    ))
}

fn binding_row(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled("    ", Style::default()),
        Span::styled(
            format!(" {:<16}", key),
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default()),
        Span::styled(desc.to_string(), Style::default().fg(Color::White)),
    ])
}