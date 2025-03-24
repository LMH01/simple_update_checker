use anyhow::Result;
use sqlx::types::chrono::NaiveDateTime;

use crate::{Identifier, Program, Provider};

use super::Db;

mod notification;
mod version;

impl Db {
    /// Add a program to the database.
    pub async fn insert_program(&self, program: &Program) -> Result<()> {
        // insert into programs table
        let sql = r#"INSERT INTO programs ('name','current_version', 'current_version_last_updated', 'latest_version', 'latest_version_last_updated' , 'provider') VALUES (?, ?, ?, ?, ?, ?)"#;
        let _ = sqlx::query(sql)
            .bind(&program.name)
            .bind(&program.current_version)
            .bind(program.current_version_last_updated)
            .bind(&program.latest_version)
            .bind(program.latest_version_last_updated)
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
        // First determine what provider the program belongs to
        let program = match self.get_program(name).await? {
            Some(program) => program,
            None => anyhow::bail!("Program named {name} does not exist"),
        };
        match program.provider {
            Provider::Github(_) => {
                let sql = r#"DELETE FROM github_programs WHERE name = ?"#;
                sqlx::query(sql).bind(name).execute(&self.pool).await?;
            }
        }
        // Delete from main programs table
        let sql = r#"DELETE FROM programs WHERE name = ?"#;
        sqlx::query(sql).bind(name).execute(&self.pool).await?;

        Ok(())
    }

