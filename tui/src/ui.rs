//! Rendering. Pure functions of `&App` (plus `&mut` where a widget needs its
//! scroll/selection state). No IO, no evaluation — just paint what's in memory.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Cell, Clear, List, ListItem, Paragraph, Row, Table, Tabs, Wrap},
};

use crate::app::{App, View};
use crate::model::{Status, Urgency, Verdict};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let [tabs, body, footer] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)])
            .areas(frame.area());

    render_tabs(frame, app, tabs);
    match app.view {
        View::Tracker => {
            let [left, right] =
                Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)])
                    .areas(body);
            render_tracker(frame, app, left);
            render_detail(frame, app, right);
        }
        View::Pipeline => render_pipeline(frame, app, body),
    }
    render_footer(frame, app, footer);

    if app.status_edit.is_some() {
        render_status_popup(frame, app);
    }
}

/// Modal list of the closed status vocabulary, centered over the whole frame.
fn render_status_popup(frame: &mut Frame, app: &mut App) {
    let height = Status::VOCAB.len() as u16 + 2;
    let area = popup_area(frame.area(), 34, height);
    let edit = app.status_edit.as_mut().expect("popup open");

    let items: Vec<ListItem> = Status::VOCAB
        .iter()
        .map(|s| ListItem::new(s.label()))
        .collect();
    let title = format!(" set status · {} ", truncate(&edit.name, 16));
    let list = List::new(items)
        .block(Block::bordered().title(title).border_style(Style::new().fg(Color::Cyan)))
        .highlight_style(Style::new().reversed())
        .highlight_symbol("▸ ");

    frame.render_widget(Clear, area); // wipe whatever's underneath
    frame.render_stateful_widget(list, area, &mut edit.state);
}

/// A centered rectangle of the given size, clamped to `area`.
fn popup_area(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max - 1).collect::<String>())
    }
}

fn render_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let selected = match app.view {
        View::Tracker => 0,
        View::Pipeline => 1,
    };
    let tabs = Tabs::new(vec![
        format!(" Tracker ({}) ", app.scholarships.len()),
        format!(" Pipeline ({}) ", app.pipeline.len()),
    ])
    .select(selected)
    .highlight_style(Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    .divider("");
    frame.render_widget(tabs, area);
}

fn render_tracker(frame: &mut Frame, app: &mut App, area: Rect) {
    let title = format!(" tracker · {}/{} ", app.visible.len(), app.scholarships.len());
    let block = Block::bordered().title(title).border_style(border());

    if app.scholarships.is_empty() {
        let hint = "No scholarships tracked yet.\n\nEvaluate one with Claude: /scholar-ops <url>";
        frame.render_widget(Paragraph::new(hint).dim().block(block), area);
        return;
    }
    if app.visible.is_empty() {
        let hint = "No rows match the filter.\n\nClear it with Esc (in filter) or toggle off.";
        frame.render_widget(Paragraph::new(hint).dim().block(block), area);
        return;
    }

    let today = app.today;
    let header = Row::new(["Name", "Level", "Deadline", "Score", "Verdict", "Status"]).bold();
    let rows: Vec<Row> = app
        .visible
        .iter()
        .map(|&i| {
            let s = &app.scholarships[i];
            let score = s.score.map(|v| format!("{v:.2}")).unwrap_or_else(|| "—".into());
            let urgency = s.deadline.urgency(today);
            let deadline = match s.deadline.days_remaining(today) {
                Some(d) => format!("{} {} {}d", s.deadline_raw, urgency.glyph(), d),
                None => format!("{} {}", s.deadline_raw, urgency.glyph()),
            };
            Row::new(vec![
                Cell::from(s.name.clone()),
                Cell::from(s.level.clone()),
                Cell::from(deadline).style(urgency_style(urgency)),
                Cell::from(score),
                Cell::from(s.verdict.label()).style(verdict_style(s.verdict)),
                Cell::from(s.status.label()),
            ])
        })
        .collect();

    let widths = [
        Constraint::Fill(3),
        Constraint::Length(9),
        Constraint::Length(18),
        Constraint::Length(6),
        Constraint::Length(11),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(Style::new().reversed())
        .block(block);

    frame.render_stateful_widget(table, area, &mut app.table);
}

fn render_detail(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::bordered().title(" detail ").border_style(border());

    let mut lines: Vec<Line> = Vec::new();
    match app.selected() {
        Some(s) => {
            lines.push(Line::from(s.name.clone()).bold());
            lines.push(Line::from(format!("{} · {}", s.provider, s.level)).dim());
            lines.push(Line::from(format!("country:  {}", s.country)));
            let glyph = s.deadline.urgency(app.today).glyph();
            lines.push(Line::from(format!("deadline: {} {}", s.deadline_raw, glyph)));
            let score = s.score.map(|v| format!("{v:.2}/5")).unwrap_or_else(|| "—".into());
            lines.push(Line::from(format!("score:    {}", score)));
            lines.push(Line::from(format!("verdict:  {}", s.verdict_raw)));
            lines.push(Line::from(format!("status:   {}", s.status.label())));
            lines.push(Line::from(format!("url:      {}", s.url)));
            lines.push(Line::from(""));
            lines.push(Line::from("── report ──").dim());
            match &app.report_body {
                Some(body) => lines.extend(body.lines().map(|l| Line::from(l.to_string()))),
                None => lines.push(Line::from("(no report file for this row)").dim()),
            }
        }
        None => lines.push(Line::from("no selection").dim()),
    }

    let detail = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll, 0));
    frame.render_widget(detail, area);
}

