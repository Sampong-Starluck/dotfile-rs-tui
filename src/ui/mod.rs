use ratatui::Frame;
use crate::app::App;
use crate::features;
use crate::models::TabModel;

mod tabs;
mod layout;

use super::ui::{layout::default_layout, tabs::render_tab };

pub fn render(app: &App, frame: &mut Frame) {
    let area = default_layout(frame);

    // Header
    // render_header(frame, area.header);

    //Tab bar
    render_tab(frame, area.tabs, app.active_tab);

    // Active feature
    match app.active_tab {
        TabModel::Home    => features::home_render(frame, area.sidebar, area.body, app),
        // TabModel::Package => features::packages::render(frame, areas.sidebar, areas.body, app),
        // TabModel::Shell   => features::shell::render(frame, areas.sidebar, areas.body, app),
        TabModel::Application => features::app_render(frame, area.sidebar, area.body, app),
        TabModel::Shell => features::shell_render(frame, area.sidebar, area.body, app),
    }

    // Status bar
    render_status(frame, area.status, app);
}

fn render_header(frame: &mut Frame, area: ratatui::layout::Rect) {
    use ratatui::widgets::{Block, Paragraph};
    let block = Paragraph::new(" My App")
        .block(Block::bordered());
    frame.render_widget(block, area);
}

pub fn render_status(frame: &mut Frame, area: ratatui::layout::Rect, _app: &App) {
    use ratatui::{
        text::Span,
        style::{Color, Style},
        widgets::Paragraph,
    };

    let hint = Span::styled(
        "Tab/Shift+Tab: switch, q: Quit",
        Style::default().fg(Color::DarkGray),
    );
    frame.render_widget(Paragraph::new(hint), area);
}