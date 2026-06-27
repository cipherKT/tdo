use crate::app::AppState;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub(super) fn render_stats(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = Block::default()
        .title(" stats ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let bar_width = inner.width.saturating_sub(2);
    let mut lines: Vec<Line> = Vec::new();

    // --- global stats ---
    lines.push(Line::from(vec![Span::styled(
        "GLOBAL STATS",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));

    let g = &state.stats;
    lines.push(Line::from(format!("Total:   {}", g.total)));

    lines.push(Line::from(format!("Done:    {}", g.done)));
    lines.push(stat_bar(g.done, g.total, bar_width, Color::Green));

    lines.push(Line::from(format!("Pending: {}", g.pending)));
    lines.push(stat_bar(g.pending, g.total, bar_width, Color::Yellow));

    lines.push(Line::from(format!("Overdue: {}", g.overdue)));
    lines.push(stat_bar(g.overdue, g.total, bar_width, Color::Red));

    lines.push(Line::from(format!(
        "P1: {}  P2: {}  P3: {}",
        g.p1, g.p2, g.p3
    )));

    lines.push(Line::from(""));

    // --- project stats ---
    let header = match &state.context {
        crate::app::AppContext::Home => "SELECTED PROJECT",
        crate::app::AppContext::Project { .. } => "CURRENT PROJECT",
    };
    lines.push(Line::from(vec![Span::styled(
        header,
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]));

    let p = &state.project_stats;
    lines.push(Line::from(format!("Total:   {}", p.total)));

    lines.push(Line::from(format!("Done:    {}", p.done)));
    lines.push(stat_bar(p.done, p.total, bar_width, Color::Green));

    lines.push(Line::from(format!("Pending: {}", p.pending)));
    lines.push(stat_bar(p.pending, p.total, bar_width, Color::Yellow));

    lines.push(Line::from(format!("Overdue: {}", p.overdue)));
    lines.push(stat_bar(p.overdue, p.total, bar_width, Color::Red));

    lines.push(Line::from(format!(
        "P1: {}  P2: {}  P3: {}",
        p.p1, p.p2, p.p3
    )));

    for (i, line) in lines.into_iter().enumerate() {
        let text_y = inner.y + i as u16;
        if text_y >= inner.y + inner.height {
            break;
        }
        let row_area = Rect {
            x: inner.x + 1,
            y: text_y,
            width: bar_width,
            height: 1,
        };
        frame.render_widget(Paragraph::new(line), row_area);
    }
}

fn stat_bar(count: i64, total: i64, width: u16, color: Color) -> Line<'static> {
    if total == 0 {
        return Line::from(Span::styled(
            "─".repeat(width as usize),
            Style::default().fg(Color::DarkGray),
        ));
    }
    let filled = ((count as f64 / total as f64) * width as f64).round() as usize;
    let filled = filled.max(if count > 0 { 1 } else { 0 });
    let empty = (width as usize).saturating_sub(filled);
    Line::from(vec![
        Span::styled("█".repeat(filled), Style::default().fg(color)),
        Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
    ])
}
