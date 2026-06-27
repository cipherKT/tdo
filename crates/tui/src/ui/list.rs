use crate::app::{AppContext, AppState};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub(super) fn render_projects_sidebar(frame: &mut Frame, state: &AppState, area: Rect) {
    let is_focused = matches!(state.context, AppContext::Home);
    let border_color = if is_focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    let block = Block::default()
        .title(" projects ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let active_project_name = match &state.context {
        AppContext::Project { name, .. } => Some(name.as_str()),
        _ => None,
    };

    for (display_pos, &real_idx) in state.filtered_projects.iter().enumerate() {
        if display_pos as u16 >= inner.height {
            break;
        }
        let project = &state.projects[real_idx];
        let is_selected_or_active = if is_focused {
            display_pos == state.selected
        } else {
            active_project_name == Some(project.name.as_str())
        };

        let prefix = if is_focused && is_selected_or_active {
            "▶ "
        } else {
            "  "
        };
        let style = if is_selected_or_active {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
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
}

pub(super) fn render_tasks_list(frame: &mut Frame, state: &AppState, area: Rect) {
    let is_focused = matches!(state.context, AppContext::Project { .. });
    let project_name = match &state.context {
        AppContext::Project { name, .. } => name.clone(),
        AppContext::Home => {
            if let Some(&proj_idx) = state.filtered_projects.get(state.selected) {
                state.projects[proj_idx].name.clone()
            } else {
                "none".to_string()
            }
        }
    };

    let border_color = if is_focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    let block = Block::default()
        .title(format!(" tasks — {} ", project_name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if state.filtered_tasks.is_empty() {
        return;
    }

    for (display_pos, &real_idx) in state.filtered_tasks.iter().enumerate() {
        if display_pos as u16 >= inner.height {
            break;
        }
        let task = &state.tasks[real_idx];
        let is_selected = is_focused && display_pos == state.selected;
        let prefix = if is_selected { "▶ " } else { "  " };
        let style = if is_selected {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let priority_label = match task.priority {
            1 => Span::styled("P1", Style::default().fg(Color::Rgb(243, 139, 168))),
            2 => Span::styled("P2", Style::default().fg(Color::Rgb(249, 226, 175))),
            3 => Span::styled("P3", Style::default().fg(Color::Rgb(166, 227, 161))),
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

pub(super) fn render_active_list(frame: &mut Frame, state: &AppState, area: Rect) {
    match &state.context {
        AppContext::Home => {
            render_projects_sidebar(frame, state, area);
        }
        AppContext::Project { .. } => {
            render_tasks_list(frame, state, area);
        }
    }
}
