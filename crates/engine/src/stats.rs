use rusqlite::Result;

use crate::{Engine, Stats, StoreError};

impl Engine {
    pub fn project_stats(&self, project_name: &str) -> Result<Stats, StoreError> {
        let result = self.conn.query_row(
            "
            SELECT
              COUNT(*),
              COALESCE(SUM(done), 0),
              COALESCE(SUM(CASE WHEN done = 0 AND (due_date IS NULL OR DATE(due_date) >= DATE('now', 'localtime')) THEN 1 ELSE 0 END), 0),
              COALESCE(SUM(CASE WHEN done = 0 AND DATE(due_date) < DATE('now', 'localtime') THEN 1 ELSE 0 END), 0),
              COALESCE(SUM(CASE WHEN priority = 1 THEN 1 ELSE 0 END), 0),
              COALESCE(SUM(CASE WHEN priority = 2 THEN 1 ELSE 0 END), 0),
              COALESCE(SUM(CASE WHEN priority = 3 THEN 1 ELSE 0 END), 0),
              COALESCE(SUM(CASE WHEN priority = 4 THEN 1 ELSE 0 END), 0)
            FROM tasks
            WHERE project_id = (SELECT id FROM projects WHERE name = ?1)
            ",
            [project_name],
            |row| {
                Ok(Stats {
                    total: row.get(0)?,
                    done: row.get(1)?,
                    pending: row.get(2)?,
                    overdue: row.get(3)?,
                    p1: row.get(4)?,
                    p2: row.get(5)?,
                    p3: row.get(6)?,
                    p4: row.get(7)?,
                })
            },
        )?;
        Ok(result)
    }

    pub fn global_stats(&self) -> Result<Stats, StoreError> {
        let result = self.conn.query_row(
            "
            SELECT
              COUNT(*),
              COALESCE(SUM(done), 0),
              COALESCE(SUM(CASE WHEN done = 0 AND (due_date IS NULL OR DATE(due_date) >= DATE('now', 'localtime')) THEN 1 ELSE 0 END), 0),
              COALESCE(SUM(CASE WHEN done = 0 AND DATE(due_date) < DATE('now', 'localtime') THEN 1 ELSE 0 END), 0),
              COALESCE(SUM(CASE WHEN priority = 1 THEN 1 ELSE 0 END), 0),
              COALESCE(SUM(CASE WHEN priority = 2 THEN 1 ELSE 0 END), 0),
              COALESCE(SUM(CASE WHEN priority = 3 THEN 1 ELSE 0 END), 0),
              COALESCE(SUM(CASE WHEN priority = 4 THEN 1 ELSE 0 END), 0)
            FROM tasks
            ",
            [],
            |row| {
                Ok(Stats {
                    total: row.get(0)?,
                    done: row.get(1)?,
                    pending: row.get(2)?,
                    overdue: row.get(3)?,
                    p1: row.get(4)?,
                    p2: row.get(5)?,
                    p3: row.get(6)?,
                    p4: row.get(7)?,
                })
            },
        )?;
        Ok(result)
    }
}
