use anyhow::Result;
use sqlx::types::chrono::NaiveDateTime;

use crate::{Identifier, NotificationInfo, Program, Provider};

use super::Db;

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

    pub async fn set_notification_sent_on(
        &self,
        program_name: &str,
        notification_sent_on: Option<NaiveDateTime>,
    ) -> Result<()> {
        let sql = r#"UPDATE programs SET notification_sent_on = ? WHERE name = ?"#;
        sqlx::query(sql)
            .bind(notification_sent_on)
            .bind(program_name)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_notification_info(
        &self,
        program_name: &str,
    ) -> Result<Option<NotificationInfo>> {
        let sql = r#"SELECT notification_sent, notification_sent_on FROM programs WHERE name = ?"#;
        if let Some((sent, sent_on)) = sqlx::query_as::<_, (bool, Option<NaiveDateTime>)>(sql)
            .bind(program_name)
            .fetch_optional(&self.pool)
            .await?
        {
            return Ok(Some(NotificationInfo { sent, sent_on }));
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

    #[sqlx::test]
    fn test_db_update_latest_version(pool: SqlitePool) {
        let db = tests::db(pool);
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
        db.insert_program(&program).await.unwrap();
        db.update_latest_version(
            &program.name,
            "0.2.0",
            new_latest_version_last_updated.clone(),
        )
        .await
        .unwrap();
        let res = db.get_program(&program.name).await.unwrap().unwrap();
        program.latest_version = "0.2.0".to_string();
        program.latest_version_last_updated = new_latest_version_last_updated;
        assert_eq!(program, res);
    }

    #[sqlx::test]
    fn test_db_update_current_version(pool: SqlitePool) {
        let db = tests::db(pool);
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
        db.insert_program(&program).await.unwrap();
        db.update_current_version(&program.name, "0.2.0", new_current_version_last_updated)
            .await
            .unwrap();
        let res = db.get_program(&program.name).await.unwrap().unwrap();
        program.current_version = "0.2.0".to_string();
        program.current_version_last_updated = new_current_version_last_updated;
        assert_eq!(program, res);
    }

    #[sqlx::test]
    fn test_db_set_notification_sent(pool: SqlitePool) {
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
        db.insert_program(&program2).await.unwrap();
        db.set_notification_sent("simple_update_checker", true)
            .await
            .unwrap();
        let res = db
            .get_notification_info("simple_update_checker")
            .await
            .unwrap()
            .unwrap()
            .sent;
        assert_eq!(true, res);
        let res = db
            .get_notification_info("test_program")
            .await
            .unwrap()
            .unwrap()
            .sent;
        assert_eq!(false, res);
    }

    #[sqlx::test]
    fn test_db_set_notification_sent_on(pool: SqlitePool) {
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
        db.insert_program(&program2).await.unwrap();

        let test_date_time = NaiveDateTime::new(
            NaiveDate::parse_from_str("10.03.2025", "%d.%m.%Y").unwrap(),
            NaiveTime::parse_from_str("10:50:00", "%H:%M:%S").unwrap(),
        );

        db.set_notification_sent_on("simple_update_checker", Some(test_date_time.clone()))
            .await
            .unwrap();
        let res = db
            .get_notification_info("simple_update_checker")
            .await
            .unwrap()
            .unwrap()
            .sent_on;
        assert_eq!(Some(test_date_time), res);

        let res = db
            .get_notification_info("test_program")
            .await
            .unwrap()
            .unwrap()
            .sent_on;
        assert_eq!(None, res);

        db.set_notification_sent_on("simple_update_checker", None)
            .await
            .unwrap();

        let res = db
            .get_notification_info("simple_update_checker")
            .await
            .unwrap()
            .unwrap()
            .sent_on;
        assert_eq!(None, res);
    }

    #[sqlx::test]
    fn test_db_get_notification_sent_program_not_existing(pool: SqlitePool) {
        let db = tests::db(pool);
        let res = db.get_notification_info("name").await.unwrap();
        assert!(res.is_none())
    }
}
