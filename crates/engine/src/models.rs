use chrono::{DateTime, Utc};

pub struct Project {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

pub struct Tag {
    pub id: i64,
    pub name: String,
}

pub struct Task {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub description: String,
    pub priority: i64,
    pub due_date: Option<DateTime<Utc>>,
    pub done: bool,
    pub created_at: DateTime<Utc>,
}

// structs for modifying tasks and projects

pub struct ProjectPatch {
    pub name: Option<String>,
    pub description: Option<String>,
}

pub struct TaskPatch {
    pub name: Option<String>,
    pub description: Option<String>,
    pub priority: Option<i64>,
    pub due_date: Option<Option<chrono::DateTime<chrono::Utc>>>,
    pub done: Option<bool>,
}

pub struct NextTask {
    pub task: Task,
    pub project_name: String,
}

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
