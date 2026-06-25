use crate::app::{AppContext, AppMode, AppState};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn render(frame: &mut Frame, state: &AppState) {
    // --- outer vertical split: header / middle / hint bar ---
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header/breadcrumb
            Constraint::Min(0),    // main content
            Constraint::Length(1), // hint bar
        ])
        .split(frame.area());

    // --- middle horizontal split: list pane / stats pane ---
    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),     // list pane
            Constraint::Length(26), // stats pane
        ])
        .split(outer[1]);

    render_header(frame, state, outer[0]);
    render_list(frame, state, middle[0]);
    render_stats(frame, state, middle[1]);
    render_hint_bar(frame, state, outer[2]);
}

fn render_header(frame: &mut Frame, state: &AppState, area: ratatui::layout::Rect) {
    let breadcrumb = match &state.context {
        AppContext::Home => " tdo  ~  home".to_string(),
        AppContext::Project { name, .. } => format!(" tdo  ~  home  /  {}", name),
    };
    let p = Paragraph::new(breadcrumb).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(p, area);
}

fn render_list(frame: &mut Frame, state: &AppState, area: ratatui::layout::Rect) {
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

    // render project rows
    if let AppContext::Home = &state.context {
        for (i, project) in state.projects.iter().enumerate() {
            if i as u16 >= inner.height {
                break;
            }
            let is_selected = i == state.selected;
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
            let row_area = ratatui::layout::Rect {
                x: inner.x,
                y: inner.y + i as u16,
                width: inner.width,
                height: 1,
            };
            frame.render_widget(Paragraph::new(line), row_area);
        }
    } else {
        for (i, task) in state.tasks.iter().enumerate() {
            if i as u16 >= inner.height {
                break;
            }
            let is_selected = i == state.selected;
            let prefix = if is_selected { "▶ " } else { "  " };
            let style = if is_selected {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            // --- replace the old single-span line with this ---
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
            // --- end replacement ---

            let row_area = ratatui::layout::Rect {
                x: inner.x,
                y: inner.y + i as u16,
                width: inner.width,
                height: 1,
            };
            frame.render_widget(Paragraph::new(line), row_area);
        }
    }
}

fn render_stats(frame: &mut Frame, state: &AppState, area: ratatui::layout::Rect) {
    let block = Block::default()
        .title(" stats ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(block, area);
}

fn render_hint_bar(frame: &mut Frame, state: &AppState, area: ratatui::layout::Rect) {
    let hint = match &state.mode {
        AppMode::Browsing => "  j/k move  ·  enter open  ·  / search  ·  q quit",
        AppMode::Search { .. } => "  type to search  ·  enter select/create  ·  esc cancel",
    };
    let p = Paragraph::new(hint).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(p, area);
}
