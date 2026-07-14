use crate::{Engine, StoreError, Subtask, SubtaskPatch};
use rusqlite::Result;

impl Engine {
    pub fn create_subtask_by_id(
        &self,
        task_id: i64,
        name: &str,
        due_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Subtask, StoreError> {
        let task = self.get_task_by_id(task_id)?;

        // Invariant: If parent task was done, reopen it.
        if task.done {
            self.conn
                .execute("UPDATE tasks SET done = FALSE WHERE id = ?", [task_id])?;
        }

        if let (Some(sub_due), Some(task_due)) = (due_date, task.due_date) {
            if sub_due > task_due {
                return Err(StoreError::InvalidDueDate(
                    "subtask due date cannot be after parent task due date".to_string(),
                ));
            }
        }

        let result = self.conn.execute(
            "INSERT INTO subtasks (task_id, name, due_date, done) VALUES (?1, ?2, ?3, FALSE)",
            (task_id, name, due_date),
        );

        match result {
            Ok(_) => {
                let id = self.conn.last_insert_rowid();
                self.get_subtask_by_id(id)
            }
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Err(StoreError::SubtaskNameTaken(name.to_string()))
            }
            Err(e) => Err(StoreError::Db(e)),
        }
    }

    pub fn create_subtask(
        &self,
        project_name: &str,
        task_name: &str,
        name: &str,
        due_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Subtask, StoreError> {
        let task = self.get_task_by_name(project_name, task_name)?;
        self.create_subtask_by_id(task.id, name, due_date)
    }

    pub(crate) fn get_subtask_by_id(&self, id: i64) -> Result<Subtask, StoreError> {
        let subtask = self.conn.query_row(
            "SELECT id, task_id, name, due_date, done, created_at FROM subtasks WHERE id = ?1",
            [id],
            |row| {
                Ok(Subtask {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    name: row.get(2)?,
                    due_date: row.get(3)?,
                    done: row.get(4)?,
                    created_at: row.get(5)?,
                })
            },
        )?;
        Ok(subtask)
    }

    pub fn list_subtasks(
        &self,
        project_name: &str,
        task_name: &str,
    ) -> Result<Vec<Subtask>, StoreError> {
        let task = self.get_task_by_name(project_name, task_name)?;
        let mut stmt = self.conn.prepare(
            "SELECT id, task_id, name, due_date, done, created_at FROM subtasks WHERE task_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map([task.id], |row| {
            Ok(Subtask {
                id: row.get(0)?,
                task_id: row.get(1)?,
                name: row.get(2)?,
                due_date: row.get(3)?,
                done: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        let subtasks = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(subtasks)
    }

    pub fn get_subtasks_for_task(&self, task_id: i64) -> Result<Vec<Subtask>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, task_id, name, due_date, done, created_at FROM subtasks WHERE task_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map([task_id], |row| {
            Ok(Subtask {
                id: row.get(0)?,
                task_id: row.get(1)?,
                name: row.get(2)?,
                due_date: row.get(3)?,
                done: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        let subtasks = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(subtasks)
    }

    pub fn delete_subtask_by_id(&self, id: i64) -> Result<bool, StoreError> {
        let result = self
            .conn
            .execute("DELETE FROM subtasks WHERE id = ?1", [id])?;
        Ok(result > 0)
    }

    pub fn delete_subtask(
        &self,
        project_name: &str,
        task_name: &str,
        subtask_name: &str,
    ) -> Result<bool, StoreError> {
        let task = self.get_task_by_name(project_name, task_name)?;
        let subtask_id = self.conn.query_row(
            "SELECT id FROM subtasks WHERE task_id = ?1 AND name = ?2",
            (task.id, subtask_name),
            |row| row.get::<_, i64>(0),
        )?;
        self.delete_subtask_by_id(subtask_id)
    }

    pub fn toggle_subtask_done_by_id(&self, id: i64) -> Result<Subtask, StoreError> {
        let subtask = self.get_subtask_by_id(id)?;
        let task = self.get_task_by_id(subtask.task_id)?;

        let new_done = !subtask.done;

        self.conn.execute(
            "UPDATE subtasks SET done = ?1 WHERE id = ?2",
            (new_done, subtask.id),
        )?;

        // Strict completion invariant:
        // If we marked a subtask UNDONE, and the parent task is currently DONE, the parent task MUST become UNDONE.
        if !new_done && task.done {
            self.conn
                .execute("UPDATE tasks SET done = FALSE WHERE id = ?", [task.id])?;
        }

        self.get_subtask_by_id(subtask.id)
    }

    pub fn toggle_subtask_done(
        &self,
        project_name: &str,
        task_name: &str,
        subtask_name: &str,
    ) -> Result<Subtask, StoreError> {
        let task = self.get_task_by_name(project_name, task_name)?;
        let subtask_id = self.conn.query_row(
            "SELECT id FROM subtasks WHERE task_id = ?1 AND name = ?2",
            (task.id, subtask_name),
            |row| row.get::<_, i64>(0),
        )?;
        self.toggle_subtask_done_by_id(subtask_id)
    }

    pub fn modify_subtask_by_id(
        &self,
        id: i64,
        patch: SubtaskPatch,
    ) -> Result<Option<Subtask>, StoreError> {
        let subtask = self.get_subtask_by_id(id)?;
        let task = self.get_task_by_id(subtask.task_id)?;

        let mut sets = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref new_name) = patch.name {
            sets.push("name = ?");
            params.push(Box::new(new_name.clone()));
        }

        if let Some(due_date) = patch.due_date {
            if let (Some(sub_due), Some(task_due)) = (due_date, task.due_date) {
                if sub_due > task_due {
                    return Err(StoreError::InvalidDueDate(
                        "subtask due date cannot be after parent task due date".to_string(),
                    ));
                }
            }
            sets.push("due_date = ?");
            params.push(Box::new(due_date));
        }

        if let Some(new_done) = patch.done {
            sets.push("done = ?");
            params.push(Box::new(new_done));

            // If we marked a subtask UNDONE, and the parent task is currently DONE, parent must become UNDONE.
            if !new_done && task.done {
                self.conn
                    .execute("UPDATE tasks SET done = FALSE WHERE id = ?", [task.id])?;
            }
        }

        if sets.is_empty() {
            return Ok(Some(subtask));
        }

        let placeholders: Vec<String> = sets
            .iter()
            .enumerate()
            .map(|(i, set)| set.replace("?", &format!("?{}", i + 1)))
            .collect();

        let sql = format!(
            "UPDATE subtasks SET {} WHERE id = ?{}",
            placeholders.join(", "),
            params.len() + 1
        );

        params.push(Box::new(id));

        let rows_affected = self.conn.execute(
            &sql,
            rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        )?;

        if rows_affected == 0 {
            return Ok(None);
        }

        self.get_subtask_by_id(id).map(Some)
    }

    pub fn modify_subtask(
        &self,
        project_name: &str,
        task_name: &str,
        subtask_name: &str,
        patch: SubtaskPatch,
    ) -> Result<Option<Subtask>, StoreError> {
        let task = self.get_task_by_name(project_name, task_name)?;
        let subtask_id = self.conn.query_row(
            "SELECT id FROM subtasks WHERE task_id = ?1 AND name = ?2",
            (task.id, subtask_name),
            |row| row.get::<_, i64>(0),
        )?;
        self.modify_subtask_by_id(subtask_id, patch)
    }
}
