//! Hand a URL to the OS default browser. This is the **Seeker's** outward action
//! — the TUI never fetches the page itself (INV-11 / the invariant wall). It only
//! spawns the platform "open" helper, detached, and returns immediately.

use std::io;
use std::process::{Command, Stdio};

/// Open `url` in the default browser, detached. Returns as soon as the helper is
/// spawned; never blocks on (or reads from) the browser.
pub fn open_url(url: &str) -> io::Result<()> {
    launcher()
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
}

#[cfg(target_os = "linux")]
fn launcher() -> Command {
    Command::new("xdg-open")
}

#[cfg(target_os = "macos")]
fn launcher() -> Command {
    Command::new("open")
}

#[cfg(target_os = "windows")]
fn launcher() -> Command {
    // `start` needs an (empty) window-title arg before the URL.
    let mut c = Command::new("cmd");
    c.args(["/C", "start", ""]);
    c
}
