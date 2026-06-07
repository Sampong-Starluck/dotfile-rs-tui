use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use crate::models::AppArea;

pub fn default_layout(frame: &mut Frame) -> AppArea {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // tab bar
            Constraint::Fill(1),    // main content
            Constraint::Length(1),  // status bar
        ])
        .split(frame.area());

    // Responsive sidebar width based on total terminal width
    let total_width = frame.area().width;
    let sidebar_width = if total_width < 80 {
        20                          // very narrow terminal
    } else if total_width < 120 {
        (total_width / 4).max(20)   // ~25% of width
    } else {
        30                          // cap at 30 on wide terminals
    };

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(sidebar_width),  // fixed responsive sidebar
            Constraint::Fill(1),                // body takes the rest
        ])
        .split(vertical[1]);

    AppArea {
        tabs:    vertical[0],
        sidebar: horizontal[0],
        body:    horizontal[1],
        status:  vertical[2],
    }
}