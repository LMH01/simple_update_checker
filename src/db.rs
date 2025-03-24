use std::str::FromStr;

use anyhow::Result;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions, types::chrono::NaiveDateTime};

use crate::{
    Identifier, NotificationInfo, Program, Provider, UpdateCheckHistoryEntry, UpdateCheckType,
    UpdateHistoryEntry,
};

pub struct Db {
    pub pool: SqlitePool,
}

impl Db {
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

    pub async fn insert_update_check_history(
        &self,
        update_check: &UpdateCheckHistoryEntry,
    ) -> Result<()> {
        let sql = r#"INSERT INTO update_check_history (date, type, updates_available, programs) VALUES (?, ?, ?, ?)"#;
        sqlx::query(sql)
            .bind(update_check.date)
            .bind(update_check.r#type.identifier())
            .bind(update_check.updates_available)
            .bind(&update_check.programs)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_latest_update_check_from_history(
        &self,
    ) -> Result<Option<UpdateCheckHistoryEntry>> {
        let sql = r#"SELECT date, type, updates_available, programs FROM update_check_history ORDER BY date DESC LIMIT 1"#;
        if let Some((date, r#type, updates_available, programs)) =
            sqlx::query_as::<_, (NaiveDateTime, String, u32, String)>(sql)
                .fetch_optional(&self.pool)
                .await?
        {
            return Ok(Some(UpdateCheckHistoryEntry {
                date,
                r#type: UpdateCheckType::from_str(&r#type)
                    .expect("database should contain only valid entries"),
                updates_available,
                programs,
            }));
        }
        Ok(None)
    }

    pub async fn get_all_update_checks(
        &self,
        max_entries: Option<u32>,
    ) -> Result<Vec<UpdateCheckHistoryEntry>> {
        let sql = r#"SELECT date, type, updates_available, programs FROM update_check_history ORDER BY date DESC LIMIT ?"#;
        let update_checks = sqlx::query_as::<_, (NaiveDateTime, String, u32, String)>(sql)
            .bind(max_entries.unwrap_or(100))
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(
                |(date, r#type, updates_available, programs)| UpdateCheckHistoryEntry {
                    date,
                    r#type: UpdateCheckType::from_str(&r#type).expect(
                        "Database should contain string that can be parsed to UpdateCheckType",
                    ),
                    updates_available,
                    programs,
                },
            )
            .collect();
        Ok(update_checks)
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

    /// Add an UpdateHistoryEntry to update_history.
    pub async fn insert_performed_update(
        &self,
        update_history_entry: &UpdateHistoryEntry,
    ) -> Result<()> {
        let sql = r#"INSERT INTO update_history (date, name, old_version, updated_to) VALUES (?, ?, ?, ?)"#;
        sqlx::query(sql)
            .bind(update_history_entry.date)
            .bind(&update_history_entry.name)
            .bind(&update_history_entry.old_version)
            .bind(&update_history_entry.updated_to)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_all_updates(
        &self,
        max_entries: Option<u32>,
    ) -> Result<Vec<UpdateHistoryEntry>> {
        let sql = r#"SELECT date, name, old_version, updated_to FROM update_history ORDER BY date ASC LIMIT ?"#;
        let entries = sqlx::query_as::<_, UpdateHistoryEntry>(sql)
            .bind(max_entries.unwrap_or(100))
            .fetch_all(&self.pool)
            .await?;

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {

    use sqlx::{
        SqlitePool,
        types::chrono::{NaiveDate, NaiveDateTime, NaiveTime},
    };

    use crate::{
        UpdateCheckHistoryEntry, UpdateCheckType, UpdateHistoryEntry,
        db::{Program, Provider},
    };

    use super::Db;

    fn db(pool: SqlitePool) -> Db {
        Db { pool }
    }

    #[sqlx::test]
    fn test_db(pool: SqlitePool) {
        let db = db(pool);
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
        let db = db(pool);
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
        let db = db(pool);
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
        let db = db(pool);
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
        let db = db(pool);
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
    fn test_db_update_check(pool: SqlitePool) {
        let db = db(pool);
        let update_check = UpdateCheckHistoryEntry {
            date: NaiveDateTime::new(
                NaiveDate::parse_from_str("10.03.2025", "%d.%m.%Y").unwrap(),
                NaiveTime::parse_from_str("10:50:00", "%H:%M:%S").unwrap(),
            ),
            r#type: UpdateCheckType::Manual,
            updates_available: 0,
            programs: "".to_string(),
        };
        let update_check1 = UpdateCheckHistoryEntry {
            date: NaiveDateTime::new(
                NaiveDate::parse_from_str("12.03.2025", "%d.%m.%Y").unwrap(),
                NaiveTime::parse_from_str("13:45:00", "%H:%M:%S").unwrap(),
            ),
            r#type: UpdateCheckType::Manual,
            updates_available: 2,
            programs: "alpha_tui, simple_update_checker".to_string(),
        };
        db.insert_update_check_history(&update_check).await.unwrap();
        db.insert_update_check_history(&update_check1)
            .await
            .unwrap();
        let res = db.get_latest_update_check_from_history().await.unwrap();
        assert_eq!(Some(update_check1), res);
    }

    #[sqlx::test]
    fn test_program_db_update_check_not_existing(pool: SqlitePool) {
        let db = db(pool);
        let res = db.get_latest_update_check_from_history().await.unwrap();
        assert!(res.is_none())
    }

    #[sqlx::test]
    fn test_db_get_all_update_checks(pool: SqlitePool) {
        let db = db(pool);
        let entry = UpdateCheckHistoryEntry {
            date: NaiveDateTime::new(
                NaiveDate::parse_from_str("12.03.2025", "%d.%m.%Y").unwrap(),
                NaiveTime::parse_from_str("13:45:00", "%H:%M:%S").unwrap(),
            ),
            r#type: UpdateCheckType::Manual,
            updates_available: 0,
            programs: "".to_string(),
        };
        let entry2 = UpdateCheckHistoryEntry {
            date: NaiveDateTime::new(
                NaiveDate::parse_from_str("13.03.2025", "%d.%m.%Y").unwrap(),
                NaiveTime::parse_from_str("13:45:00", "%H:%M:%S").unwrap(),
            ),
            r#type: UpdateCheckType::Manual,
            updates_available: 0,
            programs: "".to_string(),
        };
        let entry3 = UpdateCheckHistoryEntry {
            date: NaiveDateTime::new(
                NaiveDate::parse_from_str("14.03.2025", "%d.%m.%Y").unwrap(),
                NaiveTime::parse_from_str("13:45:00", "%H:%M:%S").unwrap(),
            ),
            r#type: UpdateCheckType::Manual,
            updates_available: 0,
            programs: "".to_string(),
        };
        db.insert_update_check_history(&entry).await.unwrap();
        db.insert_update_check_history(&entry2).await.unwrap();
        db.insert_update_check_history(&entry3).await.unwrap();

        let mut res = db.get_all_update_checks(None).await.unwrap();
        res.reverse();

        assert_eq!(vec![entry, entry2, entry3], res);
    }

    #[sqlx::test]
    fn test_db_set_notification_sent(pool: SqlitePool) {
        let db = db(pool);
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
        let db = db(pool);
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
        let db = db(pool);
        let res = db.get_notification_info("name").await.unwrap();
        assert!(res.is_none())
    }

    #[sqlx::test]
    fn test_db_insert_performed_update(pool: SqlitePool) {
        let db = db(pool);
        let entry = UpdateHistoryEntry {
            date: NaiveDateTime::new(
                NaiveDate::parse_from_str("12.03.2025", "%d.%m.%Y").unwrap(),
                NaiveTime::parse_from_str("13:45:00", "%H:%M:%S").unwrap(),
            ),
            name: "alpha_tui".to_string(),
            old_version: "1.0.0".to_string(),
            updated_to: "1.1.0".to_string(),
        };
        db.insert_performed_update(&entry).await.unwrap();

        let res = db.get_all_updates(None).await.unwrap();

        assert_eq!(entry, res[0]);
    }

    #[sqlx::test]
    fn test_db_get_all_updates(pool: SqlitePool) {
        let db = db(pool);
        let entry = UpdateHistoryEntry {
            date: NaiveDateTime::new(
                NaiveDate::parse_from_str("12.03.2025", "%d.%m.%Y").unwrap(),
                NaiveTime::parse_from_str("13:45:00", "%H:%M:%S").unwrap(),
            ),
            name: "alpha_tui".to_string(),
            old_version: "1.0.0".to_string(),
            updated_to: "1.1.0".to_string(),
        };
        let entry2 = UpdateHistoryEntry {
            date: NaiveDateTime::new(
                NaiveDate::parse_from_str("13.03.2025", "%d.%m.%Y").unwrap(),
                NaiveTime::parse_from_str("13:45:00", "%H:%M:%S").unwrap(),
            ),
            name: "alpha_tui".to_string(),
            old_version: "1.1.0".to_string(),
            updated_to: "1.2.0".to_string(),
        };
        let entry3 = UpdateHistoryEntry {
            date: NaiveDateTime::new(
                NaiveDate::parse_from_str("14.03.2025", "%d.%m.%Y").unwrap(),
                NaiveTime::parse_from_str("13:45:00", "%H:%M:%S").unwrap(),
            ),
            name: "alpha_tui".to_string(),
            old_version: "1.2.0".to_string(),
            updated_to: "1.3.0".to_string(),
        };
        db.insert_performed_update(&entry).await.unwrap();
        db.insert_performed_update(&entry2).await.unwrap();
        db.insert_performed_update(&entry3).await.unwrap();

        let res = db.get_all_updates(None).await.unwrap();

        assert_eq!(vec![entry, entry2, entry3], res);
    }

    #[sqlx::test]
    fn test_db_get_all_updates_limited_returns(pool: SqlitePool) {
        let db = db(pool);
        let entry = UpdateHistoryEntry {
            date: NaiveDateTime::new(
                NaiveDate::parse_from_str("12.03.2025", "%d.%m.%Y").unwrap(),
                NaiveTime::parse_from_str("13:45:00", "%H:%M:%S").unwrap(),
            ),
            name: "alpha_tui".to_string(),
            old_version: "1.0.0".to_string(),
            updated_to: "1.1.0".to_string(),
        };
        let entry2 = UpdateHistoryEntry {
            date: NaiveDateTime::new(
                NaiveDate::parse_from_str("13.03.2025", "%d.%m.%Y").unwrap(),
                NaiveTime::parse_from_str("13:45:00", "%H:%M:%S").unwrap(),
            ),
            name: "alpha_tui".to_string(),
            old_version: "1.1.0".to_string(),
            updated_to: "1.2.0".to_string(),
        };
        let entry3 = UpdateHistoryEntry {
            date: NaiveDateTime::new(
                NaiveDate::parse_from_str("14.03.2025", "%d.%m.%Y").unwrap(),
                NaiveTime::parse_from_str("13:45:00", "%H:%M:%S").unwrap(),
            ),
            name: "alpha_tui".to_string(),
            old_version: "1.2.0".to_string(),
            updated_to: "1.3.0".to_string(),
        };
        db.insert_performed_update(&entry).await.unwrap();
        db.insert_performed_update(&entry2).await.unwrap();
        db.insert_performed_update(&entry3).await.unwrap();

        let res = db.get_all_updates(Some(2)).await.unwrap();

        assert_eq!(vec![entry, entry2], res);
    }
}
