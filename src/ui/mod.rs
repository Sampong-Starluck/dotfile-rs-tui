use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use crate::app::App;
use crate::enumerate::AppFocus;
use crate::models::TabModel;

mod tabs;
mod layout;
pub(crate) mod features;
mod help;
mod mouse;

pub use mouse::handle_mouse;

use super::ui::{layout::default_layout, tabs::render_tab};

pub fn render(app: &mut App, frame: &mut Frame) {
    use ratatui::widgets::Clear;
    frame.render_widget(Clear, frame.area());

    let area = default_layout(frame);

    // Cache areas for mouse hit-testing
    app.mouse_tabs    = area.tabs;
    app.mouse_sidebar = area.sidebar;
    app.mouse_body    = area.body;

    render_tab(frame, area.tabs, app.active_tab);

    match app.active_tab {
        TabModel::Home        => features::home_render(frame, area.sidebar, area.body, app),
        TabModel::Application => features::app_render(frame, area.sidebar, area.body, app),
    }

    render_status(frame, area.status, app);

    if app.show_help {
        help::render_help(frame, app);
    }
}

pub fn render_status(frame: &mut Frame, area: Rect, app: &App) {
    use ratatui::widgets::{Block, Borders, Paragraph};

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    frame.render_widget(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray)),
        rows[0],
    );

    let pairs = bindings_for(app.active_tab, app.app_focus);
    let hint_spans = hints_from_pairs(&pairs);

    let mgr_suffix: Vec<Span> = match (app.active_tab, app.app_focus) {
        (TabModel::Application, AppFocus::Section)
        | (TabModel::Application, AppFocus::Apps) => {
            let mgr = app.active_package_manager();
            vec![
                sep(),
                dim(&format!("  pkg: {}  ", mgr)),
                sep(),
                dim("  Tab switch  "),
            ]
        }
        _ => vec![],
    };

    let mut all: Vec<Span> = hint_spans;
    all.extend(mgr_suffix);

    frame.render_widget(
        Paragraph::new(ratatui::text::Line::from(all)),
        rows[1],
    );
}

/// Returns the (key, description) pairs for the given tab + focus state.
/// Both the status bar and the help overlay consume this.
pub(crate) fn bindings_for(tab: TabModel, focus: AppFocus) -> Vec<(&'static str, &'static str)> {
    match tab {
        TabModel::Application => match focus {
            AppFocus::PmPicker => vec![
                ("↑↓", "navigate"),
                ("Enter", "select"),
                ("Esc", "cancel"),
            ],
            AppFocus::Section => vec![
                ("↑↓", "navigate"),
                ("Enter", "open"),
                ("i", "search"),
                ("l", "installed"),
                ("d", "install"),
                ("p", "pm"),
                ("?", "help"),
            ],
            AppFocus::Apps => vec![
                ("↑↓", "navigate"),
                ("Space", "select"),
                ("i", "search"),
                ("l", "installed"),
                ("d", "install"),
                ("Esc", "back"),
                ("p", "pm"),
                ("?", "help"),
            ],
            AppFocus::Installed => vec![
                ("↑↓", "navigate"),
                ("Space", "select"),
                ("d", "remove"),
                ("r", "refresh"),
                ("Esc", "back"),
            ],
            AppFocus::Search => vec![
                ("type", "query"),
                ("Enter", "search"),
                ("↑↓", "navigate"),
                ("Space", "select"),
                ("Esc", "close"),
            ],
            AppFocus::CustomInput => vec![
                ("type", "package name"),
                ("Enter", "add"),
                ("Esc", "cancel"),
            ],
            AppFocus::Installing => vec![
                ("type", "respond"),
                ("Enter", "send"),
                ("Esc", "close"),
            ],
            AppFocus::SudoConfirm => vec![
                ("type", "password"),
                ("Enter", "confirm"),
                ("Esc", "cancel"),
            ],
        },
        TabModel::Home => vec![
            ("↑↓", "navigate"),
            ("Tab", "switch tab"),
            ("q", "quit"),
            ("?", "help"),
        ],
    }
}

/// Build a Vec<Span> of key-chip + description spans from (key, desc) pairs.
pub(crate) fn hints_from_pairs(pairs: &[(&'static str, &'static str)]) -> Vec<Span<'static>> {
    hints(pairs)
}

/// Shared `centered_rect` helper — percentage-based centered popup area.
pub(crate) fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn hints(pairs: &[(&str, &str)]) -> Vec<Span<'static>> {
    let mut out: Vec<Span<'static>> = Vec::new();
    for (i, (key, desc)) in pairs.iter().enumerate() {
        if i > 0 {
            out.push(sep());
        }
        out.push(key_chip(key));
        out.push(dim(&format!(" {}  ", desc)));
    }
    out
}

fn key_chip(label: &str) -> Span<'static> {
    Span::styled(
        format!(" {} ", label),
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
}

pub(crate) fn sep() -> Span<'static> {
    Span::styled(
        "  │  ",
        Style::default().fg(Color::DarkGray),
    )
}

pub(crate) fn dim(label: &str) -> Span<'static> {
    Span::styled(
        label.to_string(),
        Style::default().fg(Color::DarkGray),
    )
}