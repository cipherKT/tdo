use crate::{Engine, StoreError, Tag};
use rusqlite::Result;

impl Engine {
    fn get_tag_by_name(&self, tag_name: &str) -> Result<Tag, StoreError> {
        let result = self.conn.query_row(
            "SELECT id, name FROM tags WHERE name = ?1",
            [tag_name],
            |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            },
        );
        match result {
            Ok(t) => Ok(t),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                Err(StoreError::NotFound(tag_name.to_string()))
            }
            Err(e) => Err(StoreError::Db(e)),
        }
    }

    pub(crate) fn get_or_create_tag(&self, name: &str) -> Result<Tag, StoreError> {
        self.conn
            .execute("INSERT OR IGNORE INTO tags (name) VALUES (?1)", [name])?;
        let tag =
            self.conn
                .query_row("SELECT id, name FROM tags WHERE name = ?1", [name], |row| {
                    Ok(Tag {
                        id: row.get(0)?,
                        name: row.get(1)?,
                    })
                })?;
        Ok(tag)
    }

    pub fn add_tags_to_project(&self, project_name: &str, tags: &[&str]) -> Result<(), StoreError> {
        let project = self.get_project_by_name(project_name)?;
        let project_id = project.id;
        for tag in tags {
            let temp_tag = self.get_or_create_tag(tag)?;
            self.conn.execute(
                "INSERT OR IGNORE INTO project_tags (project_id, tag_id) VALUES (?1, ?2)",
                [project_id, temp_tag.id],
            )?;
        }
        Ok(())
    }

    pub fn get_tags_for_project(&self, project_name: &str) -> Result<Vec<Tag>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT tags.id, tags.name
             FROM tags
             JOIN project_tags ON tags.id = project_tags.tag_id
             JOIN projects ON project_tags.project_id = projects.id
             WHERE projects.name = ?1",
        )?;
        let tags = stmt.query_map([project_name], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;
        Ok(tags.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn remove_tag_from_project(
        &self,
        project_name: &str,
        tag_name: &str,
    ) -> Result<bool, StoreError> {
        let project = self.get_project_by_name(project_name)?;
        let tag = self.get_tag_by_name(tag_name)?;
        let result = self.conn.execute(
            "DELETE FROM project_tags WHERE project_id = ?1 AND tag_id = ?2",
            [project.id, tag.id],
        )?;
        Ok(result > 0)
    }

    pub fn add_tags_to_task(
        &self,
        project_name: &str,
        task_name: &str,
        tags: &[&str],
    ) -> Result<(), StoreError> {
        let task = self.get_task_by_name(project_name, task_name)?;
        let task_id = task.id;
        for tag in tags {
            let temp_tag = self.get_or_create_tag(tag)?;
            self.conn.execute(
                "INSERT OR IGNORE INTO task_tags (task_id, tag_id) VALUES (?1, ?2)",
                [task_id, temp_tag.id],
            )?;
        }
        Ok(())
    }

    pub fn get_tags_for_task(
        &self,
        project_name: &str,
        task_name: &str,
    ) -> Result<Vec<Tag>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT tags.id, tags.name
             FROM tags
             JOIN task_tags ON tags.id = task_tags.tag_id
             JOIN tasks ON task_tags.task_id = tasks.id
             JOIN projects ON tasks.project_id = projects.id
             WHERE tasks.name = ?1 AND projects.name = ?2",
        )?;
        let tags = stmt.query_map([task_name, project_name], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;
        Ok(tags.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn get_tags_for_task_by_id(&self, task_id: i64) -> Result<Vec<Tag>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT tags.id, tags.name
             FROM tags
             JOIN task_tags ON tags.id = task_tags.tag_id
             WHERE task_tags.task_id = ?1",
        )?;
        let tags = stmt.query_map([task_id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;
        Ok(tags.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn remove_tag_from_task(
        &self,
        project_name: &str,
        task_name: &str,
        tag_name: &str,
    ) -> Result<bool, StoreError> {
        let task = self.get_task_by_name(project_name, task_name)?;
        let tag = self.get_tag_by_name(tag_name)?;
        let result = self.conn.execute(
            "DELETE FROM task_tags WHERE task_id = ?1 AND tag_id = ?2",
            [task.id, tag.id],
        )?;
        Ok(result > 0)
    }
}
