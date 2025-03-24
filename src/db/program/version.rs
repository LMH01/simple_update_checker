use anyhow::Result;
use sqlx::types::chrono::NaiveDateTime;

use crate::db::Db;

impl Db {
    pub async fn update_latest_version(
        &self,
        name: &str,
        latest_version: &str,
        latest_version_last_updated: NaiveDateTime,
    ) -> Result<()> {
        let sql = r"UPDATE programs SET latest_version = ?, latest_version_last_updated = ? WHERE name = ?";
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
        let sql = r"UPDATE programs SET current_version = ?, current_version_last_updated = ? WHERE name = ?";
        sqlx::query(sql)
            .bind(current_version)
            .bind(current_version_last_updated)
            .bind(name)
            .execute(&self.pool)
            .await?;

        Ok(())
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
}
