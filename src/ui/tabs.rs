use ratatui::{
    text::{Line, Span},
    style::{Modifier, Style},
    prelude::Color,
    layout::Rect,
    Frame,
    widgets::{Block, Borders, BorderType, Tabs}
};
use crate::models::TabModel;

pub fn render_tab(frame: &mut Frame, area: Rect, active: TabModel) {
    let titles: Vec<Line> = TabModel::ALL.iter()
        .map(|t| {
            let label = match t {
                TabModel::Home        => "  ~ Home  ",
                TabModel::Application => "  # Apps  ",
                TabModel::Script      => "  $ Scripts  ",
            };
            Line::from(label)
        })
        .collect();

    let selected = TabModel::ALL.iter()
        .position(|t| *t == active).unwrap_or(0);

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(Span::styled(
                    "  dotfile-rs  ",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )),
        )
        .select(selected)
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED),
        )
        .divider(Span::styled("│", Style::default().fg(Color::DarkGray)));

    frame.render_widget(tabs, area);
}