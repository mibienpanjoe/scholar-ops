//! scholar-ops TUI — a read-only dashboard over the local tracker files.
//!
//! M0: terminal skeleton. Draws one bordered frame and quits on `q`.
//! Everything real (parsing, the table, deadlines) arrives in later milestones.

use std::io;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    widgets::{Block, Paragraph},
};

fn main() -> io::Result<()> {
    // `init` puts the terminal into raw mode + the alternate screen, and installs
    // a panic hook that restores it if we crash. It hands back a ready terminal.
    let mut terminal = ratatui::init();

    // Run the app. We capture the result instead of using `?` so that we always
    // reach `restore()` below on the normal (non-panic) exit path.
    let outcome = run(&mut terminal);

    // Undo the terminal changes whether `run` returned Ok or Err. (On a *panic*
    // the hook from `init` handles this instead — unwinding skips this line.)
    ratatui::restore();
    outcome
}

/// The event loop: draw a frame, block for a key, repeat until the user hits `q`.
///
/// Takes `&mut DefaultTerminal` — a *borrow*: `main` still owns the terminal, we
/// just get temporary mutable access. `?` bubbles any IO error up to `main`.
fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    loop {
        terminal.draw(render)?;

        // `event::read` blocks until the next terminal event (key, resize, …).
        if let Event::Key(key) = event::read()? {
            // Some platforms emit both Press and Release; only act on Press.
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(());
            }
        }
    }
}

/// Renders one frame. Right now it's a pure function of nothing; once `App` state
/// exists it will take `&App` and paint the table, panes, and footer from it.
fn render(frame: &mut Frame) {
    let block = Block::bordered().title(" scholar-ops dashboard ");
    let body = Paragraph::new("M0 skeleton — the tracker table lands in M3.\n\nPress q to quit.")
        .block(block);
    frame.render_widget(body, frame.area());
}
