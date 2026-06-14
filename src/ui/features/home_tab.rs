use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    text::{Line, Span},
    style::{Color, Modifier, Style},
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
    widgets::{Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table},
};
use crate::app::App;

pub fn home_render(frame: &mut Frame, sidebar: Rect, body: Rect, app: &App) {
    render_sidebar(frame, sidebar, app);
    render_body(frame, body, app);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            app.next_pkg();
            app.apps = None;
            app.app_selected_section = 0;
            app.app_selected_app = 0;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.previous_pkg();
            app.apps = None;
            app.app_selected_section = 0;
            app.app_selected_app = 0;
        }
        _ => {}
    }
}

fn rounded_block(title: &str, focused: bool) -> Block<'static> {
    let border_color = if focused { Color::Cyan } else { Color::DarkGray };
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))
}

fn render_sidebar(frame: &mut Frame, area: Rect, app: &App) {
    if app.package_managers.is_empty() {
        frame.render_widget(
            Paragraph::new("\n  No package managers detected.")
                .style(Style::default().fg(Color::DarkGray))
                .block(rounded_block("Package Managers", false)),
            area,
        );
        return;
    }

    let items: Vec<ListItem> = app.package_managers
        .iter()
        .enumerate()
        .map(|(i, pm)| {
            let is_active = i == app.selected_pm;
            let label = if area.width < 18 { pm.binary() } else { pm.label() };
            let (prefix, style) = if is_active {
                ("▶ ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            } else {
                ("  ", Style::default().fg(Color::DarkGray))
            };
            ListItem::new(Span::styled(format!("{}{}", prefix, label), style))
        })
        .collect();

    let title = if area.width < 18 { "PM" } else { "Package Managers" };
    let mut state = ListState::default().with_selected(Some(app.selected_pm));

    frame.render_stateful_widget(
        List::new(items)
            .block(rounded_block(title, true))
            .highlight_style(Style::default()),  // we draw highlight manually
        area,
        &mut state,
    );
}

fn render_body(frame: &mut Frame, area: Rect, app: &App) {
    let platform_height = if area.height < 15 { 3 } else { 5 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(platform_height), Constraint::Fill(1)])
        .split(area);

    render_platform_info(frame, chunks[0], app);
    render_command_table(frame, chunks[1], app);
}

fn render_platform_info(frame: &mut Frame, area: Rect, app: &App) {
    let is_short = area.height < 4;

    let os_line = Line::from(vec![
        Span::styled("  OS      ", Style::default().fg(Color::DarkGray)),
        Span::styled("│  ", Style::default().fg(Color::DarkGray)),
        Span::styled(app.os.to_string(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ]);

    let pm_line = match app.selected_manager() {
        Some(pm) => Line::from(vec![
            Span::styled("  Manager ", Style::default().fg(Color::DarkGray)),
            Span::styled("│  ", Style::default().fg(Color::DarkGray)),
            Span::styled(pm.label(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  ({})", pm.binary()), Style::default().fg(Color::DarkGray)),
        ]),
        None => Line::from(Span::styled("  No package manager selected", Style::default().fg(Color::DarkGray))),
    };

    let lines = if is_short {
        vec![Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(app.os.to_string(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                app.selected_manager().map(|pm| pm.label()).unwrap_or("none"),
                Style::default().fg(Color::Yellow),
            ),
        ])]
    } else {
        vec![Line::raw(""), os_line, pm_line]
    };

    frame.render_widget(
        Paragraph::new(lines).block(rounded_block("Platform", false)),
        area,
    );
}

fn render_command_table(frame: &mut Frame, area: Rect, app: &App) {
    let Some(pm) = app.selected_manager() else {
        frame.render_widget(
            Paragraph::new("\n  Select a package manager from the sidebar.")
                .style(Style::default().fg(Color::DarkGray))
                .block(rounded_block("Commands", false)),
            area,
        );
        return;
    };

    if area.width < 20 || area.height < 4 {
        frame.render_widget(
            Paragraph::new("\n  Terminal too small.")
                .block(rounded_block("Commands", false)),
            area,
        );
        return;
    }

    let commands     = pm.commands();
    let total_rows   = commands.len() as u16;
    let visible_rows = area.height.saturating_sub(4);
    let max_scroll   = total_rows.saturating_sub(visible_rows);
    let scroll       = app.command_scroll.min(max_scroll);

    let inner_width = area.width.saturating_sub(4);
    let cmd_width   = ((inner_width as f32 * 0.45) as u16).max(15);
    let desc_width  = inner_width.saturating_sub(cmd_width).max(10);

    let header = Row::new(vec![
        Cell::from("Command").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Description").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ])
    .height(1)
    .bottom_margin(1);

    let rows: Vec<Row> = commands
        .iter()
        .skip(scroll as usize)
        .map(|cmd| {
            let desc = if desc_width < 15 { "" } else { cmd.description };
            Row::new(vec![
                Cell::from(cmd.command).style(Style::default().fg(Color::Yellow)),
                Cell::from(desc).style(Style::default().fg(Color::White)),
            ])
        })
        .collect();

    let title = if total_rows > visible_rows {
        format!("Commands — {}  [{}/{}]", pm.label(), scroll + 1, total_rows)
    } else {
        format!("Commands — {}", pm.label())
    };

    frame.render_widget(
        Table::new(rows, [Constraint::Length(cmd_width), Constraint::Length(desc_width)])
            .header(header)
            .block(rounded_block(&title, false)),
        area,
    );
}