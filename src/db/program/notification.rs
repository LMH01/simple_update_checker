use anyhow::Result;
use sqlx::types::chrono::NaiveDateTime;

use crate::{NotificationInfo, db::Db};

impl Db {
    pub async fn set_notification_sent(
        &self,
        program_name: &str,
        notification_sent: bool,
    ) -> Result<()> {
        let sql = r"UPDATE programs SET notification_sent = ? WHERE name = ?";
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
        let sql = r"UPDATE programs SET notification_sent_on = ? WHERE name = ?";
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
        let sql = r"SELECT notification_sent, notification_sent_on FROM programs WHERE name = ?";
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
