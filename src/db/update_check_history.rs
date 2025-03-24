use std::str::FromStr;

use anyhow::Result;
use sqlx::types::chrono::NaiveDateTime;

use crate::{Identifier, UpdateCheckHistoryEntry, UpdateCheckType};

use super::Db;

impl Db {
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
}

#[cfg(test)]
mod tests {
    use sqlx::{
        SqlitePool,
        types::chrono::{NaiveDate, NaiveDateTime, NaiveTime},
    };

    use crate::{UpdateCheckHistoryEntry, UpdateCheckType, db::tests};

    #[sqlx::test]
    fn test_db_update_check(pool: SqlitePool) {
        let db = tests::db(pool);
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
        let db = tests::db(pool);
        let res = db.get_latest_update_check_from_history().await.unwrap();
        assert!(res.is_none())
    }

    #[sqlx::test]
    fn test_db_get_all_update_checks(pool: SqlitePool) {
        let db = tests::db(pool);
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
}
