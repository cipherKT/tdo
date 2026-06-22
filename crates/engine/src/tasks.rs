use rusqlite::Result;

use crate::{Engine, StoreError, Task, TaskPatch};

impl Engine {
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

    pub(crate) fn get_task_by_id(&self, id: i64) -> Result<Task, StoreError> {
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

    pub(crate) fn get_task_by_name(
        &self,
        project_name: &str,
        task_name: &str,
    ) -> Result<Task, StoreError> {
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
