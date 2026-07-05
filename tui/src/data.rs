//! Load the tracker markdown into `Scholarship` rows. Tolerant of a missing
//! file and of malformed lines (mirrors `tracker-check.mjs`). The status write
//! is added in M8.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::model::{PipelineItem, Scholarship};

/// Result of a status write.
#[derive(Debug, PartialEq, Eq)]
pub enum StatusWrite {
    Written,
    /// No row had the given URL — it was renamed/removed since the snapshot.
    RowNotFound,
}

/// Rewrite exactly one row's Status cell in `data/scholarships.md`, matching the
/// row by its URL (the unique key). Every other cell and every other line is
/// preserved byte-for-byte. Written atomically (temp file + rename) so a crash
/// or a concurrent reader never sees a half-written tracker.
///
/// This is the TUI's **only** write. It never touches verdict, score, report,
/// or any other column — those belong to Claude (INV-04/05/06).
pub fn write_status(path: &Path, url: &str, new_status: &str) -> io::Result<StatusWrite> {
    let text = fs::read_to_string(path)?;
    let mut wrote = false;
    let mut out: Vec<String> = Vec::with_capacity(text.lines().count());
    for line in text.lines() {
        if !wrote
            && let Some(edited) = edit_status_line(line, url, new_status)
        {
            out.push(edited);
            wrote = true;
            continue;
        }
        out.push(line.to_string());
    }
    if !wrote {
        return Ok(StatusWrite::RowNotFound);
    }

    let mut new_text = out.join("\n");
    if text.ends_with('\n') {
        new_text.push('\n');
    }

    // Temp file in the same directory → rename is atomic on the same filesystem.
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    fs::write(&tmp, new_text)?;
    fs::rename(&tmp, path)?;
    Ok(StatusWrite::Written)
}

/// If `line` is the tracker row for `url`, return it with only the Status cell
/// replaced; otherwise `None`. Splitting on `|` keeps every other segment (and
/// its surrounding whitespace) exactly as it was.
fn edit_status_line(line: &str, url: &str, new_status: &str) -> Option<String> {
    if !line.trim_start().starts_with('|') {
        return None;
    }
    let mut segs: Vec<&str> = line.split('|').collect();
    // "" + 10 cells + "" == 12 segments. Cell k is segment k+1:
    // Status = cell 7 → segment 8; URL = cell 9 → segment 10.
    if segs.len() < 12 || segs[10].trim() != url {
        return None;
    }
    let cell = format!(" {new_status} ");
    segs[8] = &cell;
    Some(segs.join("|"))
}

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
    fn write_status_changes_only_status_cell() {
        let path = std::env::temp_dir().join(format!("sops-wtest-{}.md", std::process::id()));
        let original = "\
| Name | Provider | Level | Country | Deadline | Score | Verdict | Status | Report | URL |
|------|----------|-------|---------|----------|-------|---------|--------|--------|-----|
| DAAD | DAAD | masters | de | 2026-10-31 | 4.20 | APPLY | found | reports/x.md | https://daad.de/x |
";
        fs::write(&path, original).unwrap();

        let out = write_status(&path, "https://daad.de/x", "applied").unwrap();
        assert_eq!(out, StatusWrite::Written);

        let after = fs::read_to_string(&path).unwrap();
        assert!(after.contains("| applied |"), "after:\n{after}");
        assert!(after.contains("APPLY")); // verdict untouched
        assert!(after.contains("4.20")); // score untouched
        assert!(after.contains("https://daad.de/x")); // url untouched
        assert!(!after.contains("| found |")); // old status gone
        assert!(after.ends_with('\n')); // trailing newline preserved

        let nf = write_status(&path, "https://nope", "applied").unwrap();
        assert_eq!(nf, StatusWrite::RowNotFound);

        fs::remove_file(&path).ok();
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
