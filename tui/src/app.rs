//! Application state and the update logic key events drive. Rendering lives in
//! `ui.rs`; this module owns *what* is shown, not *how* it is drawn.

use std::path::PathBuf;

use chrono::{Local, NaiveDate};
use ratatui::widgets::TableState;

use crate::data;
use crate::model::Scholarship;

/// Which pane keyboard scrolling drives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Table,
    Detail,
}

pub struct App {
    /// Repo root — `data/` and `reports/` resolve relative to it.
    pub root: PathBuf,
    pub scholarships: Vec<Scholarship>,
    pub table: TableState,
    pub today: NaiveDate,
    pub should_quit: bool,
    /// Transient one-line footer message (errors, confirmations).
    pub message: Option<String>,
    /// Which pane arrow keys scroll.
    pub focus: Focus,
    /// Vertical scroll offset of the detail/report pane.
    pub detail_scroll: u16,
    /// Cached body of the selected row's report file (loaded on selection change).
    pub report_body: Option<String>,
}

impl App {
    pub fn new() -> App {
        let mut app = App {
            root: find_repo_root(),
            scholarships: Vec::new(),
            table: TableState::default(),
            today: Local::now().date_naive(),
            should_quit: false,
            message: None,
            focus: Focus::Table,
            detail_scroll: 0,
            report_body: None,
        };
        app.reload();
        app
    }

    /// Resolve a repo-relative path against the detected root.
    pub fn path(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }

    /// Re-read the tracker from disk, keeping the selection in range.
    pub fn reload(&mut self) {
        match data::load_tracker(&self.path("data/scholarships.md")) {
            Ok(mut rows) => {
                data::sort_by_deadline(&mut rows);
                self.scholarships = rows;
                self.clamp_selection();
                self.load_report();
            }
            Err(e) => self.message = Some(format!("tracker read error: {e}")),
        }
    }

    pub fn selected(&self) -> Option<&Scholarship> {
        self.table.selected().and_then(|i| self.scholarships.get(i))
    }

    pub fn next_row(&mut self) {
        self.move_selection(1);
        self.load_report();
    }

    pub fn prev_row(&mut self) {
        self.move_selection(-1);
        self.load_report();
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Table => Focus::Detail,
            Focus::Detail => Focus::Table,
        };
    }

    /// Scroll the detail pane, clamped at the top (bottom clamp is visual only).
    pub fn scroll_detail(&mut self, delta: i16) {
        self.detail_scroll = self.detail_scroll.saturating_add_signed(delta);
    }

    /// Read the selected row's report file into `report_body` and reset scroll.
    /// A missing file is reported inline, not treated as fatal.
    fn load_report(&mut self) {
        self.detail_scroll = 0;
        let rel = self.selected().and_then(|s| s.report.clone());
        self.report_body = rel.map(|p| {
            std::fs::read_to_string(self.path(&p))
                .unwrap_or_else(|_| format!("(report file not found: {p})"))
        });
    }

    /// Move the selection by `delta`, wrapping at the ends.
    fn move_selection(&mut self, delta: isize) {
        let len = self.scholarships.len();
        if len == 0 {
            self.table.select(None);
            return;
        }
        let cur = self.table.selected().unwrap_or(0) as isize;
        let next = (cur + delta).rem_euclid(len as isize) as usize;
        self.table.select(Some(next));
    }

    fn clamp_selection(&mut self) {
        let len = self.scholarships.len();
        if len == 0 {
            self.table.select(None);
        } else {
            let sel = self.table.selected().unwrap_or(0).min(len - 1);
            self.table.select(Some(sel));
        }
    }
}

#[cfg(test)]
impl App {
    /// Build an app straight from in-memory rows, bypassing disk — for render
    /// tests that must not depend on the real `data/` directory.
    pub fn for_test(scholarships: Vec<Scholarship>) -> App {
        App {
            root: PathBuf::from("."),
            scholarships,
            table: TableState::default().with_selected(Some(0)),
            today: NaiveDate::from_ymd_opt(2026, 7, 5).unwrap(),
            should_quit: false,
            message: None,
            focus: Focus::Table,
            detail_scroll: 0,
            report_body: None,
        }
    }
}

/// Walk up from the current directory to the first ancestor containing `data/`.
/// Falls back to the current directory (yielding an empty dashboard) if none.
fn find_repo_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut dir = cwd.as_path();
    loop {
        if dir.join("data").is_dir() {
            return dir.to_path_buf();
        }
        match dir.parent() {
            Some(p) => dir = p,
            None => return cwd,
        }
    }
}
