use crate::app::AppState;
use crate::theme::Theme;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub(super) fn render_analytics(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = Block::default()
        .title(" stats ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(state.theme.border_inactive));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let mut lines = Vec::new();

    // Title for global stats section
    lines.push(Line::from(vec![Span::styled(
        "GLOBAL STATS",
        Style::default()
            .fg(state.theme.label)
            .add_modifier(Modifier::DIM),
    )]));

    let s = &state.stats;
    lines.push(render_thin_progress_bar(
        s.done,
        s.pending,
        s.overdue,
        inner.width as usize,
        &state.theme,
    ));

    lines.push(Line::from(vec![
        Span::styled(
            format!("Done: {} ", s.done),
            Style::default().fg(state.theme.status_done),
        ),
        Span::styled(
            format!("Pend: {} ", s.pending),
            Style::default().fg(state.theme.status_pending),
        ),
        Span::styled(
            format!("Overdue: {}", s.overdue),
            Style::default().fg(state.theme.status_overdue),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Total Tasks: ", Style::default().fg(state.theme.label)),
        Span::styled(
            s.total.to_string(),
            Style::default()
                .fg(state.theme.primary_accent)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));

    // Section for task distribution
    lines.push(Line::from(vec![Span::styled(
        "TASKS PER PROJECT",
        Style::default()
            .fg(state.theme.label)
            .add_modifier(Modifier::DIM),
    )]));

    if state.projects_task_counts.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "No projects found.",
            Style::default().fg(state.theme.label),
        )]));
    } else {
        let max_count = state
            .projects_task_counts
            .iter()
            .map(|(_, c)| *c)
            .max()
            .unwrap_or(0);

        for (name, count) in &state.projects_task_counts {
            // Allocate 12 columns for project name, truncated if needed
            let name_label = if name.len() > 12 {
                format!("{}…", &name[0..11])
            } else {
                format!("{:<12}", name)
            };

            let bar_chars = if max_count > 0 && *count > 0 {
                // Determine bar size (leave room for name and count/padding)
                let max_bar_width = (inner.width as usize).saturating_sub(20).max(4);
                let bar_len =
                    ((*count as f64 / max_count as f64) * max_bar_width as f64).round() as usize;
                "█".repeat(bar_len.max(1))
            } else {
                String::new()
            };

            lines.push(Line::from(vec![
                Span::styled(name_label, Style::default().fg(state.theme.primary_accent)),
                Span::styled(" ", Style::default()),
                Span::styled(bar_chars, Style::default().fg(state.theme.secondary_accent)),
                Span::styled(
                    format!(" ({})", count),
                    Style::default().fg(state.theme.label),
                ),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, inner);
}

fn render_thin_progress_bar(
    done: i64,
    pending: i64,
    overdue: i64,
    width: usize,
    theme: &Theme,
) -> Line<'static> {
    let total = done + pending + overdue;
    if total == 0 {
        return Line::from(vec![Span::styled(
            "─".repeat(width),
            Style::default().fg(theme.label),
        )]);
    }

    let done_frac = done as f64 / total as f64;
    let pending_frac = pending as f64 / total as f64;

    let done_blocks = (done_frac * width as f64).round() as usize;
    let pending_blocks = (pending_frac * width as f64).round() as usize;
    let overdue_blocks = width
        .saturating_sub(done_blocks)
        .saturating_sub(pending_blocks);

    let mut done_blocks = done_blocks;
    let mut pending_blocks = pending_blocks;
    let mut overdue_blocks = overdue_blocks;

    if done > 0 && done_blocks == 0 && width >= 3 {
        done_blocks = 1;
    }
    if pending > 0 && pending_blocks == 0 && width >= 3 {
        pending_blocks = 1;
    }
    if overdue > 0 && overdue_blocks == 0 && width >= 3 {
        overdue_blocks = 1;
    }

    let total_blocks = done_blocks + pending_blocks + overdue_blocks;
    if total_blocks != width {
        if done_blocks > 0 && total_blocks > width {
            done_blocks = done_blocks.saturating_sub(total_blocks - width);
        } else if total_blocks < width {
            done_blocks += width - total_blocks;
        }
    }

    Line::from(vec![
        Span::styled(
            "▂".repeat(done_blocks),
            Style::default().fg(theme.status_done),
        ),
        Span::styled(
            "▂".repeat(pending_blocks),
            Style::default().fg(theme.status_pending),
        ),
        Span::styled(
            "▂".repeat(overdue_blocks),
            Style::default().fg(theme.status_overdue),
        ),
    ])
}

pub(super) fn render_pending_today(frame: &mut Frame, state: &AppState, area: Rect) {
    use crate::app::RightPane;
    let is_focused = state.right_pane == RightPane::PendingToday;

    let border_color = if is_focused {
        state.theme.border_active
    } else {
        state.theme.border_inactive
    };

    let title_style = if is_focused {
        Style::default()
            .fg(state.theme.primary_accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(state.theme.label)
    };

    let block = Block::default()
        .title(Line::from(vec![Span::styled(" pending today ", title_style)]))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    if state.pending_today.is_empty() {
        let empty_msg = Paragraph::new(Line::from(vec![Span::styled(
            "No pending tasks for today.",
            Style::default().fg(state.theme.label),
        )]));
        frame.render_widget(empty_msg, inner);
        return;
    }

    let mut lines = Vec::new();
    for (idx, nt) in state.pending_today.iter().enumerate() {
        if idx as u16 >= inner.height {
            break;
        }
        let priority_span = match nt.task.priority {
            1 => Span::styled(" P1", Style::default().fg(state.theme.status_overdue)),
            2 => Span::styled(" P2", Style::default().fg(state.theme.status_pending)),
            3 => Span::styled(" P3", Style::default().fg(state.theme.status_done)),
            _ => Span::styled(" P4", Style::default().fg(state.theme.label)),
        };

        let line = Line::from(vec![
            Span::styled("• ", Style::default().fg(state.theme.label)),
            Span::styled(
                nt.task.name.clone(),
                Style::default().fg(state.theme.highlight),
            ),
            Span::styled(
                format!(" ({})", nt.project_name),
                Style::default().fg(state.theme.primary_accent),
            ),
            priority_span,
        ]);
        lines.push(line);
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}
