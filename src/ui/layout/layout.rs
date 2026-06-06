use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use crate::models::AppArea;

pub fn default_layout(frame: &mut Frame) -> AppArea {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            // Constraint::Length(3), //header
            Constraint::Length(3), //tab bar
            Constraint::Fill(3), // main contrain
            Constraint::Length(1), //status bar
        ]).split(frame.area());

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(24), // side bar
            Constraint::Fill(1), //body
        ]).split(vertical[1]);

    AppArea {
        // header:  vertical[0],
        tabs:    vertical[0],
        sidebar: horizontal[0],
        body:    horizontal[1],
        status:  vertical[2],
    }
}