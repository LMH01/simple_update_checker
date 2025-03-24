use std::str::FromStr;

use anyhow::Result;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions, types::chrono::NaiveDateTime};

use crate::{Identifier, Program, Provider, UpdateCheck, UpdateCheckType};

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

    pub async fn update_latest_version(
        &self,
        name: &str,
        latest_version: &str,
        latest_version_last_updated: NaiveDateTime,
    ) -> Result<()> {
        let sql = r#"UPDATE programs SET latest_version = ?, latest_version_last_updated = ? WHERE name = ?"#;
        sqlx::query(sql)
            .bind(latest_version)
            .bind(latest_version_last_updated)
            .bind(name)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_current_version(
        &self,
        name: &str,
        current_version: &str,
        current_version_last_updated: NaiveDateTime,
    ) -> Result<()> {
        let sql = r#"UPDATE programs SET current_version = ?, current_version_last_updated = ? WHERE name = ?"#;
        sqlx::query(sql)
            .bind(current_version)
            .bind(current_version_last_updated)
            .bind(name)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn insert_update_check(&self, update_check: &UpdateCheck) -> Result<()> {
        let sql = r#"INSERT INTO update_checks (time, type) VALUES (?, ?)"#;
        sqlx::query(sql)
            .bind(update_check.time)
            .bind(update_check.r#type.identifier())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_latest_update_check(&self) -> Result<Option<UpdateCheck>> {
        let sql = r#"SELECT time, type FROM update_checks ORDER BY time DESC LIMIT 1"#;
        if let Some(row) = sqlx::query_as::<_, (NaiveDateTime, String)>(sql)
            .fetch_optional(&self.pool)
            .await?
        {
            return Ok(Some(UpdateCheck {
                time: row.0,
                r#type: UpdateCheckType::from_str(&row.1)
                    .expect("database should contain only valid entries"),
            }));
        }
        Ok(None)
    }

    pub async fn set_notification_sent(
        &self,
        program_name: &str,
        notification_sent: bool,
    ) -> Result<()> {
        let sql = r#"UPDATE programs SET notification_sent = ? WHERE name = ?"#;
        sqlx::query(sql)
            .bind(notification_sent)
            .bind(program_name)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_notification_sent(&self, program_name: &str) -> Result<Option<bool>> {
        let sql = r#"SELECT notification_sent FROM programs WHERE name = ?"#;
        if let Some((notification_send,)) = sqlx::query_as::<_, (bool,)>(sql)
            .bind(program_name)
            .fetch_optional(&self.pool)
            .await?
        {
            return Ok(Some(notification_send));
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {

    use sqlx::{
        SqlitePool,
        types::chrono::{NaiveDate, NaiveDateTime, NaiveTime},
    };

    use crate::{
        UpdateCheck, UpdateCheckType,
        db::{Program, Provider},
    };

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
        let new_latest_version_last_updated = NaiveDateTime::new(
            NaiveDate::parse_from_str("01.01.2025", "%d.%m.%Y").unwrap(),
            NaiveTime::parse_from_str("00:00:00", "%H:%M:%S").unwrap(),
        );
        program_db.add_program(&program).await.unwrap();
        program_db
            .update_latest_version(
                &program.name,
                "0.2.0",
                new_latest_version_last_updated.clone(),
            )
            .await
            .unwrap();
        let res = program_db
            .get_program(&program.name)
            .await
            .unwrap()
            .unwrap();
        program.latest_version = "0.2.0".to_string();
        program.latest_version_last_updated = new_latest_version_last_updated;
        assert_eq!(program, res);
    }

    #[sqlx::test]
    fn test_program_db_update_current_version(pool: SqlitePool) {
        let program_db = program_db(pool);
        let mut program = Program {
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
        let new_current_version_last_updated = NaiveDateTime::new(
            NaiveDate::parse_from_str("01.01.2025", "%d.%m.%Y").unwrap(),
            NaiveTime::parse_from_str("00:00:00", "%H:%M:%S").unwrap(),
        );
        program_db.add_program(&program).await.unwrap();
        program_db
            .update_current_version(&program.name, "0.2.0", new_current_version_last_updated)
            .await
            .unwrap();
        let res = program_db
            .get_program(&program.name)
            .await
            .unwrap()
            .unwrap();
        program.current_version = "0.2.0".to_string();
        program.current_version_last_updated = new_current_version_last_updated;
        assert_eq!(program, res);
    }

    #[sqlx::test]
    fn test_program_db_update_check(pool: SqlitePool) {
        let program_db = program_db(pool);
        let update_check = UpdateCheck {
            time: NaiveDateTime::new(
                NaiveDate::parse_from_str("10.03.2025", "%d.%m.%Y").unwrap(),
                NaiveTime::parse_from_str("10:50:00", "%H:%M:%S").unwrap(),
            ),
            r#type: UpdateCheckType::Manual,
        };
        let update_check1 = UpdateCheck {
            time: NaiveDateTime::new(
                NaiveDate::parse_from_str("12.03.2025", "%d.%m.%Y").unwrap(),
                NaiveTime::parse_from_str("13:45:00", "%H:%M:%S").unwrap(),
            ),
            r#type: UpdateCheckType::Manual,
        };
        program_db.insert_update_check(&update_check).await.unwrap();
        program_db
            .insert_update_check(&update_check1)
            .await
            .unwrap();
        let res = program_db.get_latest_update_check().await.unwrap();
        assert_eq!(Some(update_check1), res);
    }

    #[sqlx::test]
    fn test_program_db_update_check_not_existing(pool: SqlitePool) {
        let program_db = program_db(pool);
        let res = program_db.get_latest_update_check().await.unwrap();
        assert!(res.is_none())
    }

    #[sqlx::test]
    fn test_program_db_set_notification_sent(pool: SqlitePool) {
        let program_db = program_db(pool);
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
        program_db.add_program(&program).await.unwrap();
        program_db.add_program(&program2).await.unwrap();
        program_db
            .set_notification_sent("simple_update_checker", true)
            .await
            .unwrap();
        let res = program_db
            .get_notification_sent("simple_update_checker")
            .await
            .unwrap();
        assert_eq!(Some(true), res);
        let res = program_db
            .get_notification_sent("test_program")
            .await
            .unwrap();
        assert_eq!(Some(false), res);
    }

    #[sqlx::test]
    fn test_program_db_get_notification_sent_program_not_existing(pool: SqlitePool) {
        let program_db = program_db(pool);
        let res = program_db.get_notification_sent("name").await.unwrap();
        assert!(res.is_none())
    }
}
