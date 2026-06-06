use std::io::stdout;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind};
use crossterm::{event, execute};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use crate::app::App;
use crate::models::TabModel;

mod app;
mod ui;
mod models;
mod features;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

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
        terminal.draw(|frame| ui::render(&app, frame))?;

        match event::read()? {
            Event::Key(key) => {
                // Global keys
                match key.code {
                    KeyCode::Tab       => app.active_tab = app.active_tab.next(),
                    KeyCode::BackTab   => app.active_tab = app.active_tab.prev(),
                    KeyCode::Char('q') => app.running = false,
                    _ => {}
                }

                // Tab-specific keys
                match app.active_tab {
                    TabModel::Home => match key.code {
                        KeyCode::Down | KeyCode::Char('j') => app.next_pkg(),
                        KeyCode::Up   | KeyCode::Char('k') => app.previous_pkg(),
                        _ => {}
                    },
                    _ => {}
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