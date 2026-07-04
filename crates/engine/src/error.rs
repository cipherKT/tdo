use std::fmt;

#[derive(Debug)]
pub enum StoreError {
    NameTaken(String),
    TaskNameTaken(String),
    SubtaskNameTaken(String),
    PendingSubtasks(String),
    NotFound(String),
    Db(rusqlite::Error),
}

impl std::error::Error for StoreError {}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::NameTaken(name) => write!(f, "a project named '{}' already exists", name),
            StoreError::TaskNameTaken(name) => {
                write!(f, "a task named '{}' already exists in this project", name)
            }
            StoreError::SubtaskNameTaken(name) => {
                write!(f, "a subtask named '{}' already exists for this task", name)
            }
            StoreError::PendingSubtasks(name) => {
                write!(
                    f,
                    "cannot complete task '{}' because it has pending subtasks",
                    name
                )
            }
            StoreError::NotFound(name) => {
                write!(f, "no project, task or subtask named '{}' was found", name)
            }
            StoreError::Db(e) => write!(f, "database error: {}", e),
        }
    }
}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        StoreError::Db(e)
    }
}
