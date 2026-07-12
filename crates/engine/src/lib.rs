mod error;
mod models;
mod projects;
mod stats;
mod subtasks;
mod tags;
mod tasks;

use rusqlite::Connection;
use std::path::Path;

pub use error::StoreError;
pub use models::{
    NextTask, Project, ProjectPatch, Stats, Subtask, SubtaskPatch, Tag, Task, TaskPatch,
};

pub struct Engine {
    conn: Connection,
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
            CREATE TABLE IF NOT EXISTS subtasks (
                id INTEGER PRIMARY KEY,
                task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                due_date TIMESTAMP,
                done BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX IF NOT EXISTS idx_tasks_due ON tasks(due_date) WHERE done=FALSE;
            CREATE UNIQUE INDEX IF NOT EXISTS idx_tasks_name_per_project ON tasks(name, project_id);
            CREATE UNIQUE INDEX IF NOT EXISTS idx_subtasks_name_per_task ON subtasks(name, task_id);
            ",
        )?;
        let _ = self.conn.execute("ALTER TABLE subtasks ADD COLUMN due_date TIMESTAMP", []);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_crud() {
        let engine = Engine::open(":memory:").unwrap();

        // Create project
        let p = engine.create_project("test_proj", "desc").unwrap();
        assert_eq!(p.name, "test_proj");
        assert_eq!(p.description, "desc");

        // Duplicate name should fail
        let err = engine.create_project("test_proj", "other").unwrap_err();
        assert!(matches!(err, StoreError::NameTaken(_)));

        // List projects
        let list = engine.list_projects().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "test_proj");

        // Project names
        let names = engine.project_names().unwrap();
        assert_eq!(names, vec!["test_proj".to_string()]);

        // Modify project
        let patch = ProjectPatch {
            name: Some("test_proj_new".to_string()),
            description: Some("new desc".to_string()),
        };
        let updated = engine.modify_project("test_proj", patch).unwrap().unwrap();
        assert_eq!(updated.name, "test_proj_new");
        assert_eq!(updated.description, "new desc");

