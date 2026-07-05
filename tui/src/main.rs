//! scholar-ops TUI — a read-only dashboard over the local tracker files.
//!
//! Reads `data/scholarships.md` and shows it as a scrollable table. Claude still
//! does all evaluation; this is a fast, zero-token view over what it wrote.

// Model/data types are added a milestone ahead of the UI that consumes them, so
// allow dead code until the viewer wires everything up.
#![allow(dead_code)]

mod app;
mod data;
mod model;
mod ui;

use std::io;

use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
};

use crate::app::App;

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
        app.message = None; // any keypress clears the transient footer message
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
            KeyCode::Down | KeyCode::Char('j') => app.next_row(),
            KeyCode::Up | KeyCode::Char('k') => app.prev_row(),
            KeyCode::Char('r') => app.reload(),
            _ => {}
        }
    }
    Ok(())
}
