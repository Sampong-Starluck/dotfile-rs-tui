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
    use ratatui::widgets::Clear;
    frame.render_widget(Clear, frame.area());

    let area = default_layout(frame);
    render_tab(frame, area.tabs, app.active_tab);

    match app.active_tab {
        TabModel::Home        => features::home_render(frame, area.sidebar, area.body, app),
        TabModel::Application => features::app_render(frame, area.sidebar, area.body, app),
        TabModel::Shell       => features::shell_render(frame, area.sidebar, area.body, app),
    }

    render_status(frame, area.status, app);
}

pub fn render_status(frame: &mut Frame, area: Rect, app: &App) {
    use ratatui::{
        text::{Line, Span},
        widgets::Paragraph,
    };

    let hints: Vec<Span> = match app.active_tab {
        TabModel::Application => match app.app_focus {
            AppFocus::PmPicker => vec![
                styled_key("↑↓/jk"),
                plain(" navigate  "),
                styled_key("Enter"),
                plain(" select  "),
                styled_key("Esc"),
                plain(" cancel"),
            ],
            AppFocus::Section => {
                let mgr = app.active_package_manager();
                let i_hint = if mgr == "winget" { " search  " } else { " add package  " };
                vec![
                    styled_key("↑↓/jk"),
                    plain(" navigate  "),
                    styled_key("Space/Enter"),
                    plain(" open section  "),
                    styled_key("i"),
                    plain(i_hint),
                    styled_key("p"),
                    plain(&format!(" select package manager [{}]  ", mgr)),
                    styled_key("d"),
                    plain(" install selected  "),
                    styled_key("Tab"),
                    plain(" switch tab"),
                ]
            }
            AppFocus::Apps => {
                let i_hint = if app.active_package_manager() == "winget" {
                    " search  "
                } else {
                    " add package  "
                };
                let mgr = app.active_package_manager();
                vec![
                    styled_key("↑↓/jk"),
                    plain(" navigate  "),
                    styled_key("Space"),
                    plain(" toggle select  "),
                    styled_key("i"),
                    plain(i_hint),
                    styled_key("p"),
                    plain(&format!(" select package manager [{}]  ", mgr)),
                    styled_key("d"),
                    plain(" install  "),
                    styled_key("Esc"),
                    plain(" back to sections"),
                ]
            }
            AppFocus::CustomInput => vec![
                styled_key("type"),
                plain(" enter package name  "),
                styled_key("Space"),
                plain(" add to list  "),
                styled_key("Enter"),
                plain(" confirm  "),
                styled_key("Esc"),
                plain(" cancel"),
            ],
            AppFocus::Search => vec![
                styled_key("type"),
                plain(" edit query  "),
                styled_key("Enter"),
                plain(" search  "),
                styled_key("↑↓/jk"),
                plain(" navigate results  "),
                styled_key("Space"),
                plain(" select  "),
                styled_key("Esc"),
                plain(" close search"),
            ],
            AppFocus::Installing => vec![
                styled_key("type"),
                plain(" respond to prompt  "),
                styled_key("Enter"),
                plain(" send  "),
                styled_key("Esc"),
                plain(" close"),
            ],
            AppFocus::SudoConfirm => vec![
                styled_key("type"),
                plain(" enter password  "),
                styled_key("Enter"),
                plain(" confirm  "),
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