use anyhow::Result;

use crate::UpdateHistoryEntry;

use super::Db;

impl Db {
    /// Add an `UpdateHistoryEntry` to `update_history`.
    pub async fn insert_performed_update(
        &self,
        update_history_entry: &UpdateHistoryEntry,
    ) -> Result<()> {
        let sql =
            r"INSERT INTO update_history (date, name, old_version, updated_to) VALUES (?, ?, ?, ?)";
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
        let sql = r"SELECT date, name, old_version, updated_to FROM update_history ORDER BY date DESC LIMIT ?";
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

    use crate::{UpdateHistoryEntry, db::tests};

    #[sqlx::test]
    fn test_db_insert_performed_update(pool: SqlitePool) {
        let db = tests::db(pool);
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
        let db = tests::db(pool);
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

        let mut res = db.get_all_updates(None).await.unwrap();
        res.reverse();

        assert_eq!(vec![entry, entry2, entry3], res);
    }

    #[sqlx::test]
    fn test_db_get_all_updates_limited_returns(pool: SqlitePool) {
        let db = tests::db(pool);
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

        let mut res = db.get_all_updates(Some(2)).await.unwrap();
        res.reverse();

        assert_eq!(vec![entry2, entry3], res);
    }
}
