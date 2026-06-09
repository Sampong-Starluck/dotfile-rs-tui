use ratatui::{
    text::{Line, Span},
    style::{Modifier, Style},
    prelude::Color,
    layout::Rect,
    Frame,
    widgets::{Block, Tabs}
};
use crate::models::TabModel;

pub fn render_tab(frame: &mut Frame, area: Rect, active: TabModel) {
    let titles: Vec<Line> = TabModel::ALL.iter()
        .map(|t| Line::from(Span::raw(t.title())))
        .collect();

    let selected = TabModel::ALL.iter()
        .position(|t| *t == active).unwrap_or(0);

    let tabs = Tabs::new(titles)
        .block(Block::bordered().title(" Navigation "))
        // .block(Block::default()
        //     .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
        //     .title(" Navigation "))
        .select(selected)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .divider("│");

    frame.render_widget(tabs, area);
}