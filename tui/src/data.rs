//! Load the tracker markdown into `Scholarship` rows. Tolerant of a missing
//! file and of malformed lines (mirrors `tracker-check.mjs`). The status write
//! is added in M8.

use std::fs;
use std::io;
use std::path::Path;

use crate::model::{PipelineItem, Scholarship};

/// Read `data/scholarships.md` into rows. A missing file is **not** an error —
/// a Seeker who hasn't evaluated anything yet still gets a working dashboard.
pub fn load_tracker(path: &Path) -> io::Result<Vec<Scholarship>> {
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };
    Ok(parse_tracker(&text))
}

/// Parse the markdown table body, skipping the header and separator rows and any
/// line that doesn't yield a full 10-cell row (defensive, like tracker-check).
pub fn parse_tracker(text: &str) -> Vec<Scholarship> {
    text.lines()
        .filter_map(|line| {
            let cells = split_row(line)?;
            if is_separator(&cells) || is_header(&cells) {
                return None;
            }
            Scholarship::from_cells(&cells)
        })
        .collect()
}

/// Order rows by deadline: dated ascending, then `rolling`, then `unknown`
/// (BR-03 / the tracker row contract). Stable, so ties keep file order.
pub fn sort_by_deadline(rows: &mut [Scholarship]) {
    rows.sort_by_key(|s| s.deadline.sort_key());
}

/// Read `data/pipeline.md` into inbox items (missing file → empty).
pub fn load_pipeline(path: &Path) -> io::Result<Vec<PipelineItem>> {
    match fs::read_to_string(path) {
        Ok(t) => Ok(parse_pipeline(&t)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e),
    }
}

/// Parse the inbox checklist, ignoring any line that isn't a `- [ ]`/`- [x]` item.
pub fn parse_pipeline(text: &str) -> Vec<PipelineItem> {
    text.lines().filter_map(parse_pipeline_line).collect()
}

fn parse_pipeline_line(line: &str) -> Option<PipelineItem> {
    let t = line.trim();
    let (done, rest) = if let Some(r) = t.strip_prefix("- [ ]") {
        (false, r)
    } else if let Some(r) = t.strip_prefix("- [x]").or_else(|| t.strip_prefix("- [X]")) {
        (true, r)
    } else {
        return None;
    };
    let mut parts = rest.split('|').map(str::trim);
    let url = parts.next()?.to_string();
    if url.is_empty() {
        return None;
    }
    let source = parts.next().unwrap_or("").to_string();
    let deadline = parts
        .next()
        .unwrap_or("")
        .trim_start_matches("deadline")
        .trim()
        .to_string();
    Some(PipelineItem { done, url, source, deadline })
}

/// Split a table line `| a | b | … |` into trimmed cells. `None` if the line
/// isn't a table row (doesn't start with `|`).
fn split_row(line: &str) -> Option<Vec<&str>> {
    let t = line.trim();
    if !t.starts_with('|') {
        return None;
    }
    let inner = t.trim_matches('|');
    Some(inner.split('|').map(str::trim).collect())
}

/// A `|---|:--:|` separator row: every cell is only dashes/colons.
fn is_separator(cells: &[&str]) -> bool {
    cells
        .iter()
        .all(|c| !c.is_empty() && c.chars().all(|ch| ch == '-' || ch == ':'))
}

/// The header row starts with the `Name` column label.
fn is_header(cells: &[&str]) -> bool {
    cells.first().is_some_and(|c| c.eq_ignore_ascii_case("Name"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Deadline, Verdict};

    const SAMPLE: &str = "\
| Name | Provider | Level | Country | Deadline | Score | Verdict | Status | Report | URL |
|------|----------|-------|---------|----------|-------|---------|--------|--------|-----|
| DAAD EPOS | DAAD | masters | Germany | 2026-10-31 | 4.20 | APPLY | preparing | reports/daad-epos.md | https://daad.de/x |
| Bad Row | too | few |
| Chevening | FCDO | masters | UK | rolling | 3.50 | MAYBE | found | — | https://chevening.org/y |
";

    #[test]
    fn parses_only_valid_rows() {
        let rows = parse_tracker(SAMPLE);
        assert_eq!(rows.len(), 2); // the 3-cell "Bad Row" is skipped
        assert_eq!(rows[0].name, "DAAD EPOS");
        assert_eq!(rows[0].verdict, Verdict::Apply);
        assert_eq!(rows[1].provider, "FCDO");
        assert_eq!(rows[1].deadline, Deadline::Rolling);
    }

    #[test]
    fn prose_lines_are_ignored() {
        let rows = parse_tracker("# Tracker\n\nsome note\n");
        assert!(rows.is_empty());
    }

    #[test]
    fn missing_file_is_empty_not_error() {
        let rows = load_tracker(Path::new("/no/such/scholarships.md")).unwrap();
        assert!(rows.is_empty());
    }

    #[test]
    fn parses_pipeline_items() {
        let items = parse_pipeline(
            "# Inbox\n\
             - [ ] https://daad.de/x | DAAD scan 2026-07-04 | deadline 2026-10-31\n\
             prose line, ignored\n\
             - [x] https://chevening.org/y | Chevening scan 2026-07-04 | deadline unknown\n",
        );
        assert_eq!(items.len(), 2);
        assert!(!items[0].done);
        assert_eq!(items[0].url, "https://daad.de/x");
        assert_eq!(items[0].source, "DAAD scan 2026-07-04");
        assert_eq!(items[0].deadline, "2026-10-31");
        assert!(items[1].done);
        assert_eq!(items[1].deadline, "unknown");
    }

    #[test]
    fn sorts_dated_then_rolling_then_unknown() {
        let mut rows = parse_tracker(
            "| Name | P | L | C | Deadline | S | V | St | R | URL |\n\
             |---|---|---|---|---|---|---|---|---|---|\n\
             | Z | p | masters | x | rolling | 1 | SKIP | found | — | https://a |\n\
             | Y | p | masters | x | 2026-12-01 | 1 | SKIP | found | — | https://b |\n\
             | X | p | masters | x | unknown | 1 | SKIP | found | — | https://c |\n\
             | W | p | masters | x | 2026-03-01 | 1 | SKIP | found | — | https://d |\n",
        );
        sort_by_deadline(&mut rows);
        let order: Vec<&str> = rows.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(order, ["W", "Y", "Z", "X"]); // dated asc, rolling, unknown
    }
}
