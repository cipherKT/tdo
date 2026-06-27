use crate::app::{AppContext, AppState};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub(super) fn render_metadata(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = Block::default()
        .title(" metadata ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let mut lines: Vec<Line> = Vec::new();

    match &state.context {
        AppContext::Home => {
            if let Some(&proj_idx) = state.filtered_projects.get(state.selected) {
                let project = &state.projects[proj_idx];

                lines.push(Line::from(vec![Span::styled(
                    "PROJECT",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                )]));
                lines.push(Line::from(vec![Span::styled(
                    &project.name,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )]));
                lines.push(Line::from(""));

                lines.push(Line::from(vec![Span::styled(
                    "DESCRIPTION",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                )]));
                if project.description.is_empty() {
                    lines.push(Line::from(vec![Span::styled(
                        "No description.",
                        Style::default().fg(Color::DarkGray),
                    )]));
                } else {
                    lines.push(Line::from(project.description.as_str()));
                }
                lines.push(Line::from(""));

                lines.push(Line::from(vec![Span::styled(
                    "TAGS",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                )]));
                if state.selected_item_tags.is_empty() {
                    lines.push(Line::from(vec![Span::styled(
                        "No tags.",
                        Style::default().fg(Color::DarkGray),
                    )]));
                } else {
                    let tags_line: Vec<Span> = state
                        .selected_item_tags
                        .iter()
                        .map(|t| {
                            Span::styled(format!("#{} ", t), Style::default().fg(Color::Magenta))
                        })
                        .collect();
                    lines.push(Line::from(tags_line));
                }
                lines.push(Line::from(""));

                lines.push(Line::from(vec![Span::styled(
                    "PROGRESS",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                )]));
                let p = &state.project_stats;
                lines.push(render_thin_progress_bar(
                    p.done,
                    p.pending,
                    p.overdue,
                    inner.width as usize,
                ));
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("Done: {} ", p.done),
                        Style::default().fg(Color::Rgb(166, 227, 161)),
                    ),
                    Span::styled(
                        format!("Pend: {} ", p.pending),
                        Style::default().fg(Color::Rgb(249, 226, 175)),
                    ),
                    Span::styled(
                        format!("Overdue: {}", p.overdue),
                        Style::default().fg(Color::Rgb(243, 139, 168)),
                    ),
                ]));
            } else {
                lines.push(Line::from("No project selected."));
            }
        }
        AppContext::Project { name, .. } => {
            if let Some(&task_idx) = state.filtered_tasks.get(state.selected) {
                let task = &state.tasks[task_idx];

                lines.push(Line::from(vec![
                    Span::styled(
                        "PROJECT: ",
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::DIM),
                    ),
                    Span::styled(name, Style::default().fg(Color::Cyan)),
                ]));
                lines.push(Line::from(""));

                lines.push(Line::from(vec![Span::styled(
                    "TASK",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                )]));
                lines.push(Line::from(vec![Span::styled(
                    &task.name,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )]));
                lines.push(Line::from(""));

                lines.push(Line::from(vec![Span::styled(
                    "DESCRIPTION",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                )]));
                if task.description.is_empty() {
                    lines.push(Line::from(vec![Span::styled(
                        "No description.",
                        Style::default().fg(Color::DarkGray),
                    )]));
                } else {
                    lines.push(Line::from(task.description.as_str()));
                }
                lines.push(Line::from(""));

                lines.push(Line::from(vec![
                    Span::styled(
                        "PRIORITY: ",
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::DIM),
                    ),
                    match task.priority {
                        1 => Span::styled(
                            "P1 (High)",
                            Style::default().fg(Color::Rgb(243, 139, 168)),
                        ),
                        2 => Span::styled(
                            "P2 (Medium)",
                            Style::default().fg(Color::Rgb(249, 226, 175)),
                        ),
                        3 => {
                            Span::styled("P3 (Low)", Style::default().fg(Color::Rgb(166, 227, 161)))
                        }
                        _ => Span::styled("P?", Style::default().fg(Color::Gray)),
                    },
                ]));

                let due_str = match &task.due_date {
                    Some(d) => d.format("%Y-%m-%d").to_string(),
                    None => "None".to_string(),
                };
                lines.push(Line::from(vec![
                    Span::styled(
                        "DUE DATE: ",
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::DIM),
                    ),
                    Span::raw(due_str),
                ]));
                lines.push(Line::from(""));

                lines.push(Line::from(vec![Span::styled(
                    "TAGS",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                )]));
                if state.selected_item_tags.is_empty() {
                    lines.push(Line::from(vec![Span::styled(
                        "No tags.",
                        Style::default().fg(Color::DarkGray),
                    )]));
                } else {
                    let tags_line: Vec<Span> = state
                        .selected_item_tags
                        .iter()
                        .map(|t| {
                            Span::styled(format!("#{} ", t), Style::default().fg(Color::Magenta))
                        })
                        .collect();
                    lines.push(Line::from(tags_line));
                }
                lines.push(Line::from(""));

                lines.push(Line::from(vec![Span::styled(
                    "PROJECT PROGRESS",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                )]));
                let p = &state.project_stats;
                lines.push(render_thin_progress_bar(
                    p.done,
                    p.pending,
                    p.overdue,
                    inner.width as usize,
                ));
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("Done: {} ", p.done),
                        Style::default().fg(Color::Rgb(166, 227, 161)),
                    ),
                    Span::styled(
                        format!("Pend: {} ", p.pending),
                        Style::default().fg(Color::Rgb(249, 226, 175)),
                    ),
                    Span::styled(
                        format!("Overdue: {}", p.overdue),
                        Style::default().fg(Color::Rgb(243, 139, 168)),
                    ),
                ]));
            } else {
                lines.push(Line::from("No task selected."));
            }
        }
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, inner);
}

fn render_thin_progress_bar(done: i64, pending: i64, overdue: i64, width: usize) -> Line<'static> {
    let total = done + pending + overdue;
    if total == 0 {
        return Line::from(vec![Span::styled(
            "─".repeat(width),
            Style::default().fg(Color::DarkGray),
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
            Style::default().fg(Color::Rgb(166, 227, 161)),
        ),
        Span::styled(
            "▂".repeat(pending_blocks),
            Style::default().fg(Color::Rgb(249, 226, 175)),
        ),
        Span::styled(
            "▂".repeat(overdue_blocks),
            Style::default().fg(Color::Rgb(243, 139, 168)),
        ),
    ])
}
