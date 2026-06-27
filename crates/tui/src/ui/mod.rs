use crate::app::{AppMode, AppState};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

mod analytics;
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

    header::render_header(frame, state, outer[0]);
    metadata::render_metadata(frame, state, middle[0]);
    list::render_active_list(frame, state, middle[1]);
    analytics::render_analytics(frame, state, middle[2]);
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
    let height = 15;
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
    lines.push(Line::from("─".repeat(inner.width as usize)));
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

        lines.push(Line::from(vec![
            Span::styled(prefix, field_style),
            Span::styled(format!("{:<15}: ", label), field_style),
            Span::styled(
                value,
                if is_selected && in_insert_mode {
                    Style::default().fg(theme.secondary_accent)
                } else {
                    Style::default().fg(theme.value)
                },
            ),
        ]));
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

    lines.push(Line::from("─".repeat(inner.width as usize)));

    let footer_text = if in_insert_mode {
        "[Insert Mode]  Esc: back to normal  ·  Type to edit"
    } else {
        "[Normal Mode]  j/k: navigate  ·  i: edit field  ·  Enter: save  ·  Esc: cancel"
    };
    lines.push(Line::from(vec![Span::styled(
        footer_text,
        Style::default().fg(theme.label).add_modifier(Modifier::DIM),
    )]));

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
