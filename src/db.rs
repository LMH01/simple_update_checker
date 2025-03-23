use anyhow::Result;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

use crate::{Program, Provider};

pub struct ProgramDb {
    pub pool: SqlitePool,
}

impl ProgramDb {
    pub async fn connect(path: &str) -> Result<Self> {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true);
        let pool = SqlitePool::connect_lazy_with(options);
        // we try to create a test connection to see if the connection can be established
        let _ = pool.begin().await?;
        // if this was successful we know that the connection could be established
        tracing::debug!("Applying migrations");
        if let Err(e) = sqlx::migrate!().run(&pool).await {
            return Err(anyhow::anyhow!("Unable to apply migrations: {e}"));
        }
        Ok(Self { pool })
    }

    /// Add a program to the database.
    pub async fn add_program(&self, program: &Program) -> Result<()> {
        // insert into programs table
        let sql = r#"INSERT INTO programs ('name','current_version', 'latest_version' , 'provider') VALUES (?, ?, ?, ?)"#;
        let _ = sqlx::query(sql)
            .bind(&program.name)
            .bind(&program.current_version)
            .bind(&program.latest_version)
            .bind(program.provider.identifier())
            .fetch_all(&self.pool)
            .await?;
        // insert into provider specific table
        match &program.provider {
            Provider::Github(repository) => {
                let sql = r#"INSERT INTO github_programs ('name', 'repository') VALUES (?, ?)"#;
                let _ = sqlx::query(sql)
                    .bind(&program.name)
                    .bind(repository)
                    .fetch_all(&self.pool)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn remove_program(&self, name: &str) -> Result<()> {
        // Delete from provider-specific table first
        let sql = r#"DELETE FROM github_programs WHERE name = ?"#;
        sqlx::query(sql).bind(name).execute(&self.pool).await?;
        // Delete from main programs table
        let sql = r#"DELETE FROM programs WHERE name = ?"#;
        sqlx::query(sql).bind(name).execute(&self.pool).await?;

        Ok(())
    }

    /// Retrieve program form database. If name of program is no found, returns 'None'.
    pub async fn get_program(&self, name: &str) -> Result<Option<Program>> {
        // Retrieve the basic program details
        let sql = r#"SELECT name, current_version, latest_version, provider FROM programs WHERE name = ?"#;
        let row = sqlx::query_as::<_, (String, String, String, String)>(sql)
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        let Some((name, current_version, latest_version, provider)) = row else {
            return Ok(None);
        };

        // Determine the provider type and fetch additional data if needed
        let provider = match provider.as_str() {
            "github" => {
                let sql = r#"SELECT repository FROM github_programs WHERE name = ?"#;
                match sqlx::query_as::<_, (String,)>(sql)
                    .bind(&name)
                    .fetch_optional(&self.pool)
                    .await?
                {
                    Some((repository,)) => Provider::Github(repository),
                    _ => {
                        anyhow::bail!("Github repository entry missing for program: {}", name);
                    }
                }
            }
            _ => anyhow::bail!("Unknown provider type: {}", provider),
        };

        Ok(Some(Program {
            name,
            current_version,
            latest_version,
            provider,
        }))
    }

    /// Retrieve all programs from the database.
    pub async fn get_all_programs(&self) -> Result<Vec<Program>> {
        // Retrieve all programs
        let sql = r#"SELECT name, current_version, latest_version, provider FROM programs"#;
        let rows = sqlx::query_as::<_, (String, String, String, String)>(sql)
            .fetch_all(&self.pool)
            .await?;

        let mut programs = Vec::new();
        for (name, current_version, latest_version, provider) in rows {
            let provider = match provider.as_str() {
                "github" => {
                    let sql = r#"SELECT repository FROM github_programs WHERE name = ?"#;
                    match sqlx::query_as::<_, (String,)>(sql)
                        .bind(&name)
                        .fetch_optional(&self.pool)
                        .await?
                    {
                        Some((repository,)) => Provider::Github(repository),
                        _ => {
                            anyhow::bail!("Github repository entry missing for program: {}", name);
                        }
                    }
                }
                _ => anyhow::bail!("Unknown provider type: {}", provider),
            };

            programs.push(Program {
                name,
                current_version,
                latest_version,
                provider,
            });
        }

        Ok(programs)
    }

    pub async fn update_latest_version(&self, name: &str, latest_version: &str) -> Result<()> {
        let sql = r#"UPDATE programs SET latest_version = ? WHERE name = ?"#;
        sqlx::query(sql)
            .bind(latest_version)
            .bind(name)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_current_version(&self, name: &str, current_version: &str) -> Result<()> {
        let sql = r#"UPDATE programs SET current_version = ? WHERE name = ?"#;
        sqlx::query(sql)
            .bind(current_version)
            .bind(name)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::SqlitePool;

    use crate::db::{Program, Provider};

    use super::ProgramDb;

    fn program_db(pool: SqlitePool) -> ProgramDb {
        ProgramDb { pool }
    }

    #[sqlx::test]
    fn test_program_db(pool: SqlitePool) {
        let program_db = program_db(pool);
        let program = Program {
            name: "simple_update_checker".to_string(),
            current_version: "0.1.0".to_string(),
            latest_version: "0.1.0".to_string(),
            provider: Provider::Github("LMH01/simple_update_checker".to_string()),
        };
        let program2 = Program {
            name: "test_program".to_string(),
            current_version: "0.1.0".to_string(),
            latest_version: "0.1.0".to_string(),
            provider: Provider::Github("LMH01/test_program".to_string()),
        };
        program_db.add_program(&program).await.unwrap();
        let res = program_db.get_program(&program.name).await.unwrap();
        assert_eq!(Some(program), res);
        let res = program_db.get_program(&program2.name).await.unwrap();
        assert_eq!(None, res);
    }

    #[sqlx::test]
    fn test_program_db_remove_program(pool: SqlitePool) {
        let program_db = program_db(pool);
        let program = Program {
            name: "simple_update_checker".to_string(),
            current_version: "0.1.0".to_string(),
            latest_version: "0.1.0".to_string(),
            provider: Provider::Github("LMH01/simple_update_checker".to_string()),
        };
        program_db.add_program(&program).await.unwrap();
        program_db.remove_program(&program.name).await.unwrap();
        let res = program_db.get_program(&program.name).await.unwrap();
        assert_eq!(res, None);
    }

    #[sqlx::test]
    fn test_program_db_get_all_programs(pool: SqlitePool) {
        let program_db = program_db(pool);
        let program = Program {
            name: "simple_update_checker".to_string(),
            current_version: "0.1.0".to_string(),
            latest_version: "0.1.0".to_string(),
            provider: Provider::Github("LMH01/simple_update_checker".to_string()),
        };
        let program2 = Program {
            name: "test_program".to_string(),
            current_version: "0.1.0".to_string(),
            latest_version: "0.1.0".to_string(),
            provider: Provider::Github("LMH01/test_program".to_string()),
        };
        program_db.add_program(&program).await.unwrap();
        program_db.add_program(&program2).await.unwrap();
        let mut should = vec![program, program2];
        should.sort_by(|a, b| a.name.cmp(&b.name));
        let mut res = program_db.get_all_programs().await.unwrap();
        res.sort_by(|a, b| a.name.cmp(&b.name));
        assert_eq!(should, res);
    }

    #[sqlx::test]
    fn test_program_db_update_latest_version(pool: SqlitePool) {
        let program_db = program_db(pool);
        let mut program = Program {
            name: "simple_update_checker".to_string(),
            current_version: "0.1.0".to_string(),
            latest_version: "0.1.0".to_string(),
            provider: Provider::Github("LMH01/simple_update_checker".to_string()),
        };
        program_db.add_program(&program).await.unwrap();
        program_db
            .update_latest_version(&program.name, "0.2.0")
            .await
            .unwrap();
        let res = program_db
            .get_program(&program.name)
            .await
            .unwrap()
            .unwrap();
        program.latest_version = "0.2.0".to_string();
        assert_eq!(program, res);
    }

    #[sqlx::test]
    fn test_program_db_update_current_version(pool: SqlitePool) {
        let program_db = program_db(pool);
        let mut program = Program {
            name: "simple_update_checker".to_string(),
            current_version: "0.1.0".to_string(),
            latest_version: "0.1.0".to_string(),
            provider: Provider::Github("LMH01/simple_update_checker".to_string()),
        };
        program_db.add_program(&program).await.unwrap();
        program_db
            .update_current_version(&program.name, "0.2.0")
            .await
            .unwrap();
        let res = program_db
            .get_program(&program.name)
            .await
            .unwrap()
            .unwrap();
        program.current_version = "0.2.0".to_string();
        assert_eq!(program, res);
    }
}