        // Delete project
        let deleted = engine.delete_project("test_proj_new").unwrap();
        assert!(deleted);
        assert_eq!(engine.list_projects().unwrap().len(), 0);
    }

    #[test]
    fn test_task_crud() {
        let engine = Engine::open(":memory:").unwrap();
        engine.create_project("proj", "desc").unwrap();

        // Create task
        let t = engine
            .create_task("proj", "task1", "task desc", 1, None)
            .unwrap();
        assert_eq!(t.name, "task1");
        assert_eq!(t.priority, 1);
        assert!(!t.done);

        // Duplicate task name in same project should fail
        let err = engine
            .create_task("proj", "task1", "other", 2, None)
            .unwrap_err();
        assert!(matches!(err, StoreError::TaskNameTaken(_)));

        // List tasks
        let list = engine.list_tasks("proj").unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "task1");

        // Toggle done
        let toggled = engine.toggle_done("proj", "task1").unwrap();
        assert!(toggled.done);

        // Modify task
        let patch = TaskPatch {
            name: Some("task1_new".to_string()),
            description: Some("new desc".to_string()),
            priority: Some(2),
            due_date: None,
            done: Some(false),
        };
        let modified = engine.modify_task("proj", "task1", patch).unwrap().unwrap();
        assert_eq!(modified.name, "task1_new");
        assert_eq!(modified.description, "new desc");
        assert_eq!(modified.priority, 2);
        assert!(!modified.done);

        // Delete task
        let deleted = engine.delete_task("proj", "task1_new").unwrap();
        assert!(deleted);
        assert_eq!(engine.list_tasks("proj").unwrap().len(), 0);
    }

    #[test]
    fn test_tags() {
        let engine = Engine::open(":memory:").unwrap();
        engine.create_project("proj", "desc").unwrap();
        engine
            .create_task("proj", "task1", "desc", 3, None)
            .unwrap();

        // Add tags to project
        engine
            .add_tags_to_project("proj", &["tag1", "tag2"])
            .unwrap();
        let p_tags = engine.get_tags_for_project("proj").unwrap();
        assert_eq!(p_tags.len(), 2);
        let mut p_tag_names: Vec<String> = p_tags.into_iter().map(|t| t.name).collect();
        p_tag_names.sort();
        assert_eq!(p_tag_names, vec!["tag1".to_string(), "tag2".to_string()]);

        // Remove tag from project
        let removed = engine.remove_tag_from_project("proj", "tag1").unwrap();
        assert!(removed);
        let p_tags = engine.get_tags_for_project("proj").unwrap();
        assert_eq!(p_tags.len(), 1);
        assert_eq!(p_tags[0].name, "tag2");

        // Add tags to task
        engine
            .add_tags_to_task("proj", "task1", &["task_tag"])
            .unwrap();
    }

    #[test]
    fn test_stats() {
        let engine = Engine::open(":memory:").unwrap();
        engine.create_project("proj", "desc").unwrap();
        engine.create_task("proj", "t1", "d", 1, None).unwrap();
        engine.create_task("proj", "t2", "d", 2, None).unwrap();

        let stats = engine.project_stats("proj").unwrap();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.p1, 1);
        assert_eq!(stats.p2, 1);
        assert_eq!(stats.done, 0);

        engine.toggle_done("proj", "t1").unwrap();
        let stats = engine.project_stats("proj").unwrap();
        assert_eq!(stats.done, 1);

        let g_stats = engine.global_stats().unwrap();
        assert_eq!(g_stats.total, 2);
        assert_eq!(g_stats.done, 1);
    }

    #[test]
    fn test_pending_today_tasks() {
        let engine = Engine::open(":memory:").unwrap();
        engine.create_project("proj1", "desc1").unwrap();
        engine.create_project("proj2", "desc2").unwrap();

        let today = chrono::Utc::now();
        engine
            .create_task("proj1", "task1", "d", 1, Some(today))
            .unwrap();
        engine
            .create_task("proj2", "task2", "d", 2, Some(today))
            .unwrap();

        let tomorrow = today + chrono::Days::new(1);
        engine
            .create_task("proj1", "task3", "d", 3, Some(tomorrow))
            .unwrap();

        let pending_today = engine.list_pending_today_tasks().unwrap();
        assert_eq!(pending_today.len(), 2);
        assert_eq!(pending_today[0].task.name, "task1");
        assert_eq!(pending_today[0].project_name, "proj1");
        assert_eq!(pending_today[1].task.name, "task2");
        assert_eq!(pending_today[1].project_name, "proj2");
    }

    #[test]
    fn test_list_today_tasks() {
        let engine = Engine::open(":memory:").unwrap();
        engine.create_project("proj", "desc").unwrap();

        // 1. Initially, no tasks -> list_today_tasks() is empty
        assert!(engine.list_today_tasks().unwrap().is_empty());

        let today = chrono::Utc::now();
        let yesterday = today - chrono::Days::new(1);
        let tomorrow = today + chrono::Days::new(1);

        // 2. Future task -> list_today_tasks() is still empty because it's not due yet
        engine
            .create_task("proj", "future", "d", 1, Some(tomorrow))
            .unwrap();
        assert!(engine.list_today_tasks().unwrap().is_empty());

        // 3. Overdue task (priority 3) -> should be returned
        engine
            .create_task("proj", "overdue", "d", 3, Some(yesterday))
            .unwrap();
        let list = engine.list_today_tasks().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].task.name, "overdue");

        // 4. Today task (priority 2) -> priority 2 is higher than 3 (priority ASC order)
        engine
            .create_task("proj", "today_p2", "d", 2, Some(today))
            .unwrap();
        let list = engine.list_today_tasks().unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].task.name, "today_p2");
        assert_eq!(list[1].task.name, "overdue");

        // 5. Overdue task (priority 1) -> priority 1 is even higher, should be returned first
        engine
            .create_task("proj", "overdue_p1", "d", 1, Some(yesterday))
            .unwrap();
        let list = engine.list_today_tasks().unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].task.name, "overdue_p1");
        assert_eq!(list[1].task.name, "today_p2");
        assert_eq!(list[2].task.name, "overdue");

        // 6. Complete the highest priority task -> next should be "today_p2"
        engine.toggle_done("proj", "overdue_p1").unwrap();
        let list = engine.list_today_tasks().unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].task.name, "today_p2");
        assert_eq!(list[1].task.name, "overdue");
    }

    #[test]
    fn test_subtasks() {
        let engine = Engine::open(":memory:").unwrap();
        engine.create_project("proj", "desc").unwrap();
        engine
            .create_task("proj", "task1", "task desc", 1, None)
            .unwrap();

        // 1. Create subtask
        let st1 = engine.create_subtask("proj", "task1", "sub1", None).unwrap();
        assert_eq!(st1.name, "sub1");
        assert!(!st1.done);

        // Duplicate subtask name should fail
        let err = engine.create_subtask("proj", "task1", "sub1", None).unwrap_err();
        assert!(matches!(err, StoreError::SubtaskNameTaken(_)));

        // 2. List subtasks
        let list = engine.list_subtasks("proj", "task1").unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "sub1");

        // 3. Strict completion check: cannot mark task1 done since sub1 is undone
        let err = engine.toggle_done("proj", "task1").unwrap_err();
        assert!(matches!(err, StoreError::PendingSubtasks(_)));

        // 4. Toggle subtask done
        let st1 = engine.toggle_subtask_done("proj", "task1", "sub1").unwrap();
        assert!(st1.done);

        // 5. Now we can mark task1 done
        let t1 = engine.toggle_done("proj", "task1").unwrap();
        assert!(t1.done);

        // 6. Creating a new subtask (starts undone) should reopen the task
        let st2 = engine.create_subtask("proj", "task1", "sub2", None).unwrap();
        assert_eq!(st2.name, "sub2");
        assert!(!st2.done);

        let t1 = engine.get_task_by_name("proj", "task1").unwrap();
        assert!(!t1.done); // Auto-reopened!

        // Complete sub2
        engine.toggle_subtask_done("proj", "task1", "sub2").unwrap();
        // Mark task1 done again
        engine.toggle_done("proj", "task1").unwrap();

        // 7. Toggling subtask undone should reopen task1
        engine.toggle_subtask_done("proj", "task1", "sub1").unwrap(); // sub1 is now undone
        let t1 = engine.get_task_by_name("proj", "task1").unwrap();
        assert!(!t1.done); // Auto-reopened!

        // 8. Delete subtask
        let deleted = engine.delete_subtask("proj", "task1", "sub1").unwrap();
        assert!(deleted);
        let list = engine.list_subtasks("proj", "task1").unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "sub2");
    }
}
