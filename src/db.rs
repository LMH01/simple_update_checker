use anyhow::Result;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};

pub struct ProgramDb {
    pub pool: SqlitePool,
}

#[derive(PartialEq, Debug)]
pub struct Program {
    name: String,
    latest_version: String,
    provider: Provider,
}

#[derive(PartialEq, Debug)]
pub enum Provider {
    // String contains the gihub repository. For example: LMH01/simple_update_checker
    Github(String),
}

impl Provider {
    fn identifier(&self) -> String {
        match self {
            Self::Github(_) => "github".to_string(),
        }
    }
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
        let sql = r#"INSERT INTO programs ('name', 'latest_version', 'provider') VALUES (?, ?, ?)"#;
        let _ = sqlx::query(sql)
            .bind(&program.name)
            .bind(&program.latest_version)
            .bind(&program.provider.identifier())
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

    /// Retrieve program form database. If name of program is no found, returns 'None'.
    pub async fn get_program(&self, name: &str) -> Result<Option<Program>> {
        // Retrieve the basic program details
        let sql = r#"SELECT name, latest_version, provider FROM programs WHERE name = ?"#;
        let row = sqlx::query_as::<_, (String, String, String)>(sql)
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        let Some((name, latest_version, provider)) = row else {
            return Ok(None);
        };

        // Determine the provider type and fetch additional data if needed
        let provider = match provider.as_str() {
            "github" => {
                let sql = r#"SELECT repository FROM github_programs WHERE name = ?"#;
                if let Some((repository,)) = sqlx::query_as::<_, (String,)>(sql)
                    .bind(&name)
                    .fetch_optional(&self.pool)
                    .await?
                {
                    Provider::Github(repository)
                } else {
                    anyhow::bail!("Github repository entry missing for program: {}", name);
                }
            }
            _ => anyhow::bail!("Unknown provider type: {}", provider),
        };

        Ok(Some(Program {
            name,
            latest_version,
            provider,
        }))
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
    fn test_songdb(pool: SqlitePool) {
        let program_db = program_db(pool);
        let program = Program {
            name: "simple_update_checker".to_string(),
            latest_version: "0.1.0".to_string(),
            provider: Provider::Github("LMH01/simple_update_checker".to_string()),
        };
        let program2 = Program {
            name: "test_program".to_string(),
            latest_version: "0.1.0".to_string(),
            provider: Provider::Github("LMH01/test_program".to_string()),
        };
        program_db.add_program(&program).await.unwrap();
        //program_db.add_song("TestId").await.unwrap();
        let res = program_db.get_program(&program.name).await.unwrap();
        assert_eq!(Some(program), res);
        let res = program_db.get_program(&program2.name).await.unwrap();
        assert_eq!(None, res);
    }
}
