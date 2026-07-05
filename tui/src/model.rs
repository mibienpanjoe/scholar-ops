//! Domain types for a tracker row, and parsers for its cells.
//! Cell formats come from `modes/_shared.md` (the tracker row contract).

use chrono::{Datelike, NaiveDate};

/// Verdict badge. Cells may carry trailing ` ⚠` flags; parsing ignores them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Apply,
    Maybe,
    Skip,
    Ineligible,
    Dead,
    Unknown, // cell absent or garbled — surfaced, never invented
}

impl Verdict {
    pub fn from_cell(s: &str) -> Verdict {
        let head = s
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_ascii_uppercase();
        match head.as_str() {
            "APPLY" => Verdict::Apply,
            "MAYBE" => Verdict::Maybe,
            "SKIP" => Verdict::Skip,
            "INELIGIBLE" => Verdict::Ineligible,
            "DEAD" => Verdict::Dead,
            _ => Verdict::Unknown,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Verdict::Apply => "APPLY",
            Verdict::Maybe => "MAYBE",
            Verdict::Skip => "SKIP",
            Verdict::Ineligible => "INELIGIBLE",
            Verdict::Dead => "DEAD",
            Verdict::Unknown => "?",
        }
    }
}

/// Application status — the closed vocabulary from BR-06.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Found,
    Evaluated,
    Preparing,
    Applied,
    Awaiting,
    Interview,
    Won,
    Lost,
    Dead,
    Unknown,
}

impl Status {
    /// The closed vocabulary in workflow order (excludes `Unknown`). Drives the
    /// status-edit popup in M8 — the only statuses the TUI may write.
    pub const VOCAB: [Status; 9] = [
        Status::Found,
        Status::Evaluated,
        Status::Preparing,
        Status::Applied,
        Status::Awaiting,
        Status::Interview,
        Status::Won,
        Status::Lost,
        Status::Dead,
    ];

    pub fn from_cell(s: &str) -> Status {
        match s.trim().to_ascii_lowercase().as_str() {
            "found" => Status::Found,
            "evaluated" => Status::Evaluated,
            "preparing" => Status::Preparing,
            "applied" => Status::Applied,
            "awaiting" => Status::Awaiting,
            "interview" => Status::Interview,
            "won" => Status::Won,
            "lost" => Status::Lost,
            "dead" => Status::Dead,
            _ => Status::Unknown,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Status::Found => "found",
            Status::Evaluated => "evaluated",
            Status::Preparing => "preparing",
            Status::Applied => "applied",
            Status::Awaiting => "awaiting",
            Status::Interview => "interview",
            Status::Won => "won",
            Status::Lost => "lost",
            Status::Dead => "dead",
            Status::Unknown => "unknown",
        }
    }
}

/// How close a deadline is — drives color and the urgency glyph (BR-03).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Urgency {
    Passed,
    Fire,   // < 7 days
    Warn,   // < 14 days
    Later,  // >= 14 days
    Rolling,
    Unknown,
}

impl Urgency {
    pub fn glyph(self) -> &'static str {
        match self {
            Urgency::Passed => "✗",
            Urgency::Fire => "🔥",
            Urgency::Warn => "⚠",
            Urgency::Later => " ",
            Urgency::Rolling => "∞",
            Urgency::Unknown => "?",
        }
    }
}

/// A deadline cell: a concrete date, `rolling`, or `unknown`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Deadline {
    Date(NaiveDate),
    Rolling,
    Unknown,
}

impl Deadline {
    pub fn from_cell(s: &str) -> Deadline {
        let t = s.trim();
        match t.to_ascii_lowercase().as_str() {
            "rolling" => Deadline::Rolling,
            "unknown" | "" => Deadline::Unknown,
            _ => match NaiveDate::parse_from_str(t, "%Y-%m-%d") {
                Ok(d) => Deadline::Date(d),
                Err(_) => Deadline::Unknown, // never guess a date (INV-06)
            },
        }
    }

    pub fn days_remaining(&self, today: NaiveDate) -> Option<i64> {
        match self {
            Deadline::Date(d) => Some((*d - today).num_days()),
            _ => None,
        }
    }

    pub fn urgency(&self, today: NaiveDate) -> Urgency {
        match self {
            Deadline::Date(d) => {
                let days = (*d - today).num_days();
                if days < 0 {
                    Urgency::Passed
                } else if days < 7 {
                    Urgency::Fire
                } else if days < 14 {
                    Urgency::Warn
                } else {
                    Urgency::Later
                }
            }
            Deadline::Rolling => Urgency::Rolling,
            Deadline::Unknown => Urgency::Unknown,
        }
    }