fn render_pipeline(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::bordered()
        .title(" pipeline inbox — run /scholar-ops pipeline to evaluate ")
        .border_style(border());

    if app.pipeline.is_empty() {
        let hint = "Inbox empty.\n\nDiscover candidates with Claude: /scholar-ops scan";
        frame.render_widget(Paragraph::new(hint).dim().block(block), area);
        return;
    }

    let items: Vec<ListItem> = app
        .pipeline
        .iter()
        .map(|p| {
            let mark = if p.done { "[x]" } else { "[ ]" };
            let deadline = if p.deadline.is_empty() { "—" } else { &p.deadline };
            let line = Line::from(vec![
                format!("{mark} ").dim(),
                p.url.clone().into(),
                format!("  · {}  · deadline {}", p.source, deadline).dim(),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::new().reversed())
        .highlight_symbol("▸ ");

    frame.render_stateful_widget(list, area, &mut app.pipeline_state);
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let text = if app.input_mode {
        format!("filter: {}_    (Enter apply · Esc clear)", app.filter_text)
    } else if let Some(m) = &app.message {
        m.clone()
    } else {
        let mut chips: Vec<String> = Vec::new();
        if !app.filter_text.is_empty() {
            chips.push(format!("text:'{}'", app.filter_text));
        }
        if app.urgent_only {
            chips.push("urgent<14d".into());
        }
        if let Some(v) = app.verdict_filter {
            chips.push(format!("verdict:{}", v.label()));
        }
        let hint =
            "↑/↓ move · PgUp/Dn scroll · Tab view · o open · s status · / filter · u urgent · v verdict · r refresh · q quit";
        if chips.is_empty() {
            hint.to_string()
        } else {
            format!("[{}]  {hint}", chips.join(" "))
        }
    };
    frame.render_widget(Line::from(text).dim(), area);
}

/// Static border color for panes.
fn border() -> Style {
    Style::new().fg(Color::DarkGray)
}

/// Color for a verdict badge (mirrors the badge palette in 07_visual_identity).
fn verdict_style(v: Verdict) -> Style {
    match v {
        Verdict::Apply => Style::new().fg(Color::Green).add_modifier(Modifier::BOLD),
        Verdict::Maybe => Style::new().fg(Color::Yellow),
        Verdict::Skip => Style::new().fg(Color::Red),
        Verdict::Ineligible => Style::new().fg(Color::Red).add_modifier(Modifier::DIM),
        Verdict::Dead => Style::new().fg(Color::DarkGray),
        Verdict::Unknown => Style::new().fg(Color::Gray),
    }
}

/// Color for a deadline by urgency (🔥 red, ⚠ yellow, passed dimmed, …).
fn urgency_style(u: Urgency) -> Style {
    match u {
        Urgency::Fire => Style::new().fg(Color::Red).add_modifier(Modifier::BOLD),
        Urgency::Warn => Style::new().fg(Color::Yellow),
        Urgency::Passed => Style::new().fg(Color::DarkGray).add_modifier(Modifier::DIM),
        Urgency::Rolling => Style::new().fg(Color::Blue),
        Urgency::Unknown => Style::new().fg(Color::DarkGray),
        Urgency::Later => Style::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::data::{parse_pipeline, parse_tracker};
    use ratatui::{Terminal, backend::TestBackend};

    /// Render one frame into an off-screen buffer and flatten it to a string.
    fn render(app: &mut App) -> String {
        let mut terminal = Terminal::new(TestBackend::new(140, 16)).unwrap();
        terminal.draw(|f| draw(f, app)).unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect()
    }

    fn one_row() -> Vec<crate::model::Scholarship> {
        parse_tracker(
            "| Name | Provider | Level | Country | Deadline | Score | Verdict | Status | Report | URL |\n\
             |---|---|---|---|---|---|---|---|---|---|\n\
             | DAAD EPOS | DAAD | masters | Germany | 2026-10-31 | 4.20 | APPLY | preparing | reports/daad-epos.md | https://daad.de/x |\n",
        )
    }

    #[test]
    fn table_shows_name_and_verdict() {
        let mut app = App::for_test(one_row());
        let out = render(&mut app);
        assert!(out.contains("DAAD EPOS"), "buffer:\n{out}");
        assert!(out.contains("APPLY"), "buffer:\n{out}");
    }

    #[test]
    fn detail_pane_shows_selected_url() {
        let mut app = App::for_test(one_row());
        let out = render(&mut app);
        assert!(out.contains("url:"), "buffer:\n{out}");
        assert!(out.contains("https://daad.de/x"), "buffer:\n{out}");
    }

    #[test]
    fn pipeline_view_lists_items() {
        let mut app = App::for_test(vec![]);
        app.pipeline = parse_pipeline(
            "- [ ] https://x.org/scholarship | X scan 2026-07-04 | deadline 2026-09-01\n",
        );
        app.view = View::Pipeline;
        let out = render(&mut app);
        assert!(out.contains("https://x.org/scholarship"), "buffer:\n{out}");
        assert!(out.contains("Pipeline (1)"), "buffer:\n{out}");
    }

    #[test]
    fn empty_state_renders_hint() {
        let mut app = App::for_test(vec![]);
        let out = render(&mut app);
        assert!(out.contains("No scholarships tracked yet"), "buffer:\n{out}");
    }
}
