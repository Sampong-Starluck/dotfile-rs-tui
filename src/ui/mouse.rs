use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use crate::app::App;
use crate::enumerate::AppFocus;
use crate::models::TabModel;

pub fn handle_mouse(app: &mut App, event: MouseEvent) {
    // While help is open, any click closes it; swallow all other mouse events.
    if app.show_help {
        if matches!(event.kind, MouseEventKind::Down(_)) {
            app.show_help = false;
        }
        return;
    }

    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => click(app, event.column, event.row),
        MouseEventKind::ScrollDown => scroll(app, event.column, event.row, 1i8),
        MouseEventKind::ScrollUp   => scroll(app, event.column, event.row, -1i8),
        _ => {}
    }
}

fn click(app: &mut App, col: u16, row: u16) {
    if hit(app.mouse_tabs, col, row) {
        tab_click(app, col);
        return;
    }
    if hit(app.mouse_sidebar, col, row) {
        sidebar_click(app, row);
        return;
    }
    if hit(app.mouse_body, col, row) {
        body_click(app, row);
    }
}

fn tab_click(app: &mut App, col: u16) {
    // Tab bar inner content starts after the left border (x + 1).
    // Titles: "  ~ Home  " (10 chars) | "│" | "  # Apps  " (10 chars)
    let inner = col.saturating_sub(app.mouse_tabs.x + 1) as usize;
    let new_tab = if inner < 10 {
        TabModel::Home
    } else if inner >= 11 {
        TabModel::Application
    } else {
        return; // divider
    };
    if new_tab != app.active_tab {
        app.active_tab = new_tab;
        app.app_focus = AppFocus::Section;
        app.app_custom_input.clear();
    }
}

fn sidebar_click(app: &mut App, row: u16) {
    // List items start one row below the top border.
    let idx = row.saturating_sub(app.mouse_sidebar.y + 1) as usize;
    match app.active_tab {
        TabModel::Home => {
            if idx < app.package_managers.len() {
                app.selected_pm = idx;
                app.apps = None;
                app.app_selected_section = 0;
                app.app_selected_app = 0;
            }
        }
        TabModel::Application => {
            let section_count = app.apps.as_ref().map(|a| a.len()).unwrap_or(0);
            if idx < section_count {
                if idx == app.app_selected_section
                    && matches!(app.app_focus, AppFocus::Section | AppFocus::Apps)
                {
                    // Second click on the already-selected section → enter apps list
                    app.app_focus = AppFocus::Apps;
                } else {
                    app.app_selected_section = idx;
                    app.app_selected_app = 0;
                    app.app_focus = AppFocus::Section;
                }
            }
        }
    }
}

fn body_click(app: &mut App, row: u16) {
    // Items start one row below the top border.
    let visual_row = row.saturating_sub(app.mouse_body.y + 1) as usize;
    let idx = visual_row + app.body_list_offset;

    match app.active_tab {
        TabModel::Application => match app.app_focus {
            AppFocus::Apps | AppFocus::Section => {
                let len = app.apps.as_ref()
                    .and_then(|a| a.get(app.app_selected_section))
                    .map(|s| s.apps.len())
                    .unwrap_or(0);
                if idx < len {
                    app.app_selected_app = idx;
                    app.app_focus = AppFocus::Apps;
                }
            }
            AppFocus::Search => {
                if idx < app.search_results.len() {
                    app.search_selected = idx;
                }
            }
            AppFocus::Installed => {
                if idx < app.installed_packages.len() {
                    app.installed_selected = idx;
                }
            }
            _ => {}
        },
        TabModel::Home => {
            // Body is command table — no clickable items.
        }
    }
}

fn scroll(app: &mut App, col: u16, row: u16, delta: i8) {
    let in_sidebar = hit(app.mouse_sidebar, col, row);
    let in_body    = hit(app.mouse_body, col, row);

    if in_sidebar {
        sidebar_scroll(app, delta);
    } else if in_body {
        body_scroll(app, delta);
    } else {
        // Anywhere else: scroll command table on Home, body list on Application.
        match app.active_tab {
            TabModel::Home        => home_scroll(app, delta),
            TabModel::Application => body_scroll(app, delta),
        }
    }
}

fn sidebar_scroll(app: &mut App, delta: i8) {
    match app.active_tab {
        TabModel::Home => {
            if delta > 0 {
                app.next_pkg();
            } else {
                app.previous_pkg();
            }
            app.apps = None;
            app.app_selected_section = 0;
            app.app_selected_app = 0;
        }
        TabModel::Application => {
            let max = app.apps.as_ref().map(|a| a.len()).unwrap_or(0).saturating_sub(1);
            if delta > 0 {
                app.app_selected_section = (app.app_selected_section + 1).min(max);
            } else {
                app.app_selected_section = app.app_selected_section.saturating_sub(1);
            }
            app.app_selected_app = 0;
        }
    }
}

fn body_scroll(app: &mut App, delta: i8) {
    match app.active_tab {
        TabModel::Application => match app.app_focus {
            AppFocus::Apps | AppFocus::Section => {
                let max = app.apps.as_ref()
                    .and_then(|a| a.get(app.app_selected_section))
                    .map(|s| s.apps.len().saturating_sub(1))
                    .unwrap_or(0);
                if delta > 0 {
                    app.app_selected_app = (app.app_selected_app + 1).min(max);
                    if app.app_focus == AppFocus::Section {
                        app.app_focus = AppFocus::Apps;
                    }
                } else {
                    app.app_selected_app = app.app_selected_app.saturating_sub(1);
                }
            }
            AppFocus::Search => {
                let max = app.search_results.len().saturating_sub(1);
                if delta > 0 {
                    app.search_selected = (app.search_selected + 1).min(max);
                } else {
                    app.search_selected = app.search_selected.saturating_sub(1);
                }
            }
            AppFocus::Installed => {
                let max = app.installed_packages.len().saturating_sub(1);
                if delta > 0 {
                    app.installed_selected = (app.installed_selected + 1).min(max);
                } else {
                    app.installed_selected = app.installed_selected.saturating_sub(1);
                }
            }
            _ => {}
        },
        TabModel::Home => home_scroll(app, delta),
    }
}

fn home_scroll(app: &mut App, delta: i8) {
    if delta > 0 { app.scroll_down(); } else { app.scroll_up(); }
}

fn hit(area: Rect, col: u16, row: u16) -> bool {
    col >= area.x && col < area.x + area.width
        && row >= area.y && row < area.y + area.height
}