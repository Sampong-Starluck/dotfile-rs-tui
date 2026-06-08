use std::io::stdout;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind},
    event,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};
use crossterm::event::KeyEventKind;
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
    logging::init();
    tracing::info!("Starting RataTUI application ....");

    // Single stdout handle used everywhere
    let mut stdout = stdout();

    enable_raw_mode()?;
    execute!(
        stdout,              // ← same handle
        EnterAlternateScreen,
        EnableMouseCapture,
    )?;

    let backend = CrosstermBackend::new(stdout); // ← same handle moved in
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal);

    // Restore using the backend's writer, not a new stdout()
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(), // ← reach into the backend's handle
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
                // FIX: on Windows, crossterm emits KeyEventKind::Repeat for held keys
                // in addition to KeyEventKind::Press. Linux suppresses repeats at the
                // terminal level so it worked fine on Arch. Filtering here ensures one
                // logical keypress = one navigation step on both platforms.
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                // Global keys
                match key.code {
                    KeyCode::Tab => {
                        app.app_focus = AppFocus::Section;
                        app.app_custom_input.clear();
                        app.active_tab = app.active_tab.next();
                        // Force Ratatui to forget its previous buffer entirely so
                        // the next draw() redraws every cell unconditionally.
                        terminal.clear()?;
                        terminal.draw(|frame| ui::render(&mut app, frame))?; // immediate redraw
                    }
                    KeyCode::BackTab => {
                        app.app_focus = AppFocus::Section;
                        app.app_custom_input.clear();
                        app.active_tab = app.active_tab.prev();
                        terminal.clear()?;
                        terminal.draw(|frame| ui::render(&mut app, frame))?; // immediate redraw
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
            // FIX: handle resize — Ratatui needs to know the terminal
            // changed size so it invalidates its buffer and does a full
            // redraw. Without this, resizing leaves ghost content at
            // the old dimensions.
            Event::Resize(_, _) => {
                terminal.clear()?;
            }
            _ => {}
        }
    }
    Ok(())
}

// fn reset_tab_state(app: &mut App) {
//     // Reset application tab focus
//     app.app_focus = AppFocus::Section;
//     app.app_custom_input.clear();
// }