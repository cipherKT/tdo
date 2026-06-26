use crate::app::{AppContext, AppState};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub(super) fn render_list(frame: &mut Frame, state: &AppState, area: Rect) {
    let title = match &state.context {
        AppContext::Home => format!("projects ({})", state.projects.len()),
        AppContext::Project { name, .. } => format!("tasks — {}", name),
    };

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let AppContext::Home = &state.context {
        for (display_pos, &real_idx) in state.filtered_projects.iter().enumerate() {
            if display_pos as u16 >= inner.height {
                break;
            }
            let project = &state.projects[real_idx];
            let is_selected = display_pos == state.selected;
            let prefix = if is_selected { "▶ " } else { "  " };
            let style = if is_selected {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            let line = Line::from(vec![Span::styled(
                format!("{}{}", prefix, project.name),
                style,
            )]);
            let row_area = Rect {
                x: inner.x,
                y: inner.y + display_pos as u16,
                width: inner.width,
                height: 1,
            };
            frame.render_widget(Paragraph::new(line), row_area);
        }
    } else {
        for (display_pos, &real_idx) in state.filtered_tasks.iter().enumerate() {
            if display_pos as u16 >= inner.height {
                break;
            }
            let task = &state.tasks[real_idx];
            let is_selected = display_pos == state.selected;
            let prefix = if is_selected { "▶ " } else { "  " };
            let style = if is_selected {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let priority_label = match task.priority {
                1 => Span::styled("P1", Style::default().fg(Color::Red)),
                2 => Span::styled("P2", Style::default().fg(Color::Yellow)),
                3 => Span::styled("P3", Style::default().fg(Color::Green)),
                _ => Span::styled("P?", Style::default().fg(Color::Gray)),
            };

            let due_str = match &task.due_date {
                Some(d) => {
                    let now = chrono::Utc::now();
                    let diff = d.signed_duration_since(now).num_days();
                    if diff < 0 {
                        format!("overdue {}d", diff.abs())
                    } else if diff == 0 {
                        "due today".to_string()
                    } else {
                        format!("due in {}d", diff)
                    }
                }
                None => "no due date".to_string(),
            };

            let done_style = if task.done {
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::CROSSED_OUT)
            } else {
                style
            };

            let line = Line::from(vec![
                Span::styled(format!("{}{}", prefix, task.name), done_style),
                Span::raw("  "),
                priority_label,
                Span::raw("  "),
                Span::styled(due_str, Style::default().fg(Color::DarkGray)),
            ]);

            let row_area = Rect {
                x: inner.x,
                y: inner.y + display_pos as u16,
                width: inner.width,
                height: 1,
            };
            frame.render_widget(Paragraph::new(line), row_area);
        }
    }
}
