use rusqlite::Result;

use crate::{Engine, NextTask, StoreError, Task, TaskPatch};

impl Engine {
    pub fn create_task(
        &self,
        project_name: &str,
        name: &str,
        description: &str,
        priority: i64,
        due_date: Option<chrono::DateTime<chrono::Utc>>,
        recurrence: Option<String>,
    ) -> Result<Task, StoreError> {
        let project = self.get_project_by_name(project_name)?;
        let project_id = project.id;

        if let Some(ref rec) = recurrence {
            if !rec.trim().is_empty() && crate::models::Recurrence::parse(rec).is_none() {
                return Err(StoreError::InvalidRecurrence(rec.clone()));
            }
        }

        let mut final_due = due_date;
        if final_due.is_none() {
            if let Some(ref rec) = recurrence {
                if !rec.trim().is_empty() {
                    let today = chrono::Utc::now().date_naive();
                    final_due = today.and_hms_opt(0, 0, 0).map(|dt| dt.and_utc());
                }
            }
        }

        let result = self.conn.execute(
            "INSERT INTO tasks (project_id, name, description, priority, due_date, recurrence, done)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, FALSE)",
            (project_id, name, description, priority, final_due, recurrence),
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
            "SELECT id, project_id, name, description, priority, due_date, recurrence, done, created_at
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
                    recurrence: row.get(6)?,
                    done: row.get(7)?,
                    created_at: row.get(8)?,
                })
            },
        )?;
        Ok(task)
    }

    pub fn list_tasks(&self, project_name: &str) -> Result<Vec<Task>, StoreError> {
        let project = self.get_project_by_name(project_name)?;
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, name, description, priority, due_date, recurrence, done, created_at
             FROM tasks WHERE project_id = ?1 ORDER BY due_date ASC NULLS LAST",
        )?;
        let rows = stmt.query_map([project.id], |row| {
            Ok(Task {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                priority: row.get(4)?,
                due_date: row.get(5)?,
                recurrence: row.get(6)?,
                done: row.get(7)?,
                created_at: row.get(8)?,
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
            "SELECT t.id, t.project_id, t.name, t.description, t.priority, t.due_date, t.recurrence, t.done, t.created_at
             FROM tasks t
             JOIN projects p ON t.project_id = p.id
             WHERE p.name = ?1 AND t.name = ?2
             ORDER BY t.done ASC, t.due_date DESC",
            (project_name, task_name),
            |row| {
                Ok(Task {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    priority: row.get(4)?,
                    due_date: row.get(5)?,
                    recurrence: row.get(6)?,
                    done: row.get(7)?,
                    created_at: row.get(8)?,
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

    pub fn delete_task_by_id(&self, id: i64) -> Result<bool, StoreError> {
        let result = self.conn.execute("DELETE FROM tasks WHERE id = ?1", [id])?;
        Ok(result > 0)
    }

    pub fn delete_task(&self, project_name: &str, task_name: &str) -> Result<bool, StoreError> {
        let task = self.get_task_by_name(project_name, task_name)?;
        self.delete_task_by_id(task.id)
    }

    pub fn modify_task_by_id(&self, id: i64, patch: TaskPatch) -> Result<Option<Task>, StoreError> {
        let current_task = self.get_task_by_id(id)?;

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

        if let Some(ref recurrence) = patch.recurrence {
            if let Some(rec) = recurrence {
                if !rec.trim().is_empty() && crate::models::Recurrence::parse(rec).is_none() {
                    return Err(StoreError::InvalidRecurrence(rec.clone()));
                }
            }
            sets.push("recurrence = ?");
            params.push(Box::new(recurrence.clone()));
        }

        // If we are changing to a repetitive task, and it doesn't currently have a due date (and we aren't setting one), default to today.
        if let Some(Some(ref rec)) = patch.recurrence {
            if !rec.trim().is_empty() {
                let will_have_due = match patch.due_date {
                    Some(Some(_)) => true,
                    Some(None) => false,
                    None => current_task.due_date.is_some(),
                };
                if !will_have_due {
                    sets.push("due_date = ?");
                    let today = chrono::Utc::now().date_naive();
                    let today_dt = today.and_hms_opt(0, 0, 0).map(|dt| dt.and_utc());
                    params.push(Box::new(today_dt));
                }
            }
        }

        if let Some(done) = patch.done {
            if done {
                let pending_count: i64 = self.conn.query_row(
                    "SELECT COUNT(*) FROM subtasks WHERE task_id = ? AND done = FALSE",
                    [current_task.id],
                    |row| row.get(0),
                )?;
                if pending_count > 0 {
                    return Err(StoreError::PendingSubtasks(current_task.name.clone()));
                }
            }
            sets.push("done = ?");
            params.push(Box::new(done));
        }

        if sets.is_empty() {
            return Ok(Some(current_task));
        }

        let placeholders: Vec<String> = sets
            .iter()
            .enumerate()
            .map(|(i, set)| set.replace("?", &format!("?{}", i + 1)))
            .collect();

        let sql = format!(
            "UPDATE tasks SET {} WHERE id = ?{}",
            placeholders.join(", "),
            params.len() + 1,
        );

        params.push(Box::new(id));

        let rows_affected = self.conn.execute(
            &sql,
            rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        )?;

        if rows_affected == 0 {
            return Ok(None);
        }

        let task = self.get_task_by_id(id)?;

        if let Some(true) = patch.done {
            if let Some(ref rec_str) = task.recurrence {
                if !rec_str.trim().is_empty() {
                    self.handle_recurrence_clone(&task, rec_str)?;
                }
            }
        }

        Ok(Some(task))
    }

    pub fn modify_task(
        &self,
        project_name: &str,
        task_name: &str,
        patch: TaskPatch,
    ) -> Result<Option<Task>, StoreError> {
        let task = self.get_task_by_name(project_name, task_name)?;
        self.modify_task_by_id(task.id, patch)
    }

    fn handle_recurrence_clone(&self, task: &Task, rec_str: &str) -> Result<(), StoreError> {
        let recurrence = crate::models::Recurrence::parse(rec_str)
            .ok_or_else(|| StoreError::InvalidRecurrence(rec_str.to_string()))?;

        let base_date = task.due_date.unwrap_or_else(chrono::Utc::now);
        let next_due = recurrence.next_date(base_date);

        self.conn.execute(
            "INSERT INTO tasks (project_id, name, description, priority, due_date, recurrence, done)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, FALSE)",
            (
                task.project_id,
                &task.name,
                &task.description,
                task.priority,
                Some(next_due),
                Some(rec_str.to_string()),
            ),
        )?;
        let new_task_id = self.conn.last_insert_rowid();

        // Copy tags
        let tags = self.get_tags_for_task_by_id(task.id)?;
        for tag in tags {
            self.conn.execute(
                "INSERT OR IGNORE INTO task_tags (task_id, tag_id) VALUES (?, ?)",
                (new_task_id, tag.id),
            )?;
        }

        // Copy subtasks
        let subtasks = self.get_subtasks_for_task(task.id)?;
        for sub in subtasks {
            let sub_due = sub.due_date.map(|sd| {
                if let Some(orig_task_due) = task.due_date {
                    let diff = sd.signed_duration_since(orig_task_due);
                    next_due + diff
                } else {
                    recurrence.next_date(sd)
                }
            });

            self.conn.execute(
                "INSERT INTO subtasks (task_id, name, due_date, done) VALUES (?1, ?2, ?3, FALSE)",
                (new_task_id, &sub.name, sub_due),
            )?;
        }

        Ok(())
    }

    pub fn toggle_done_by_id(&self, id: i64) -> Result<Task, StoreError> {
        let task = self.get_task_by_id(id)?;
        let patch = TaskPatch {
            done: Some(!task.done),
            name: None,
            description: None,
            priority: None,
            due_date: None,
            recurrence: None,
        };
        self.modify_task_by_id(id, patch)
            .map(|t| t.expect("task must exist since we fetched it"))
    }

    pub fn toggle_done(&self, project_name: &str, task_name: &str) -> Result<Task, StoreError> {
        let task = self.get_task_by_name(project_name, task_name)?;
        self.toggle_done_by_id(task.id)
    }

    pub fn list_today_tasks(&self) -> Result<Vec<NextTask>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.project_id, t.name, t.description, t.priority, t.due_date, t.recurrence, t.done, t.created_at, p.name as project_name
             FROM tasks t
             JOIN projects p ON t.project_id = p.id
             WHERE t.done = FALSE
               AND t.due_date IS NOT NULL
               AND DATE(t.due_date) <= DATE('now', 'localtime')
             UNION ALL
             SELECT s.id, t.project_id, t.name || ' ↪ ' || s.name as name, '' as description, t.priority, s.due_date, NULL as recurrence, s.done, s.created_at, p.name as project_name
             FROM subtasks s
             JOIN tasks t ON s.task_id = t.id
             JOIN projects p ON t.project_id = p.id
             WHERE s.done = FALSE
               AND s.due_date IS NOT NULL
               AND DATE(s.due_date) <= DATE('now', 'localtime')
             ORDER BY priority ASC, due_date ASC, name ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(NextTask {
                task: Task {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    priority: row.get(4)?,
                    due_date: row.get(5)?,
                    recurrence: row.get(6)?,
                    done: row.get(7)?,
                    created_at: row.get(8)?,
                },
                project_name: row.get(9)?,
            })
        })?;
        let tasks = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(tasks)
    }

    pub fn task_names(&self, project_name: &str) -> Result<Vec<String>, StoreError> {
        let project = self.get_project_by_name(project_name)?;
        let mut stmt = self
            .conn
            .prepare("SELECT name FROM tasks WHERE project_id = ?1 ORDER BY name ASC")?;
        let names = stmt
            .query_map([project.id], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;
        Ok(names)
    }

    /// Returns all pending tasks due on a specific date (year, month, day).
    pub fn list_tasks_due_on(
        &self,
        year: i32,
        month: u32,
        day: u32,
    ) -> Result<Vec<NextTask>, StoreError> {
        let date_str = format!("{:04}-{:02}-{:02}", year, month, day);
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.project_id, t.name, t.description, t.priority, t.due_date, t.recurrence, t.done, t.created_at, p.name as project_name
             FROM tasks t
             JOIN projects p ON t.project_id = p.id
             WHERE t.done = 0 AND t.due_date IS NOT NULL AND DATE(t.due_date) = ?1
             UNION ALL
             SELECT s.id, t.project_id, t.name || ' ↪ ' || s.name as name, '' as description, t.priority, s.due_date, NULL as recurrence, s.done, s.created_at, p.name as project_name
             FROM subtasks s
             JOIN tasks t ON s.task_id = t.id
             JOIN projects p ON t.project_id = p.id
             WHERE s.done = 0 AND s.due_date IS NOT NULL AND DATE(s.due_date) = ?1
             ORDER BY priority ASC, name ASC",
        )?;
        let rows = stmt.query_map([&date_str], |row| {
            Ok(NextTask {
                task: Task {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    priority: row.get(4)?,
                    due_date: row.get(5)?,
                    recurrence: row.get(6)?,
                    done: row.get(7)?,
                    created_at: row.get(8)?,
                },
                project_name: row.get(9)?,
            })
        })?;
        let tasks = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(tasks)
    }

    /// Returns the count of pending tasks due on a specific date.
    pub fn count_tasks_due_on(&self, year: i32, month: u32, day: u32) -> Result<i64, StoreError> {
        let date_str = format!("{:04}-{:02}-{:02}", year, month, day);
        let count: i64 = self.conn.query_row(
            "SELECT (
                (SELECT COUNT(*) FROM tasks t WHERE t.done = 0 AND t.due_date IS NOT NULL AND DATE(t.due_date) = ?1) +
                (SELECT COUNT(*) FROM subtasks s WHERE s.done = 0 AND s.due_date IS NOT NULL AND DATE(s.due_date) = ?1)
             )",
            [&date_str],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn list_pending_today_tasks(&self) -> Result<Vec<NextTask>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.project_id, t.name, t.description, t.priority, t.due_date, t.recurrence, t.done, t.created_at, p.name as project_name
             FROM tasks t
             JOIN projects p ON t.project_id = p.id
             WHERE t.done = 0 AND t.due_date IS NOT NULL AND DATE(t.due_date) <= DATE('now', 'localtime')
             UNION ALL
             SELECT s.id, t.project_id, t.name || ' ↪ ' || s.name as name, '' as description, t.priority, s.due_date, NULL as recurrence, s.done, s.created_at, p.name as project_name
             FROM subtasks s
             JOIN tasks t ON s.task_id = t.id
             JOIN projects p ON t.project_id = p.id
             WHERE s.done = 0 AND s.due_date IS NOT NULL AND DATE(s.due_date) <= DATE('now', 'localtime')
             ORDER BY priority ASC, name ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(NextTask {
                task: Task {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    priority: row.get(4)?,
                    due_date: row.get(5)?,
                    recurrence: row.get(6)?,
                    done: row.get(7)?,
                    created_at: row.get(8)?,
                },
                project_name: row.get(9)?,
            })
        })?;
        let tasks = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(tasks)
    }
}
