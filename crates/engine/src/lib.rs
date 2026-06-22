mod models;
use rusqlite::{Connection, Result};
use std::path::Path;

pub use models::{Project, ProjectPatch, Tag, Task, TaskPatch};
pub struct Engine {
    conn: Connection,
}

#[derive(Debug)]
pub enum StoreError {
    NameTaken(String),
    TaskNameTaken(String),
    NotFound(String),
    Db(rusqlite::Error),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl Engine {
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Engine> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
        let engine = Engine { conn };
        engine.migrate()?;
        Ok(engine)
    }
    fn migrate(&self) -> rusqlite::Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                description TEXT NOT NULL DEFAULT '',
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            );
            CREATE TABLE IF NOT EXISTS project_tags (
                project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
                PRIMARY KEY (project_id, tag_id)
            );
            CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY,
                project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                priority INTEGER NOT NULL,
                due_date TIMESTAMP,
                done BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS task_tags (
                task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
                tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
                PRIMARY KEY (task_id, tag_id)
            );
            CREATE INDEX IF NOT EXISTS idx_tasks_due ON tasks(due_date) WHERE done=FALSE;
            CREATE UNIQUE INDEX IF NOT EXISTS idx_tasks_name_per_project ON tasks(name, project_id);
            ",
        )
    }

    pub fn create_project(&self, name: &str, description: &str) -> Result<Project, StoreError> {
        let result = self.conn.execute(
            "INSERT INTO projects (name, description) VALUES (?1, ?2)",
            (name, description),
        );

        match result {
            Ok(_) => {
                let id = self.conn.last_insert_rowid();
                self.get_project_by_id(id)
            }
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Err(StoreError::NameTaken(name.to_string()))
            }
            Err(e) => Err(StoreError::Db(e)),
        }
    }

    fn get_project_by_id(&self, id: i64) -> Result<Project, StoreError> {
        let project = self.conn.query_row(
            "SELECT id, name, description, created_at FROM projects where id = ?1",
            [id],
            |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    created_at: row.get(3)?,
                })
            },
        )?;
        Ok(project)
    }

    pub fn list_projects(&self) -> Result<Vec<Project>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, created_at FROM projects ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        let projects: Vec<Project> = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(projects)
    }

    pub fn delete_project(&self, name: &str) -> Result<bool, StoreError> {
        let result = self
            .conn
            .execute("DELETE FROM projects where name = ?1", [name])?;
        Ok(result > 0)
    }

    pub fn modify_project(
        &self,
        name: &str,
        patch: ProjectPatch,
    ) -> Result<Option<Project>, StoreError> {
        let mut sets: Vec<&str> = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref new_name) = patch.name {
            sets.push("name = ?");
            params.push(Box::new(new_name.clone()));
        }

        if let Some(ref new_desc) = patch.description {
            sets.push("description = ?");
            params.push(Box::new(new_desc.clone()));
        }

        if sets.is_empty() {
            return self.get_project_by_name(name).map(Some);
        }

        let placeholders: Vec<String> = sets
            .iter()
            .enumerate()
            .map(|(i, set)| set.replace("?", &format!("?{}", i + 1)))
            .collect();

        let sql = format!(
            "UPDATE projects SET {} WHERE name = ?{}",
            placeholders.join(", "),
            params.len() + 1
        );

        params.push(Box::new(name.to_string()));

        let rows_affected = self.conn.execute(
            &sql,
            rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        )?;

        if rows_affected == 0 {
            return Ok(None);
        }

        let updated_name = patch.name.as_deref().unwrap_or(name);
        self.get_project_by_name(updated_name).map(Some)
    }

    fn get_project_by_name(&self, name: &str) -> Result<Project, StoreError> {
        let result = self.conn.query_row(
            "SELECT id, name, description, created_at FROM projects WHERE name = ?1",
            [name],
            |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    created_at: row.get(3)?,
                })
            },
        );
        match result {
            Ok(p) => Ok(p),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                Err(StoreError::NotFound(name.to_string()))
            }
            Err(e) => Err(StoreError::Db(e)),
        }
    }

    pub fn create_task(
        &self,
        project_name: &str,
        name: &str,
        description: &str,
        priority: i64,
        due_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Task, StoreError> {
        let project = self.get_project_by_name(project_name)?;
        let project_id = project.id;

        let result = self.conn.execute(
            "INSERT INTO tasks (project_id, name, description, priority, due_date, done)
             VALUES (?1, ?2, ?3, ?4, ?5, FALSE)",
            (project_id, name, description, priority, due_date),
        );

        match result {
            Ok(_) => {
                let id = self.conn.last_insert_rowid();
                self.get_task_by_id(id)
            }
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Err(StoreError::TaskNameTaken(name.to_string()))
            }
            Err(e) => Err(StoreError::Db(e)),
        }
    }

    fn get_task_by_id(&self, id: i64) -> Result<Task, StoreError> {
        let task = self.conn.query_row(
            "SELECT id, project_id, name, description, priority, due_date, done, created_at
             FROM tasks WHERE id = ?1",
            [id],
            |row| {
                Ok(Task {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    priority: row.get(4)?,
                    due_date: row.get(5)?,
                    done: row.get(6)?,
                    created_at: row.get(7)?,
                })
            },
        )?;
        Ok(task)
    }

    pub fn list_tasks(&self, project_name: &str) -> Result<Vec<Task>, StoreError> {
        let project = self.get_project_by_name(project_name)?;
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, name, description, priority, due_date, done, created_at
             FROM tasks WHERE project_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([project.id], |row| {
            Ok(Task {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                priority: row.get(4)?,
                due_date: row.get(5)?,
                done: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;
        let tasks: Vec<Task> = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(tasks)
    }

    fn get_task_by_name(&self, project_name: &str, task_name: &str) -> Result<Task, StoreError> {
        let result = self.conn.query_row(
            "SELECT t.id, t.project_id, t.name, t.description, t.priority, t.due_date, t.done, t.created_at
             FROM tasks t
             JOIN projects p ON t.project_id = p.id
             WHERE p.name = ?1 AND t.name = ?2",
            (project_name, task_name),
            |row| {
                Ok(Task {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    priority: row.get(4)?,
                    due_date: row.get(5)?,
                    done: row.get(6)?,
                    created_at: row.get(7)?,
                })
            },
        );
        match result {
            Ok(t) => Ok(t),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                Err(StoreError::NotFound(task_name.to_string()))
            }
            Err(e) => Err(StoreError::Db(e)),
        }
    }

    pub fn delete_task(&self, project_name: &str, task_name: &str) -> Result<bool, StoreError> {
        let result = self.conn.execute(
            "DELETE FROM tasks WHERE project_id = (SELECT id FROM projects WHERE name = ?1) AND name = ?2",
            (project_name, task_name),
        )?;
        Ok(result > 0)
    }

    pub fn modify_task(
        &self,
        project_name: &str,
        task_name: &str,
        patch: TaskPatch,
    ) -> Result<Option<Task>, StoreError> {
        let mut sets: Vec<&str> = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref new_name) = patch.name {
            sets.push("name = ?");
            params.push(Box::new(new_name.clone()));
        }

        if let Some(ref new_desc) = patch.description {
            sets.push("description = ?");
            params.push(Box::new(new_desc.clone()));
        }

        if let Some(new_priority) = patch.priority {
            sets.push("priority = ?");
            params.push(Box::new(new_priority));
        }

        if let Some(due_date) = patch.due_date {
            sets.push("due_date = ?");
            params.push(Box::new(due_date));
        }

        if let Some(done) = patch.done {
            sets.push("done = ?");
            params.push(Box::new(done));
        }

        if sets.is_empty() {
            return self.get_task_by_name(project_name, task_name).map(Some);
        }

        let placeholders: Vec<String> = sets
            .iter()
            .enumerate()
            .map(|(i, set)| set.replace("?", &format!("?{}", i + 1)))
            .collect();

        let sql = format!(
            "UPDATE tasks SET {} WHERE project_id = (SELECT id FROM projects WHERE name = ?{}) AND name = ?{}",
            placeholders.join(", "),
            params.len() + 1,
            params.len() + 2,
        );

        params.push(Box::new(project_name.to_string()));
        params.push(Box::new(task_name.to_string()));

        let rows_affected = self.conn.execute(
            &sql,
            rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        )?;

        if rows_affected == 0 {
            return Ok(None);
        }

        let updated_name = patch.name.as_deref().unwrap_or(task_name);
        self.get_task_by_name(project_name, updated_name).map(Some)
    }
}
