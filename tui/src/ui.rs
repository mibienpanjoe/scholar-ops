//! Rendering. Pure functions of `&App` (plus `&mut` where a widget needs its
//! scroll/selection state). No IO, no evaluation — just paint what's in memory.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Paragraph, Row, Table},
};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let [body, footer] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());
    render_tracker(frame, app, body);
    render_footer(frame, app, footer);
}

fn render_tracker(frame: &mut Frame, app: &mut App, area: Rect) {
    let title = format!(" scholar-ops · {} tracked ", app.scholarships.len());
    let block = Block::bordered().title(title);

    if app.scholarships.is_empty() {
        let hint = "No scholarships tracked yet.\n\nEvaluate one with Claude: /scholar-ops <url>";
        frame.render_widget(Paragraph::new(hint).dim().block(block), area);
        return;
    }

    let header = Row::new(["Name", "Level", "Deadline", "Score", "Verdict", "Status"]).bold();
    let rows: Vec<Row> = app
        .scholarships
        .iter()
        .map(|s| {
            let score = s.score.map(|v| format!("{v:.2}")).unwrap_or_else(|| "—".into());
            Row::new([
                s.name.clone(),
                s.level.clone(),
                s.deadline_raw.clone(),
                score,
                s.verdict.label().to_string(),
                s.status.label().to_string(),
            ])
        })
        .collect();

    let widths = [
        Constraint::Fill(3),
        Constraint::Length(9),
        Constraint::Length(12),
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

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let text = app
        .message
        .clone()
        .unwrap_or_else(|| "↑/↓ move · r refresh · q quit".to_string());
    frame.render_widget(Line::from(text).dim(), area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::data::parse_tracker;
    use ratatui::{Terminal, backend::TestBackend};

    /// Render one frame into an off-screen buffer and flatten it to a string.
    fn render(app: &mut App) -> String {
        let mut terminal = Terminal::new(TestBackend::new(90, 12)).unwrap();
        terminal.draw(|f| draw(f, app)).unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect()
    }

    #[test]
    fn table_shows_name_and_verdict() {
        let rows = parse_tracker(
            "| Name | Provider | Level | Country | Deadline | Score | Verdict | Status | Report | URL |\n\
             |---|---|---|---|---|---|---|---|---|---|\n\
             | DAAD EPOS | DAAD | masters | Germany | 2026-10-31 | 4.20 | APPLY | preparing | reports/daad-epos.md | https://daad.de/x |\n",
        );
        let mut app = App::for_test(rows);
        let out = render(&mut app);
        assert!(out.contains("DAAD EPOS"), "buffer:\n{out}");
        assert!(out.contains("APPLY"), "buffer:\n{out}");
    }

    #[test]
    fn empty_state_renders_hint() {
        let mut app = App::for_test(vec![]);
        let out = render(&mut app);
        assert!(out.contains("No scholarships tracked yet"), "buffer:\n{out}");
    }
}
