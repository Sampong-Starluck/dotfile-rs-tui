use std::{io::{stdout, Write}, time::Duration};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
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
mod utils;

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
        // ── External interactive execution ─────────────────────────────────
        // Interactive managers (pacman, paru, apt, …) write prompts to the
        // controlling terminal, not to pipes. We leave the TUI, run the
        // command(s) with inherited stdio so the user can interact normally,
        // then restore the TUI.
        if !app.run_external.is_empty() {
            let commands        = std::mem::take(&mut app.run_external);
            let was_removing    = app.run_external_removing;
            app.run_external_removing = false;
            run_in_terminal(terminal, &commands)?;
            // Reset selection state
            app.app_selected_ids.clear();
            app.app_remove_mode = false;
            if was_removing {
                // Signal installed view to refresh on next open
                app.installed_packages.clear();
                app.installed_set.clear();
            }
            app.app_focus = AppFocus::Section;
            terminal.clear()?;
            continue;
        }

        terminal.draw(|frame| ui::render(&mut app, frame))?;

        // While loading, use a short timeout so the spinner can advance even
        // with no keypress. Otherwise block until the next input event.
        let loading = app.search_loading || app.installed_loading;
        let timeout = if loading { Duration::from_millis(80) } else { Duration::from_secs(10) };

        if !event::poll(timeout)? {
            // No event arrived — advance the animation tick and redraw.
            if loading {
                app.loading_tick = app.loading_tick.wrapping_add(1);
            }
            continue;
        }

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
                    // While help is open, only toggle/close keys pass through.
                    _ if app.show_help => {
                        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?')) {
                            app.show_help = false;
                        }
                    }
                    KeyCode::Char('?') if !app.is_text_input_focus() => {
                        app.show_help = true;
                    }
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
                    // Only quit when not typing in a text input field.
                    KeyCode::Char('q') if !app.is_text_input_focus() => app.running = false,
                    // Delegate to active tab
                    _ => match app.active_tab {
                        TabModel::Home        => ui::features::home_handle_key(&mut app, key),
                        TabModel::Application => ui::features::app_handle_key(&mut app, key),
                    }
                }
            }
            Event::Mouse(mouse) => {
                ui::handle_mouse(&mut app, mouse);
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

/// Leave the TUI, run `commands` in the real terminal with inherited stdio
/// (so sudo prompts, Y/N confirmations, coloured output all work normally),
/// then restore the TUI.
fn run_in_terminal(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    commands: &[String],
) -> color_eyre::Result<()> {
    // ── Leave TUI ──────────────────────────────────────────────────────────
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;

    // ── Run each command with full terminal access ─────────────────────────
    println!("\n\x1b[36m  Running {} command(s)\x1b[0m", commands.len());
    for cmd in commands {
        let mut parts = cmd.split_whitespace();
        let Some(binary) = parts.next() else { continue };
        let args: Vec<&str> = parts.collect();

        println!("\n\x1b[36m▶  {}\x1b[0m", cmd);
        match std::process::Command::new(binary).args(&args).status() {
            Ok(s) if s.success() => println!("\x1b[32m✓  Done\x1b[0m"),
            Ok(s)                => println!("\x1b[31m✗  Exited with status {}\x1b[0m", s),
            Err(e)               => println!("\x1b[31m✗  Failed to run '{}': {}\x1b[0m", binary, e),
        }
    }

    // ── Wait for user before returning to TUI ─────────────────────────────
    print!("\n\x1b[33m  Press Enter to return…\x1b[0m  ");
    std::io::stdout().flush()?;
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;

    // ── Restore TUI ────────────────────────────────────────────────────────
    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;

    Ok(())
}

// fn reset_tab_state(app: &mut App) {
//     // Reset application tab focus
//     app.app_focus = AppFocus::Section;
//     app.app_custom_input.clear();
// }