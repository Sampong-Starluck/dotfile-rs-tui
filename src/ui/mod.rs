use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use crate::app::App;
use crate::enumerate::AppFocus;
use crate::models::TabModel;

mod tabs;
mod layout;
pub(crate) mod features;

use super::ui::{layout::default_layout, tabs::render_tab};

pub fn render(app: &mut App, frame: &mut Frame) {
    use ratatui::widgets::Block;
    frame.render_widget(
        Block::default().style(Style::default().fg(Color::Reset).bg(Color::Reset)),
        frame.area(),
    );

    let area = default_layout(frame);
    render_tab(frame, area.tabs, app.active_tab);

    match app.active_tab {
        TabModel::Home        => features::home_render(frame, area.sidebar, area.body, app),
        TabModel::Application => features::app_render(frame, area.sidebar, area.body, app),
        TabModel::Shell       => features::shell_render(frame, area.sidebar, area.body, app),
    }

    render_status(frame, area.status, app);
}

// fn render_header(frame: &mut Frame, area: ratatui::layout::Rect) {
//     use ratatui::widgets::{Block, Paragraph};
//     let block = Paragraph::new(" My App")
//         .block(Block::bordered());
//     frame.render_widget(block, area);
// }

pub fn render_status(frame: &mut Frame, area: Rect, app: &App) {
    use ratatui::{
        text::{Line, Span},
        widgets::Paragraph,
    };

    let hints: Vec<Span> = match app.active_tab {
        TabModel::Application => match app.app_focus {
            AppFocus::Section => vec![
                styled_key("↑↓/jk"),
                plain(" navigate  "),
                styled_key("Space/Enter"),
                plain(" open section  "),
                styled_key("p"),
                plain(" switch pkg manager  "),
                styled_key("d"),
                plain(" install selected  "),
                styled_key("Tab"),
                plain(" switch tab"),
            ],
            AppFocus::Apps => vec![
                styled_key("↑↓/jk"),
                plain(" navigate  "),
                styled_key("Space"),
                plain(" toggle select  "),
                styled_key("i"),
                plain(" custom app  "),
                styled_key("d"),
                plain(" install  "),
                styled_key("Esc"),
                plain(" back to sections"),
            ],
            AppFocus::CustomInput => vec![
                styled_key("type"),
                plain(" enter app id  "),
                styled_key("Space"),
                plain(" add to list  "),
                styled_key("Enter"),
                plain(" confirm  "),
                styled_key("Esc"),
                plain(" cancel"),
            ],
            AppFocus::Installing => vec![
                styled_key("Esc"),
                plain(" close install log"),
            ],
            AppFocus::SudoConfirm => vec![
                styled_key("y"),
                plain(" confirm sudo  "),
                styled_key("n"),
                plain(" cancel  "),
                styled_key("Esc"),
                plain(" cancel"),
            ],
        },
        TabModel::Home => vec![
            styled_key("↑↓/jk"),
            plain(" navigate  "),
            styled_key("Tab"),
            plain(" switch tab  "),
            styled_key("q"),
            plain(" quit"),
        ],
        TabModel::Shell => vec![
            styled_key("Tab"),
            plain(" switch tab  "),
            styled_key("q"),
            plain(" quit"),
        ],
    };

    frame.render_widget(
        Paragraph::new(Line::from(hints)),
        area,
    );
}

fn styled_key(label: &str) -> Span<'static> {
    Span::styled(
        format!(" {} ", label),
        Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )
}

fn plain(label: &str) -> Span<'static> {
    Span::styled(
        label.to_string(),
        Style::default().fg(Color::DarkGray),
    )
}