    /// Sort key for ascending order: dated first (earliest → latest), then
    /// `rolling`, then `unknown` (BR-03 / tracker row contract).
    pub fn sort_key(&self) -> (u8, i32) {
        match self {
            Deadline::Date(d) => (0, d.num_days_from_ce()),
            Deadline::Rolling => (1, 0),
            Deadline::Unknown => (2, 0),
        }
    }
}

/// One tracker row. `*_raw` fields keep the original cell text so display and
/// the M8 status rewrite stay faithful to what Claude wrote.
#[derive(Debug, Clone)]
pub struct Scholarship {
    pub name: String,
    pub provider: String,
    pub level: String,
    pub country: String,
    pub deadline: Deadline,
    pub deadline_raw: String,
    pub score: Option<f32>,
    pub verdict: Verdict,
    pub verdict_raw: String,
    pub status: Status,
    pub report: Option<String>,
    pub url: String,
}

impl Scholarship {
    /// Build a row from its 10 trimmed cells (Name … URL). `None` if too few.
    pub fn from_cells(cells: &[&str]) -> Option<Scholarship> {
        if cells.len() < 10 {
            return None;
        }
        // A cell holding `—`, `-`, or empty means "no value".
        let dash = |s: &str| {
            let t = s.trim();
            if t.is_empty() || t == "—" || t == "-" {
                None
            } else {
                Some(t.to_string())
            }
        };
        Some(Scholarship {
            name: cells[0].trim().to_string(),
            provider: cells[1].trim().to_string(),
            level: cells[2].trim().to_string(),
            country: cells[3].trim().to_string(),
            deadline: Deadline::from_cell(cells[4]),
            deadline_raw: cells[4].trim().to_string(),
            score: dash(cells[5]).and_then(|s| s.parse::<f32>().ok()),
            verdict: Verdict::from_cell(cells[6]),
            verdict_raw: cells[6].trim().to_string(),
            status: Status::from_cell(cells[7]),
            report: dash(cells[8]),
            url: cells[9].trim().to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn parse_deadline_date() {
        assert_eq!(Deadline::from_cell("2026-10-31"), Deadline::Date(date(2026, 10, 31)));
    }

    #[test]
    fn parse_deadline_rolling_unknown_garbage() {
        assert_eq!(Deadline::from_cell("rolling"), Deadline::Rolling);
        assert_eq!(Deadline::from_cell("unknown"), Deadline::Unknown);
        assert_eq!(Deadline::from_cell(""), Deadline::Unknown);
        assert_eq!(Deadline::from_cell("not a date"), Deadline::Unknown);
    }

    #[test]
    fn urgency_bands() {
        let today = date(2026, 7, 5);
        assert_eq!(Deadline::from_cell("2026-07-01").urgency(today), Urgency::Passed);
        assert_eq!(Deadline::from_cell("2026-07-08").urgency(today), Urgency::Fire); // 3d
        assert_eq!(Deadline::from_cell("2026-07-17").urgency(today), Urgency::Warn); // 12d
        assert_eq!(Deadline::from_cell("2026-09-01").urgency(today), Urgency::Later);
        assert_eq!(Deadline::Rolling.urgency(today), Urgency::Rolling);
    }

    #[test]
    fn verdict_ignores_flags() {
        assert_eq!(Verdict::from_cell("APPLY ⚠"), Verdict::Apply);
        assert_eq!(Verdict::from_cell("ineligible"), Verdict::Ineligible);
        assert_eq!(Verdict::from_cell("weird"), Verdict::Unknown);
    }

    #[test]
    fn row_parses_score_and_paths() {
        let cells = [
            "DAAD EPOS", "DAAD", "masters", "Germany", "2026-10-31", "4.20", "APPLY",
            "preparing", "reports/daad-epos.md", "https://daad.de/x",
        ];
        let s = Scholarship::from_cells(&cells).unwrap();
        assert_eq!(s.name, "DAAD EPOS");
        assert_eq!(s.score, Some(4.20));
        assert_eq!(s.verdict, Verdict::Apply);
        assert_eq!(s.status, Status::Preparing);
        assert_eq!(s.report.as_deref(), Some("reports/daad-epos.md"));
        assert_eq!(s.url, "https://daad.de/x");
    }

    #[test]
    fn row_dashes_become_none() {
        let cells = [
            "X", "Y", "masters", "various", "unknown", "—", "INELIGIBLE", "found", "—",
            "https://x",
        ];
        let s = Scholarship::from_cells(&cells).unwrap();
        assert_eq!(s.score, None);
        assert_eq!(s.report, None);
        assert_eq!(s.deadline, Deadline::Unknown);
    }

    #[test]
    fn too_few_cells_is_none() {
        assert!(Scholarship::from_cells(&["only", "three"]).is_none());
    }
}
