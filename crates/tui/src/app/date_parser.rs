use chrono::{Datelike, Days, Months, NaiveDate, Weekday};

pub fn parse_due_date(input: &str, today: NaiveDate) -> Result<NaiveDate, String> {
    let s = input.trim().to_lowercase();
    if s.is_empty() {
        return Err("Empty input".to_string());
    }

    // 1. Try standard YYYY-MM-DD
    if let Ok(d) = NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
        return Ok(d);
    }
    // Also support YYYY/MM/DD
    if let Ok(d) = NaiveDate::parse_from_str(&s, "%Y/%m/%d") {
        return Ok(d);
    }

    // 2. Today / Tomorrow shortcuts
    if s == "today" || s == "tod" || s == "t" {
        return Ok(today);
    }
    if s == "tomorrow" || s == "tom" || s == "tm" {
        return Ok(today + Days::new(1));
    }

    // 3. Relative offsets: +N, +Nw, +Nm, +Nd
    if s.starts_with('+') {
        let suffix = s.trim_start_matches('+');
        if let Some(n_str) = suffix.strip_suffix('w') {
            if let Ok(n) = n_str.parse::<u64>() {
                return Ok(today + Days::new(n * 7));
            }
        } else if let Some(n_str) = suffix.strip_suffix('m') {
            if let Some(d) = n_str.parse::<u32>().ok().and_then(|n| today.checked_add_months(Months::new(n))) {
                return Ok(d);
            }
        } else if let Some(n_str) = suffix.strip_suffix('d') {
            if let Ok(n) = n_str.parse::<u64>() {
                return Ok(today + Days::new(n));
            }
        } else if let Ok(n) = suffix.parse::<u64>() {
            return Ok(today + Days::new(n));
        }
    }

    // 4. Weekdays: next occurrence (strictly future: 1 to 7 days out)
    let weekday_opt = match s.as_str() {
        "mon" | "monday" => Some(Weekday::Mon),
        "tue" | "tuesday" => Some(Weekday::Tue),
        "wed" | "wednesday" => Some(Weekday::Wed),
        "thu" | "thursday" => Some(Weekday::Thu),
        "fri" | "friday" => Some(Weekday::Fri),
        "sat" | "saturday" => Some(Weekday::Sat),
        "sun" | "sunday" => Some(Weekday::Sun),
        _ => None,
    };
    if let Some(target_wd) = weekday_opt {
        let mut d = today + Days::new(1);
        for _ in 0..7 {
            if d.weekday() == target_wd {
                return Ok(d);
            }
            d = d + Days::new(1);
        }
    }

    // 5. Partial dates: MM-DD or MM/DD (e.g. 12-25, 7/4)
    let parts: Vec<&str> = if s.contains('-') {
        s.split('-').collect()
    } else if s.contains('/') {
        s.split('/').collect()
    } else {
        Vec::new()
    };

    if parts.len() == 2
        && let Some(date) = parts[0].parse::<u32>().ok()
            .and_then(|m| parts[1].parse::<u32>().ok().and_then(|d| NaiveDate::from_ymd_opt(today.year(), m, d)))
    {
        return Ok(date);
    }

    // 6. Just DD (day of month)
    if let Ok(d) = s.parse::<u32>() {
        let year = today.year();
        let month = today.month();
        // If the day is >= today's day, use current month, else next month
        if d >= today.day() {
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, d) {
                return Ok(date);
            }
        } else {
            // Next month
            let mut next_month = month + 1;
            let mut next_year = year;
            if next_month > 12 {
                next_month = 1;
                next_year += 1;
            }
            if let Some(date) = NaiveDate::from_ymd_opt(next_year, next_month, d) {
                return Ok(date);
            }
        }
    }

    Err(
        "Invalid date format. Use YYYY-MM-DD, today, tomorrow, +N, +Nw, weekday, MM-DD, or DD"
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_due_date() {
        // Assume today is Tuesday, June 30, 2026
        let today = NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();

        // 1. Standard YYYY-MM-DD
        assert_eq!(
            parse_due_date("2026-07-01", today),
            Ok(NaiveDate::from_ymd_opt(2026, 7, 1).unwrap())
        );
        assert_eq!(
            parse_due_date("2026/07/04", today),
            Ok(NaiveDate::from_ymd_opt(2026, 7, 4).unwrap())
        );

        // 2. Today / Tomorrow shortcuts
        assert_eq!(parse_due_date("today", today), Ok(today));
        assert_eq!(parse_due_date("tod", today), Ok(today));
        assert_eq!(parse_due_date("t", today), Ok(today));
        assert_eq!(
            parse_due_date("tomorrow", today),
            Ok(NaiveDate::from_ymd_opt(2026, 7, 1).unwrap())
        );
        assert_eq!(
            parse_due_date("tm", today),
            Ok(NaiveDate::from_ymd_opt(2026, 7, 1).unwrap())
        );

        // 3. Relative offsets
        assert_eq!(
            parse_due_date("+1", today),
            Ok(NaiveDate::from_ymd_opt(2026, 7, 1).unwrap())
        );
        assert_eq!(
            parse_due_date("+7", today),
            Ok(NaiveDate::from_ymd_opt(2026, 7, 7).unwrap())
        );
        assert_eq!(
            parse_due_date("+1w", today),
            Ok(NaiveDate::from_ymd_opt(2026, 7, 7).unwrap())
        );
        assert_eq!(
            parse_due_date("+1m", today),
            Ok(NaiveDate::from_ymd_opt(2026, 7, 30).unwrap())
        );
        assert_eq!(
            parse_due_date("+2d", today),
            Ok(NaiveDate::from_ymd_opt(2026, 7, 2).unwrap())
        );

        // 4. Weekdays (strictly future)
        // Today is Tuesday, June 30, 2026
        // Wednesday is tomorrow (July 1)
        assert_eq!(
            parse_due_date("wed", today),
            Ok(NaiveDate::from_ymd_opt(2026, 7, 1).unwrap())
        );
        assert_eq!(
            parse_due_date("wednesday", today),
            Ok(NaiveDate::from_ymd_opt(2026, 7, 1).unwrap())
        );
        // Next Monday is July 6
        assert_eq!(
            parse_due_date("mon", today),
            Ok(NaiveDate::from_ymd_opt(2026, 7, 6).unwrap())
        );
        // Next Tuesday is July 7 (strictly future)
        assert_eq!(
            parse_due_date("tuesday", today),
            Ok(NaiveDate::from_ymd_opt(2026, 7, 7).unwrap())
        );

        // 5. Partial dates
        assert_eq!(
            parse_due_date("07-04", today),
            Ok(NaiveDate::from_ymd_opt(2026, 7, 4).unwrap())
        );
        assert_eq!(
            parse_due_date("7/4", today),
            Ok(NaiveDate::from_ymd_opt(2026, 7, 4).unwrap())
        );

        // 6. Just DD (day of month)
        // Today is June 30.
        // 30 is today
        assert_eq!(
            parse_due_date("30", today),
            Ok(NaiveDate::from_ymd_opt(2026, 6, 30).unwrap())
        );
        // 5 is in the past for June, so it defaults to next month (July 5)
        assert_eq!(
            parse_due_date("5", today),
            Ok(NaiveDate::from_ymd_opt(2026, 7, 5).unwrap())
        );
        assert_eq!(
            parse_due_date("05", today),
            Ok(NaiveDate::from_ymd_opt(2026, 7, 5).unwrap())
        );
    }
}
