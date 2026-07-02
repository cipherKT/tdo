use crate::app::{AppMode, AppState};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

mod analytics;
mod calendar;
mod header;
mod hint;
mod list;
mod metadata;

pub fn render(frame: &mut Frame, state: &AppState) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(outer[1]);

    let right_panes = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(middle[2]);

    header::render_header(frame, state, outer[0]);
    metadata::render_metadata(frame, state, middle[0]);
    list::render_active_list(frame, state, middle[1]);
    analytics::render_analytics(frame, state, right_panes[0]);
    // Bottom-right pane: calendar or pending-today depending on focus.
    if state.right_pane == crate::app::RightPane::Calendar {
        calendar::render_calendar(frame, state, right_panes[1]);
    } else {
        analytics::render_pending_today(frame, state, right_panes[1]);
    }
    hint::render_hint_bar(frame, state, outer[2]);

    if let AppMode::MultiStepForm {
        kind,
        step,
        answers,
        current_input,
        warning,
        in_insert_mode,
        ..
    } = &state.mode
    {
        render_form_modal(
            frame,
            kind,
            *step,
            answers,
            current_input,
            warning,
            *in_insert_mode,
            &state.theme,
        );
    }
}

fn render_form_modal(
    frame: &mut Frame,
    kind: &crate::app::FormKind,
    step: usize,
    answers: &[String],
    current_input: &str,
    warning: &Option<String>,
    in_insert_mode: bool,
    theme: &crate::theme::Theme,
) {
    let width = 64;
    let area_width = width.min(frame.area().width);
    let inner_width = area_width.saturating_sub(4) as usize;

    let mut lines = Vec::new();

    let heading = match kind {
        crate::app::FormKind::CreateProject => "✦  CREATE NEW PROJECT  ✦".to_string(),
        crate::app::FormKind::CreateTask => "✦  CREATE NEW TASK  ✦".to_string(),
        crate::app::FormKind::ModifyProject { original_name } => {
            format!("✦  EDIT PROJECT: {}  ✦", original_name)
        }
        crate::app::FormKind::ModifyTask { original_name } => {
            format!("✦  EDIT TASK: {}  ✦", original_name)
        }
    };
    lines.push(Line::from(vec![Span::styled(
        heading,
        Style::default()
            .fg(theme.primary_accent)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from("─".repeat(inner_width)));
    lines.push(Line::from(""));

    let total_steps = crate::app::form_total_steps(kind);
    for idx in 0..total_steps {
        let label = crate::app::form_prompt(kind, idx);
        let value = if idx == step && in_insert_mode {
            format!("{}▌", current_input)
        } else {
            answers.get(idx).cloned().unwrap_or_default()
        };

        let is_selected = idx == step;
        let prefix = if is_selected { "▶ " } else { "  " };

        let field_style = if is_selected {
            if in_insert_mode {
                Style::default()
                    .fg(theme.secondary_accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(theme.highlight)
                    .add_modifier(Modifier::BOLD)
            }
        } else {
            Style::default().fg(theme.label)
        };

        let val_style = if is_selected && in_insert_mode {
            Style::default().fg(theme.secondary_accent)
        } else {
            Style::default().fg(theme.value)
        };

        let val_width = inner_width.saturating_sub(19);
        let wrapped_val = wrap_text(&value, val_width);

        lines.push(Line::from(vec![
            Span::styled(prefix, field_style),
            Span::styled(format!("{:<15}: ", label), field_style),
            Span::styled(wrapped_val[0].clone(), val_style),
        ]));

        for line in wrapped_val.iter().skip(1) {
            lines.push(Line::from(vec![
                Span::styled(" ".repeat(19), field_style),
                Span::styled(line.clone(), val_style),
            ]));
        }
    }

    lines.push(Line::from(""));

    if let Some(warn) = warning {
        lines.push(Line::from(vec![Span::styled(
            format!(" ⚠️  {}", warn),
            Style::default()
                .fg(theme.status_overdue)
                .add_modifier(Modifier::BOLD),
        )]));
    } else {
        lines.push(Line::from(""));
    }

    lines.push(Line::from("─".repeat(inner_width)));

    let footer_text = if in_insert_mode {
        match (kind, step) {
            (_, 2) => "[Insert]  Esc: accept  ·  Enter tags starting with #, separated by space",
            (crate::app::FormKind::CreateTask | crate::app::FormKind::ModifyTask { .. }, 3) => {
                "[Insert]  Esc: accept  ·  Enter priority (1, 2, or 3)"
            }
            (crate::app::FormKind::CreateTask | crate::app::FormKind::ModifyTask { .. }, 4) => {
                "[Insert]  Esc: accept  ·  e.g. today, tomorrow, +3, +1w, mon, 07-04, 15"
            }
            _ => "[Insert Mode]  Esc: back to normal  ·  Type to edit",
        }
    } else {
        "[Normal Mode]  j/k: navigate  ·  i: edit field  ·  Enter: save  ·  Esc: cancel"
    };
    lines.push(Line::from(vec![Span::styled(
        footer_text,
        Style::default().fg(theme.label).add_modifier(Modifier::DIM),
    )]));

    let height = (lines.len() as u16) + 2;
    let area = centered_rect_fixed(width, height, frame.area());

    frame.render_widget(Clear, area);

    let border_color = if in_insert_mode {
        theme.secondary_accent
    } else {
        theme.border_active
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(border_color));

    frame.render_widget(block, area);

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn centered_rect_fixed(width: u16, height: u16, r: Rect) -> Rect {
    let x = r.x + r.width.saturating_sub(width) / 2;
    let y = r.y + r.height.saturating_sub(height) / 2;
    Rect {
        x,
        y,
        width: width.min(r.width),
        height: height.min(r.height),
    }
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }
    if max_width == 0 {
        return vec![text.to_string()];
    }
    let mut lines = Vec::new();
    let mut current_line = String::new();

    let mut chars = text.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c == '\n' || c == '\r' {
            let _ = chars.next();
            if c == '\r' && chars.peek() == Some(&'\n') {
                let _ = chars.next();
            }
            lines.push(current_line);
            current_line = String::new();
        } else if c.is_whitespace() {
            let mut ws = String::new();
            while let Some(&next_c) = chars.peek() {
                if next_c.is_whitespace() && next_c != '\n' && next_c != '\r' {
                    ws.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            if current_line.len() + ws.len() <= max_width {
                current_line.push_str(&ws);
            } else {
                lines.push(current_line);
                current_line = ws;
            }
        } else {
            let mut word = String::new();
            while let Some(&next_c) = chars.peek() {
                if !next_c.is_whitespace() {
                    word.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            if word.len() > max_width {
                if !current_line.is_empty() {
                    lines.push(current_line);
                    current_line = String::new();
                }
                let mut word_chars = word.chars();
                loop {
                    let chunk: String = word_chars.by_ref().take(max_width).collect();
                    if chunk.is_empty() {
                        break;
                    }
                    if chunk.len() == max_width {
                        lines.push(chunk);
                    } else {
                        current_line = chunk;
                    }
                }
            } else if current_line.len() + word.len() <= max_width {
                current_line.push_str(&word);
            } else {
                lines.push(current_line);
                current_line = word;
            }
        }
    }
    if !current_line.is_empty() || lines.is_empty() {
        lines.push(current_line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_text_basic() {
        let text = "hello world";
        let wrapped = wrap_text(text, 7);
        assert_eq!(wrapped, vec!["hello ", "world"]);
    }

    #[test]
    fn test_wrap_text_no_split() {
        let text = "hello";
        let wrapped = wrap_text(text, 10);
        assert_eq!(wrapped, vec!["hello"]);
    }

    #[test]
    fn test_wrap_text_long_word() {
        let text = "supercalifragilistic";
        let wrapped = wrap_text(text, 5);
        assert_eq!(wrapped, vec!["super", "calif", "ragil", "istic"]);
    }

    #[test]
    fn test_wrap_text_newlines() {
        let text = "hello\nworld\r\nnext";
        let wrapped = wrap_text(text, 10);
        assert_eq!(wrapped, vec!["hello", "world", "next"]);
    }

    #[test]
    fn test_wrap_text_spaces() {
        let text = "hello   world  ";
        let wrapped = wrap_text(text, 8);
        assert_eq!(wrapped, vec!["hello   ", "world  "]);
    }
}
