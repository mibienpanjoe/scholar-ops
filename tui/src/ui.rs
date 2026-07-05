//! Rendering. Pure functions of `&App` (plus `&mut` where a widget needs its
//! scroll/selection state). No IO, no evaluation — just paint what's in memory.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Cell, Paragraph, Row, Table, Wrap},
};

use crate::app::{App, Focus};
use crate::model::{Urgency, Verdict};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let [body, footer] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).areas(body);
    render_tracker(frame, app, left);
    render_detail(frame, app, right);
    render_footer(frame, app, footer);
}

/// Border style for a pane: highlighted when it holds keyboard focus.
fn pane_border(active: bool) -> Style {
    if active {
        Style::new().fg(Color::Cyan)
    } else {
        Style::new().fg(Color::DarkGray)
    }
}

fn render_tracker(frame: &mut Frame, app: &mut App, area: Rect) {
    let title = format!(" scholar-ops · {} tracked ", app.scholarships.len());
    let block = Block::bordered()
        .title(title)
        .border_style(pane_border(app.focus == Focus::Table));

    if app.scholarships.is_empty() {
        let hint = "No scholarships tracked yet.\n\nEvaluate one with Claude: /scholar-ops <url>";
        frame.render_widget(Paragraph::new(hint).dim().block(block), area);
        return;
    }

    let today = app.today;
    let header = Row::new(["Name", "Level", "Deadline", "Score", "Verdict", "Status"]).bold();
    let rows: Vec<Row> = app
        .scholarships
        .iter()
        .map(|s| {
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
    let block = Block::bordered()
        .title(" detail ")
        .border_style(pane_border(app.focus == Focus::Detail));

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

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let text = app
        .message
        .clone()
        .unwrap_or_else(|| "↑/↓ move · Tab focus · r refresh · q quit".to_string());
    frame.render_widget(Line::from(text).dim(), area);
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
    use crate::data::parse_tracker;
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
    fn empty_state_renders_hint() {
        let mut app = App::for_test(vec![]);
        let out = render(&mut app);
        assert!(out.contains("No scholarships tracked yet"), "buffer:\n{out}");
    }
}
