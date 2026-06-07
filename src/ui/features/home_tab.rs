use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    text::{Line, Span},
    style::{Color, Modifier, Style},
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
    widgets::{Block, Cell, List, ListItem, ListState, Paragraph, Row, Table}
};
use crate::app::App;

pub fn home_render(frame: &mut Frame, sidebar: Rect, body: Rect, app: &App) {
    render_sidebar(frame, sidebar, app);
    render_body(frame, body, app);
}

// src/ui/features/home_tab.rs
pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            app.next_pkg();
            app.apps = None; // reload app list with new pkg manager filter
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

fn render_sidebar(frame: &mut Frame, area: Rect, app: &App) {
    if app.package_managers.is_empty() {
        let msg = Paragraph::new("No package managers detected.")
            .block(Block::bordered().title(" Package Managers "));
        frame.render_widget(msg, area);
        return;
    }

    let items: Vec<ListItem> = app.package_managers
        .iter()
        .map(|pm| {
            // Truncate label if sidebar is too narrow
            let label = if area.width < 18 {
                pm.binary().to_string()
            } else {
                pm.label().to_string()
            };
            ListItem::new(format!(" {}", label))
        })
        .collect();

    let title = if area.width < 18 {
        " PM "
    } else {
        " Package Managers "
    };

    let list = List::new(items)
        .block(Block::bordered().title(title))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::REVERSED),  // visible highlight even without color
        )
        .highlight_symbol("► ");

    let mut state = ListState::default().with_selected(Some(app.selected_pm));
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_body(frame: &mut Frame, area: Rect, app: &App) {
    // Platform block height: shrink to 3 if area is short
    let platform_height = if area.height < 15 { 3 } else { 5 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(platform_height),
            Constraint::Fill(1),
        ])
        .split(area);

    render_platform_info(frame, chunks[0], app);
    render_command_table(frame, chunks[1], app);
}

fn render_platform_info(frame: &mut Frame, area: Rect, app: &App) {
    let is_short = area.height < 4;

    let os_line = Line::from(vec![
        Span::styled("  OS:       ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            app.os.to_string(),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
    ]);

    let selected_line = match app.selected_manager() {
        Some(pm) => Line::from(vec![
            Span::styled("  Selected: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                pm.label(),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  ({})", pm.binary()),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        None => Line::from(Span::styled(
            "  No package manager selected",
            Style::default().fg(Color::DarkGray),
        )),
    };

    // When area is short, combine into one line to fit
    let lines = if is_short {
        vec![Line::from(vec![
            Span::styled("  OS: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                app.os.to_string(),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                app.selected_manager()
                    .map(|pm| pm.label())
                    .unwrap_or("none"),
                Style::default().fg(Color::Yellow),
            ),
        ])]
    } else {
        vec![Line::raw(""), os_line, selected_line]
    };

    let block = Paragraph::new(lines)
        .block(Block::bordered().title(" Platform "));

    frame.render_widget(block, area);
}

fn render_command_table(frame: &mut Frame, area: Rect, app: &App) {
    let Some(pm) = app.selected_manager() else {
        let msg = Paragraph::new("\n  Select a package manager from the sidebar.")
            .block(Block::bordered().title(" Commands "));
        frame.render_widget(msg, area);
        return;
    };

    if area.width < 20 || area.height < 4 {
        let msg = Paragraph::new("\n  Terminal too small.")
            .block(Block::bordered().title(" Commands "));
        frame.render_widget(msg, area);
        return;
    }

    let commands = pm.commands();
    let total_rows   = commands.len() as u16;
    let visible_rows = area.height.saturating_sub(4);
    let max_scroll   = total_rows.saturating_sub(visible_rows);
    let scroll       = app.command_scroll.min(max_scroll);

    // Responsive column split: command gets 45%, description gets the rest
    let inner_width = area.width.saturating_sub(4); // 2 borders + 2 padding
    let cmd_width   = ((inner_width as f32 * 0.45) as u16).max(15);
    let desc_width  = inner_width.saturating_sub(cmd_width).max(10);

    let header = Row::new(vec![
        Cell::from("Command").style(
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Cell::from("Description").style(
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
    ])
        .height(1)
        .bottom_margin(1);

    let rows: Vec<Row> = commands
        .iter()
        .skip(scroll as usize)
        .map(|cmd| {
            // Truncate description if too narrow to prevent overflow
            let desc = if desc_width < 15 {
                ""
            } else {
                cmd.description
            };

            Row::new(vec![
                Cell::from(cmd.command).style(
                    Style::default().fg(Color::Yellow),
                ),
                Cell::from(desc).style(
                    Style::default().fg(Color::White),
                ),
            ])
        })
        .collect();

    let scroll_indicator = if total_rows > visible_rows {
        format!(
            " Commands — {}  [{}/{}] ▲▼ ",
            pm.label(),
            scroll + 1,
            total_rows,
        )
    } else {
        format!(" Commands — {} ", pm.label())
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(cmd_width),
            Constraint::Length(desc_width),
        ],
    )
        .header(header)
        .block(Block::bordered().title(scroll_indicator));

    frame.render_widget(table, area);
}