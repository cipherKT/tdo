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
            } else {
                lines.push(Line::from("No task selected."));
            }
        }
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, inner);
}
