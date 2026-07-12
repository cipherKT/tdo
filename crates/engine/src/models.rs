use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Recurrence {
    Daily,
    Weekly,
    Biweekly,
    Triweekly,
    Monthly,
    Bimonthly,
    Yearly,
}

impl Recurrence {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().trim() {
            "daily" | "d" => Some(Recurrence::Daily),
            "weekly" | "w" => Some(Recurrence::Weekly),
            "biweekly" => Some(Recurrence::Biweekly),
            "triweekly" => Some(Recurrence::Triweekly),
            "monthly" | "m" => Some(Recurrence::Monthly),
            "bimonthly" => Some(Recurrence::Bimonthly),
            "yearly" | "y" => Some(Recurrence::Yearly),
            _ => None,
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            Recurrence::Daily => "daily",
            Recurrence::Weekly => "weekly",
            Recurrence::Biweekly => "biweekly",
            Recurrence::Triweekly => "triweekly",
            Recurrence::Monthly => "monthly",
            Recurrence::Bimonthly => "bimonthly",
            Recurrence::Yearly => "yearly",
        }
    }

    pub fn next_date(self, from: chrono::DateTime<chrono::Utc>) -> chrono::DateTime<chrono::Utc> {
        match self {
            Recurrence::Daily => from + chrono::Days::new(1),
            Recurrence::Weekly => from + chrono::Days::new(7),
            Recurrence::Biweekly => from + chrono::Days::new(14),
            Recurrence::Triweekly => from + chrono::Days::new(21),
            Recurrence::Monthly => from
                .checked_add_months(chrono::Months::new(1))
                .unwrap_or(from),
            Recurrence::Bimonthly => from
                .checked_add_months(chrono::Months::new(2))
                .unwrap_or(from),
            Recurrence::Yearly => from
                .checked_add_months(chrono::Months::new(12))
                .unwrap_or(from),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Task {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub description: String,
    pub priority: i64,
    pub due_date: Option<DateTime<Utc>>,
    pub recurrence: Option<String>,
    pub done: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Subtask {
    pub id: i64,
    pub task_id: i64,
    pub name: String,
    pub due_date: Option<DateTime<Utc>>,
    pub done: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SubtaskPatch {
    pub name: Option<String>,
    pub due_date: Option<Option<DateTime<Utc>>>,
    pub done: Option<bool>,
}

// structs for modifying tasks and projects

#[derive(Debug, Clone)]
pub struct ProjectPatch {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TaskPatch {
    pub name: Option<String>,
    pub description: Option<String>,
    pub priority: Option<i64>,
    pub due_date: Option<Option<chrono::DateTime<chrono::Utc>>>,
    pub recurrence: Option<Option<String>>,
    pub done: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct NextTask {
    pub task: Task,
    pub project_name: String,
}

#[derive(Default, Debug, Clone)]
pub struct Stats {
    pub total: i64,
    pub done: i64,
    pub pending: i64,
    pub overdue: i64,
    pub p1: i64,
    pub p2: i64,
    pub p3: i64,
    pub p4: i64,
}