    /// Retrieve program form database. If name of program is no found, returns 'None'.
    pub async fn get_program(&self, name: &str) -> Result<Option<Program>> {
        // Retrieve the basic program details
        let sql = r#"SELECT name, current_version, current_version_last_updated, latest_version, latest_version_last_updated, provider FROM programs WHERE name = ?"#;
        let row =
            sqlx::query_as::<_, (String, String, NaiveDateTime, String, NaiveDateTime, String)>(
                sql,
            )
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        let Some((
            name,
            current_version,
            current_version_last_updated,
            latest_version,
            latest_version_last_updated,
            provider,
        )) = row
        else {
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
            current_version_last_updated,
            latest_version,
            latest_version_last_updated,
            provider,
        }))
    }

    /// Retrieve all programs from the database.
    pub async fn get_all_programs(&self) -> Result<Vec<Program>> {
        // Retrieve all programs
        let sql = r#"SELECT name, current_version, current_version_last_updated, latest_version, latest_version_last_updated, provider FROM programs"#;
        let rows = sqlx::query_as::<
            _,
            (String, String, NaiveDateTime, String, NaiveDateTime, String),
        >(sql)
        .fetch_all(&self.pool)
        .await?;

        let mut programs = Vec::new();
        for (
            name,
            current_version,
            current_version_last_updated,
            latest_version,
            latest_version_last_updated,
            provider,
        ) in rows
        {
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
                current_version_last_updated,
                latest_version,
                latest_version_last_updated,
                provider,
            });
        }

        Ok(programs)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{
        SqlitePool,
        types::chrono::{NaiveDate, NaiveDateTime, NaiveTime},
    };

    use crate::{Program, Provider, db::tests};

    #[sqlx::test]
    fn test_db_programs(pool: SqlitePool) {
        let db = tests::db(pool);
        let program = Program {
            name: "simple_update_checker".to_string(),
            current_version: "0.1.0".to_string(),
            current_version_last_updated: NaiveDateTime::new(
                NaiveDate::parse_from_str("10.03.2025", "%d.%m.%Y").unwrap(),
                NaiveTime::parse_from_str("10:50:00", "%H:%M:%S").unwrap(),
            ),
            latest_version: "0.1.0".to_string(),
            latest_version_last_updated: NaiveDateTime::new(
                NaiveDate::parse_from_str("12.03.2025", "%d.%m.%Y").unwrap(),
                NaiveTime::parse_from_str("13:45:00", "%H:%M:%S").unwrap(),
            ),
            provider: Provider::Github("LMH01/simple_update_checker".to_string()),
        };
        let program2 = Program {
            name: "test_program".to_string(),
            current_version: "0.1.0".to_string(),
            current_version_last_updated: NaiveDateTime::new(
                NaiveDate::parse_from_str("10.03.2025", "%d.%m.%Y").unwrap(),
                NaiveTime::parse_from_str("10:50:00", "%H:%M:%S").unwrap(),
            ),
            latest_version_last_updated: NaiveDateTime::new(
                NaiveDate::parse_from_str("12.03.2025", "%d.%m.%Y").unwrap(),
                NaiveTime::parse_from_str("13:45:00", "%H:%M:%S").unwrap(),
            ),
            latest_version: "0.1.0".to_string(),
            provider: Provider::Github("LMH01/test_program".to_string()),
        };
        db.insert_program(&program).await.unwrap();
        let res = db.get_program(&program.name).await.unwrap();
        assert_eq!(Some(program), res);
        let res = db.get_program(&program2.name).await.unwrap();
        assert_eq!(None, res);
    }

    #[sqlx::test]
    fn test_db_remove_program(pool: SqlitePool) {
        let db = tests::db(pool);
        let program = Program {
            name: "simple_update_checker".to_string(),
            current_version: "0.1.0".to_string(),
            current_version_last_updated: NaiveDateTime::new(
                NaiveDate::parse_from_str("10.03.2025", "%d.%m.%Y").unwrap(),
                NaiveTime::parse_from_str("10:50:00", "%H:%M:%S").unwrap(),
            ),
            latest_version: "0.1.0".to_string(),
            latest_version_last_updated: NaiveDateTime::new(
                NaiveDate::parse_from_str("12.03.2025", "%d.%m.%Y").unwrap(),
                NaiveTime::parse_from_str("13:45:00", "%H:%M:%S").unwrap(),
            ),
            provider: Provider::Github("LMH01/simple_update_checker".to_string()),
        };
        db.insert_program(&program).await.unwrap();
        db.remove_program(&program.name).await.unwrap();
        let res = db.get_program(&program.name).await.unwrap();
        assert_eq!(res, None);
    }

    #[sqlx::test]
    fn test_db_get_all_programs(pool: SqlitePool) {
        let db = tests::db(pool);
        let program = Program {
            name: "simple_update_checker".to_string(),
            current_version: "0.1.0".to_string(),
            current_version_last_updated: NaiveDateTime::new(
                NaiveDate::parse_from_str("10.03.2025", "%d.%m.%Y").unwrap(),
                NaiveTime::parse_from_str("10:50:00", "%H:%M:%S").unwrap(),
            ),
            latest_version: "0.1.0".to_string(),
            latest_version_last_updated: NaiveDateTime::new(
                NaiveDate::parse_from_str("12.03.2025", "%d.%m.%Y").unwrap(),
                NaiveTime::parse_from_str("13:45:00", "%H:%M:%S").unwrap(),
            ),
            provider: Provider::Github("LMH01/simple_update_checker".to_string()),
        };
        let program2 = Program {
            name: "test_program".to_string(),
            current_version: "0.1.0".to_string(),
            current_version_last_updated: NaiveDateTime::new(
                NaiveDate::parse_from_str("10.03.2025", "%d.%m.%Y").unwrap(),
                NaiveTime::parse_from_str("10:50:00", "%H:%M:%S").unwrap(),
            ),
            latest_version: "0.1.0".to_string(),
            latest_version_last_updated: NaiveDateTime::new(
                NaiveDate::parse_from_str("12.03.2025", "%d.%m.%Y").unwrap(),
                NaiveTime::parse_from_str("13:45:00", "%H:%M:%S").unwrap(),
            ),
            provider: Provider::Github("LMH01/test_program".to_string()),
        };
        db.insert_program(&program).await.unwrap();
        db.insert_program(&program2).await.unwrap();
        let mut should = vec![program, program2];
        should.sort_by(|a, b| a.name.cmp(&b.name));
        let mut res = db.get_all_programs().await.unwrap();
        res.sort_by(|a, b| a.name.cmp(&b.name));
        assert_eq!(should, res);
    }
}