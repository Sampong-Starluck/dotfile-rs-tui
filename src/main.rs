use std::io::stdout;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind},
    event,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal
};
use crate::{
    app::App,
    enumerate::AppFocus,
    models::TabModel
};

mod app;
mod ui;
mod models;
mod logging;
mod service;
mod enumerate;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    
    // enable log
    logging::init();
    tracing::info!("Starting RataTUI application ....");

    // Manual terminal setup instead of ratatui::init()
    // so we control the exact sequence
    enable_raw_mode()?;
    execute!(
        stdout(),
        EnterAlternateScreen,
        EnableMouseCapture,
    )?;

    let backend  = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal);

    // Always restore even if run() errored
    disable_raw_mode()?;
    execute!(
        stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> color_eyre::Result<()> {
    let mut app = App::new();
    while app.running {
        terminal.draw(|frame| ui::render(&mut app, frame))?;
        match event::read()? {
            Event::Key(key) => {
                // Global keys
                match key.code {
                    KeyCode::Tab => {
                        app.app_focus = AppFocus::Section; // reset FIRST
                        app.app_custom_input.clear();
                        app.active_tab = app.active_tab.next();
                    }
                    KeyCode::BackTab => {
                        app.app_focus = AppFocus::Section; // reset FIRST
                        app.app_custom_input.clear();
                        app.active_tab = app.active_tab.prev();
                    }
                    KeyCode::Char('p') => {
                        app.cycle_package_manager();
                    }
                    KeyCode::Char('q') => app.running = false,
                    // Delegate to active tab
                    _ => match app.active_tab {
                        TabModel::Home        => ui::features::home_handle_key(&mut app, key),
                        TabModel::Application => ui::features::app_handle_key(&mut app, key),
                        TabModel::Shell       => ui::features::shell_handle_key(&mut app, key),
                    }
                }
            }
            Event::Mouse(mouse) => {
                match mouse.kind {
                    MouseEventKind::ScrollDown => app.scroll_down(),
                    MouseEventKind::ScrollUp   => app.scroll_up(),
                    _ => {}
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn reset_tab_state(app: &mut App) {
    // Reset application tab focus
    app.app_focus = AppFocus::Section;
    app.app_custom_input.clear();
}