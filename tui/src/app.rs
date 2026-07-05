//! Application state and the update logic key events drive. Rendering lives in
//! `ui.rs`; this module owns *what* is shown, not *how* it is drawn.

use std::path::PathBuf;

use chrono::{Local, NaiveDate};
use ratatui::widgets::{ListState, TableState};

use crate::data;
use crate::model::{PipelineItem, Scholarship, Status, Urgency, Verdict};

/// The top-level view, switched with Tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Tracker,
    Pipeline,
}

/// Open status-edit popup. `url` snapshots the target row (the unique key), so
/// the write can find it again even if the tracker changed underneath us.
pub struct StatusEdit {
    pub url: String,
    pub name: String,
    pub state: ListState,
}

pub struct App {
    /// Repo root — `data/` and `reports/` resolve relative to it.
    pub root: PathBuf,
    pub view: View,

    // Tracker view.
    pub scholarships: Vec<Scholarship>,
    /// Indices into `scholarships` passing the active filters, in display order.
    pub visible: Vec<usize>,
    pub table: TableState,
    /// Vertical scroll offset of the detail/report pane.
    pub detail_scroll: u16,
    /// Cached body of the selected row's report file (loaded on selection change).
    pub report_body: Option<String>,

    // Filters (tracker only).
    pub filter_text: String,
    pub urgent_only: bool,
    pub verdict_filter: Option<Verdict>,
    /// True while the `/` text box is capturing keystrokes.
    pub input_mode: bool,

    // Pipeline view.
    pub pipeline: Vec<PipelineItem>,
    pub pipeline_state: ListState,

