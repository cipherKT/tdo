mod error;
mod models;
mod projects;
mod stats;
mod tags;
mod tasks;

use rusqlite::Connection;
use std::path::Path;

pub use error::StoreError;
pub use models::{NextTask, Project, ProjectPatch, Stats, Tag, Task, TaskPatch};

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
            CREATE INDEX IF NOT EXISTS idx_tasks_due ON tasks(due_date) WHERE done=FALSE;
            CREATE UNIQUE INDEX IF NOT EXISTS idx_tasks_name_per_project ON tasks(name, project_id);
            ",
        )
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
}
