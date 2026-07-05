//! scholar-ops TUI — a read-only dashboard over the local tracker files.
//!
//! Reads `data/scholarships.md` and shows it as a scrollable table. Claude still
//! does all evaluation; this is a fast, zero-token view over what it wrote.

mod app;
mod data;
mod model;
mod ui;

use std::io;

use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
};

use crate::app::{App, View};

fn main() -> io::Result<()> {
    // Build state before touching the terminal, so an early error can't leave the
    // terminal in raw mode. `init` then flips into raw mode + the alternate screen.
    let mut app = App::new();
    let mut terminal = ratatui::init();
    let outcome = run(&mut terminal, &mut app);
    ratatui::restore();
    outcome
}

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|frame| ui::draw(frame, app))?;
        handle_event(app)?;
    }
    Ok(())
}

/// Block for one event and fold it into app state.
fn handle_event(app: &mut App) -> io::Result<()> {
    if let Event::Key(key) = event::read()? {
        if key.kind != KeyEventKind::Press {
            return Ok(()); // ignore key-release (Windows emits both)
        }

        // The status popup is modal: it captures keys until confirmed/cancelled.
        if app.status_editing() {
            match key.code {
                KeyCode::Down | KeyCode::Char('j') => app.status_move(1),
                KeyCode::Up | KeyCode::Char('k') => app.status_move(-1),
                KeyCode::Enter => app.confirm_status(),
                KeyCode::Esc | KeyCode::Char('s') => app.cancel_status(),
                _ => {}
            }
            return Ok(());
        }

        // While the filter box is open, keys edit the query instead of navigating.
        if app.input_mode {
            match key.code {
                KeyCode::Char(c) => app.push_filter_char(c),
                KeyCode::Backspace => app.pop_filter_char(),
                KeyCode::Enter => app.end_filter(false),
                KeyCode::Esc => app.end_filter(true),
                _ => {}
            }
            return Ok(());
        }

        app.message = None; // any keypress clears the transient footer message
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
            KeyCode::Tab => app.toggle_view(),
            KeyCode::Down | KeyCode::Char('j') => app.select_next(),
            KeyCode::Up | KeyCode::Char('k') => app.select_prev(),
            KeyCode::PageDown => app.scroll_detail(3),
            KeyCode::PageUp => app.scroll_detail(-3),
            KeyCode::Char('s') if app.view == View::Tracker => app.open_status_editor(),
            KeyCode::Char('/') if app.view == View::Tracker => app.start_filter(),
            KeyCode::Char('u') if app.view == View::Tracker => app.toggle_urgent(),
            KeyCode::Char('v') if app.view == View::Tracker => app.cycle_verdict_filter(),
            KeyCode::Char('r') => app.reload(),
            _ => {}
        }
    }
    Ok(())
}
