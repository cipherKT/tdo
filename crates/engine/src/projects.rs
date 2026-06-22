use rusqlite::Result;

use crate::{Engine, Project, ProjectPatch, StoreError};

impl Engine {
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

    pub(crate) fn get_project_by_id(&self, id: i64) -> Result<Project, StoreError> {
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

    pub(crate) fn get_project_by_name(&self, name: &str) -> Result<Project, StoreError> {
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

    pub fn project_names(&self) -> Result<Vec<String>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT name FROM projects ORDER BY name ASC")?;
        let names = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;
        Ok(names)
    }
}