    /// Some when the status-edit popup is open (captures keys until confirmed).
    pub status_edit: Option<StatusEdit>,

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
            visible: Vec::new(),
            table: TableState::default(),
            detail_scroll: 0,
            report_body: None,
            filter_text: String::new(),
            urgent_only: false,
            verdict_filter: None,
            input_mode: false,
            pipeline: Vec::new(),
            pipeline_state: ListState::default(),
            status_edit: None,
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
        self.pipeline_state
            .select(clamp(self.pipeline_state.selected(), self.pipeline.len()));
        self.recompute_visible();
    }

    pub fn toggle_view(&mut self) {
        self.view = match self.view {
            View::Tracker => View::Pipeline,
            View::Pipeline => View::Tracker,
        };
    }

    /// The selected scholarship, mapped through the visible-index set.
    pub fn selected(&self) -> Option<&Scholarship> {
        let sel = self.table.selected()?;
        let idx = *self.visible.get(sel)?;
        self.scholarships.get(idx)
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
                let next = wrapped(self.table.selected(), self.visible.len(), delta);
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

    // --- status edit (the only write) ---------------------------------------

    pub fn status_editing(&self) -> bool {
        self.status_edit.is_some()
    }

    /// Open the popup for the selected row, pre-selecting its current status.
    pub fn open_status_editor(&mut self) {
        if let Some(s) = self.selected() {
            let start = Status::VOCAB.iter().position(|v| *v == s.status).unwrap_or(0);
            let mut state = ListState::default();
            state.select(Some(start));
            self.status_edit = Some(StatusEdit {
                url: s.url.clone(),
                name: s.name.clone(),
                state,
            });
        }
    }

    pub fn status_move(&mut self, delta: isize) {
        if let Some(e) = &mut self.status_edit {
            let next = wrapped(e.state.selected(), Status::VOCAB.len(), delta);
            e.state.select(next);
        }
    }

    pub fn cancel_status(&mut self) {
        self.status_edit = None;
    }

    /// Commit the chosen status: re-read the tracker, rewrite the matching row's
    /// Status cell atomically, reload, and keep the cursor on the same row.
    pub fn confirm_status(&mut self) {
        let Some(e) = self.status_edit.take() else {
            return;
        };
        let chosen = Status::VOCAB[e.state.selected().unwrap_or(0)];
        let path = self.path("data/scholarships.md");
        match data::write_status(&path, &e.url, chosen.label()) {
            Ok(data::StatusWrite::Written) => {
                self.reload();
                if let Some(pos) = self.visible.iter().position(|&i| self.scholarships[i].url == e.url)
                {
                    self.table.select(Some(pos));
                    self.load_report();
                }
                self.message = Some(format!("status → {} · {}", chosen.label(), e.name));
            }
            Ok(data::StatusWrite::RowNotFound) => {
                self.reload();
                self.message = Some(format!("row not found (changed on disk?): {}", e.name));
            }
            Err(err) => self.message = Some(format!("status write failed: {err}")),
        }
    }

    // --- filters -------------------------------------------------------------

    pub fn start_filter(&mut self) {
        self.input_mode = true;
    }

    /// Leave the text box. `clear` wipes the query; otherwise it's kept applied.
    pub fn end_filter(&mut self, clear: bool) {
        self.input_mode = false;
        if clear {
            self.filter_text.clear();
            self.recompute_visible();
        }
    }

    pub fn push_filter_char(&mut self, c: char) {
        self.filter_text.push(c);
        self.recompute_visible();
    }

    pub fn pop_filter_char(&mut self) {
        self.filter_text.pop();
        self.recompute_visible();
    }

    pub fn toggle_urgent(&mut self) {
        self.urgent_only = !self.urgent_only;
        self.recompute_visible();
    }

    /// Cycle the verdict filter: none → APPLY → MAYBE → SKIP → INELIGIBLE → DEAD → none.
    pub fn cycle_verdict_filter(&mut self) {
        self.verdict_filter = match self.verdict_filter {
            None => Some(Verdict::Apply),
            Some(Verdict::Apply) => Some(Verdict::Maybe),
            Some(Verdict::Maybe) => Some(Verdict::Skip),
            Some(Verdict::Skip) => Some(Verdict::Ineligible),
            Some(Verdict::Ineligible) => Some(Verdict::Dead),
            _ => None,
        };
        self.recompute_visible();
    }

    /// Does a row survive the active filters?
    fn passes(&self, s: &Scholarship) -> bool {
        if self.urgent_only {
            match s.deadline.urgency(self.today) {
                Urgency::Fire | Urgency::Warn => {}
                _ => return false,
            }
        }
        if let Some(v) = self.verdict_filter
            && s.verdict != v
        {
            return false;
        }
        if !self.filter_text.is_empty() {
            let q = self.filter_text.to_ascii_lowercase();
            let hay = format!("{} {} {}", s.name, s.provider, s.level).to_ascii_lowercase();
            if !hay.contains(&q) {
                return false;
            }
        }
        true
    }

    /// Rebuild the visible-index set from the filters, then fix up selection.
    fn recompute_visible(&mut self) {
        let vis: Vec<usize> = (0..self.scholarships.len())
            .filter(|&i| self.passes(&self.scholarships[i]))
            .collect();
        self.visible = vis;
        self.table
            .select(clamp(self.table.selected(), self.visible.len()));
        self.load_report();
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
            visible: Vec::new(),
            table: TableState::default(),
            detail_scroll: 0,
            report_body: None,
            filter_text: String::new(),
            urgent_only: false,
            verdict_filter: None,
            input_mode: false,
            pipeline: Vec::new(),
            pipeline_state: ListState::default(),
            status_edit: None,
            today: NaiveDate::from_ymd_opt(2026, 7, 5).unwrap(),
            should_quit: false,
            message: None,
        };
        app.recompute_visible();
        app.table.select(if app.visible.is_empty() { None } else { Some(0) });
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::parse_tracker;

    fn sample() -> Vec<Scholarship> {
        parse_tracker(
            "| Name | P | L | C | Deadline | S | V | St | R | URL |\n\
             |---|---|---|---|---|---|---|---|---|---|\n\
             | DAAD EPOS | DAAD | masters | de | 2026-07-08 | 4.2 | APPLY | found | — | https://a |\n\
             | Chevening | FCDO | masters | uk | 2026-12-01 | 3.1 | MAYBE | found | — | https://b |\n\
             | Ghost | X | masters | x | unknown | — | INELIGIBLE | found | — | https://c |\n",
        )
    }

    #[test]
    fn text_filter_narrows_visible() {
        let mut app = App::for_test(sample());
        assert_eq!(app.visible.len(), 3);
        app.push_filter_char('d');
        app.push_filter_char('a');
        app.push_filter_char('a');
        assert_eq!(app.visible.len(), 1);
        assert_eq!(app.selected().unwrap().name, "DAAD EPOS");
    }

    #[test]
    fn urgent_only_keeps_near_deadline() {
        let mut app = App::for_test(sample());
        app.toggle_urgent(); // today is 2026-07-05; only the 07-08 row is < 14d
        assert_eq!(app.visible.len(), 1);
        assert_eq!(app.selected().unwrap().name, "DAAD EPOS");
    }

    #[test]
    fn verdict_filter_cycles() {
        let mut app = App::for_test(sample());
        app.cycle_verdict_filter(); // APPLY
        assert_eq!(app.visible.len(), 1);
        assert_eq!(app.selected().unwrap().verdict, Verdict::Apply);
    }

    #[test]
    fn confirm_status_writes_disk_and_memory() {
        let dir = std::env::temp_dir().join(format!("sops-app-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("data")).unwrap();
        let tracker = dir.join("data/scholarships.md");
        std::fs::write(
            &tracker,
            "| Name | P | L | C | Deadline | S | V | Status | R | URL |\n\
             |---|---|---|---|---|---|---|---|---|---|\n\
             | DAAD | DAAD | masters | de | 2026-10-31 | 4.20 | APPLY | found | — | https://daad.de/x |\n",
        )
        .unwrap();

        let mut app = App::for_test(vec![]);
        app.root = dir.clone();
        app.reload();
        assert_eq!(app.scholarships.len(), 1);

        app.open_status_editor(); // starts on "found" (VOCAB index 0)
        app.status_move(3); // → "applied"
        app.confirm_status();

        assert!(app.status_edit.is_none());
        assert_eq!(app.scholarships[0].status, Status::Applied);
        let after = std::fs::read_to_string(&tracker).unwrap();
        assert!(after.contains("| applied |"), "after:\n{after}");
        assert!(after.contains("APPLY")); // verdict untouched

        std::fs::remove_dir_all(&dir).ok();
    }
}
