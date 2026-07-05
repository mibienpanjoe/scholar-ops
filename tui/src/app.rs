//! Application state and the update logic key events drive. Rendering lives in
//! `ui.rs`; this module owns *what* is shown, not *how* it is drawn.

use std::path::PathBuf;

use chrono::{Local, NaiveDate};
use ratatui::widgets::{ListState, TableState};

use crate::data;
use crate::model::{PipelineItem, Scholarship};

/// The top-level view, switched with Tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Tracker,
    Pipeline,
}

pub struct App {
    /// Repo root — `data/` and `reports/` resolve relative to it.
    pub root: PathBuf,
    pub view: View,

    // Tracker view.
    pub scholarships: Vec<Scholarship>,
    pub table: TableState,
    /// Vertical scroll offset of the detail/report pane.
    pub detail_scroll: u16,
    /// Cached body of the selected row's report file (loaded on selection change).
    pub report_body: Option<String>,

    // Pipeline view.
    pub pipeline: Vec<PipelineItem>,
    pub pipeline_state: ListState,

    pub today: NaiveDate,
    pub should_quit: bool,
    /// Transient one-line footer message (errors, confirmations).
    pub message: Option<String>,
}

impl App {
    pub fn new() -> App {
        let mut app = App {
            root: find_repo_root(),
            view: View::Tracker,
            scholarships: Vec::new(),
            table: TableState::default(),
            detail_scroll: 0,
            report_body: None,
            pipeline: Vec::new(),
            pipeline_state: ListState::default(),
            today: Local::now().date_naive(),
            should_quit: false,
            message: None,
        };
        app.reload();
        app
    }

    /// Resolve a repo-relative path against the detected root.
    pub fn path(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }

    /// Re-read both data files from disk, keeping selections in range.
    pub fn reload(&mut self) {
        match data::load_tracker(&self.path("data/scholarships.md")) {
            Ok(mut rows) => {
                data::sort_by_deadline(&mut rows);
                self.scholarships = rows;
            }
            Err(e) => self.message = Some(format!("tracker read error: {e}")),
        }
        match data::load_pipeline(&self.path("data/pipeline.md")) {
            Ok(items) => self.pipeline = items,
            Err(e) => self.message = Some(format!("pipeline read error: {e}")),
        }
        self.clamp_selections();
        self.load_report();
    }

    pub fn toggle_view(&mut self) {
        self.view = match self.view {
            View::Tracker => View::Pipeline,
            View::Pipeline => View::Tracker,
        };
    }

    pub fn selected(&self) -> Option<&Scholarship> {
        self.table.selected().and_then(|i| self.scholarships.get(i))
    }

    /// Move selection down (`+1`) or up (`-1`) in whichever view is active.
    pub fn select_next(&mut self) {
        self.move_selection(1);
    }

    pub fn select_prev(&mut self) {
        self.move_selection(-1);
    }

    fn move_selection(&mut self, delta: isize) {
        match self.view {
            View::Tracker => {
                let next = wrapped(self.table.selected(), self.scholarships.len(), delta);
                self.table.select(next);
                self.load_report();
            }
            View::Pipeline => {
                let next = wrapped(self.pipeline_state.selected(), self.pipeline.len(), delta);
                self.pipeline_state.select(next);
            }
        }
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

    fn clamp_selections(&mut self) {
        self.table
            .select(clamp(self.table.selected(), self.scholarships.len()));
        self.pipeline_state
            .select(clamp(self.pipeline_state.selected(), self.pipeline.len()));
    }
}

/// Next index after moving `delta`, wrapping at the ends; `None` if empty.
fn wrapped(cur: Option<usize>, len: usize, delta: isize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let c = cur.unwrap_or(0) as isize;
    Some((c + delta).rem_euclid(len as isize) as usize)
}

/// Keep an existing selection within `[0, len)`; default to 0 for a non-empty list.
fn clamp(cur: Option<usize>, len: usize) -> Option<usize> {
    if len == 0 {
        None
    } else {
        Some(cur.unwrap_or(0).min(len - 1))
    }
}

#[cfg(test)]
impl App {
    /// Build an app straight from in-memory rows, bypassing disk — for render
    /// tests that must not depend on the real `data/` directory.
    pub fn for_test(scholarships: Vec<Scholarship>) -> App {
        let mut app = App {
            root: PathBuf::from("."),
            view: View::Tracker,
            scholarships,
            table: TableState::default().with_selected(Some(0)),
            detail_scroll: 0,
            report_body: None,
            pipeline: Vec::new(),
            pipeline_state: ListState::default(),
            today: NaiveDate::from_ymd_opt(2026, 7, 5).unwrap(),
            should_quit: false,
            message: None,
        };
        if app.scholarships.is_empty() {
            app.table.select(None);
        }
        app
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
