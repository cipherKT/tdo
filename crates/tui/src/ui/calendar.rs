use crate::app::{AppState, RightPane};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

const WEEKDAYS: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

pub(super) fn render_calendar(frame: &mut Frame, state: &AppState, area: Rect) {
    let is_focused = state.right_pane == RightPane::Calendar;

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
        .title(Line::from(vec![Span::styled(" calendar ", title_style)]))
        .borders(Borders::ALL)
        .border_type(if is_focused {
            BorderType::Thick
        } else {
            BorderType::Plain
        })
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width < 20 || inner.height < 5 {
        return;
    }

    // Split inner area: top = calendar grid, bottom = tasks for selected day.
    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // calendar header + 6 week rows + weekday header
            Constraint::Min(0),     // day-tasks list
        ])
        .split(inner);

    render_grid(frame, state, split[0]);
    render_day_tasks(frame, state, split[1]);
}

// ---------------------------------------------------------------------------
// Calendar grid
// ---------------------------------------------------------------------------

fn render_grid(frame: &mut Frame, state: &AppState, area: Rect) {
    use chrono::{Datelike, Local, NaiveDate};

    let cal = &state.calendar;
    let is_focused = state.right_pane == RightPane::Calendar;

    let first = NaiveDate::from_ymd_opt(cal.year, cal.month, 1).unwrap();
    let offset = first.weekday().num_days_from_monday(); // padding cells at start
    let total_days = days_in_month(cal.year, cal.month);
    let total_cells = offset + total_days;
    let num_rows = (total_cells + 6) / 7;

    let today = Local::now().date_naive();

    let mut lines: Vec<Line> = Vec::new();

    // Month / year header.
    let month_name = MONTH_NAMES[(cal.month - 1) as usize];
    lines.push(Line::from(vec![Span::styled(
        format!("  {}  {}", month_name, cal.year),
        Style::default()
            .fg(state.theme.primary_accent)
            .add_modifier(Modifier::BOLD),
    )]));

    // Weekday header row.
    let mut wd_spans: Vec<Span> = Vec::new();
    for (i, wd) in WEEKDAYS.iter().enumerate() {
        let style = if i >= 5 {
            // Weekend
            Style::default()
                .fg(state.theme.secondary_accent)
                .add_modifier(Modifier::DIM)
        } else {
            Style::default()
                .fg(state.theme.label)
                .add_modifier(Modifier::DIM)
        };
        wd_spans.push(Span::styled(format!("{:>3}", wd), style));
    }
    lines.push(Line::from(wd_spans));

    // Week rows.
    for row in 0..num_rows {
        let mut row_spans: Vec<Span> = Vec::new();

        for col in 0..7u32 {
            let cell = row * 7 + col;

            if cell < offset || cell >= offset + total_days {
                // Padding cell.
                row_spans.push(Span::raw("   "));
                continue;
            }

            let day = cell - offset + 1; // 1-based

            // Fetch task count for this cell (stored in calendar state if
            // available, otherwise display a plain number).
            // We build style based on what kind of day this is.
            let is_today =
                cal.year == today.year() && cal.month == today.month() && day == today.day();
            let is_cursor = is_focused && row == cal.cursor_row && col == cal.cursor_col;
            let is_weekend = col >= 5;

            let base_style = if is_cursor {
                Style::default()
                    .fg(state.theme.highlight)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else if is_today {
                Style::default()
                    .fg(state.theme.primary_accent)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            } else if is_weekend {
                Style::default()
                    .fg(state.theme.secondary_accent)
                    .add_modifier(Modifier::DIM)
            } else {
                Style::default().fg(state.theme.value)
            };

            row_spans.push(Span::styled(format!("{:>3}", day), base_style));
        }

        lines.push(Line::from(row_spans));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

// ---------------------------------------------------------------------------
// Day-tasks pane (below the grid)
// ---------------------------------------------------------------------------

fn render_day_tasks(frame: &mut Frame, state: &AppState, area: Rect) {
    let cal = &state.calendar;
    let is_focused = state.right_pane == RightPane::Calendar;

    if area.height == 0 {
        return;
    }

    // Header showing the selected date.
    let day_label = if cal.cursor_day > 0 {
        let month_name = MONTH_NAMES[(cal.month - 1) as usize];
        format!(" {} {}, {} ", month_name, cal.cursor_day, cal.year)
    } else {
        " — ".to_string()
    };

    let mut lines: Vec<Line> = Vec::new();

    // Section separator / label.
    let sep_style = if is_focused {
        Style::default()
            .fg(state.theme.primary_accent)
            .add_modifier(Modifier::DIM)
    } else {
        Style::default()
            .fg(state.theme.label)
            .add_modifier(Modifier::DIM)
    };
    lines.push(Line::from(vec![Span::styled(day_label, sep_style)]));

    match &cal.day_tasks {
        None => {
            // Tasks not yet loaded; show placeholder.
            lines.push(Line::from(vec![Span::styled(
                "—",
                Style::default()
                    .fg(state.theme.label)
                    .add_modifier(Modifier::DIM),
            )]));
        }
        Some(tasks) if tasks.is_empty() => {
            let msg = if cal.cursor_day == 0 {
                "—"
            } else {
                "No pending tasks."
            };
            lines.push(Line::from(vec![Span::styled(
                msg,
                Style::default()
                    .fg(state.theme.label)
                    .add_modifier(Modifier::DIM),
            )]));
        }
        Some(tasks) => {
            for nt in tasks.iter().take(area.height.saturating_sub(1) as usize) {
                let priority_span = match nt.task.priority {
                    1 => Span::styled(" P1", Style::default().fg(state.theme.status_overdue)),
                    2 => Span::styled(" P2", Style::default().fg(state.theme.status_pending)),
                    3 => Span::styled(" P3", Style::default().fg(state.theme.status_done)),
                    _ => Span::styled(" P?", Style::default().fg(state.theme.label)),
                };
                lines.push(Line::from(vec![
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
                ]));
            }
        }
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn days_in_month(year: i32, month: u32) -> u32 {
    use chrono::NaiveDate;
    let (ny, nm) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    NaiveDate::from_ymd_opt(ny, nm, 1)
        .unwrap()
        .pred_opt()
        .unwrap()
        .day()
}

// Needed to suppress "import not used" from chrono::Datelike
use chrono::Datelike as _;
