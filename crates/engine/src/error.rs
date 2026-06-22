use std::fmt;

#[derive(Debug)]
pub enum StoreError {
    NameTaken(String),
    TaskNameTaken(String),
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
            StoreError::NotFound(name) => {
                write!(f, "no project or task named '{}' was found", name)
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
