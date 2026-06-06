use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use crate::app::App;

pub fn shell_render(frame: &mut Frame, sidebar: Rect, body: Rect, app: &App) {
    let area = frame.area();

    // Create a vertical split: one centered panel
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // top margin
            Constraint::Length(10), // panel height
            Constraint::Min(0),     // remaining space
        ])
        .split(area);

    let panel_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // left margin
            Constraint::Percentage(60), // panel width
            Constraint::Percentage(20), // right margin
        ])
        .split(layout[1])[1]; // take the middle column

    let panel = Paragraph::new("Welcome to the default page\n\nPress 'q' to quit.")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Default Page ")
                .style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(panel, panel_area);
}