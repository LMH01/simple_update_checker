use anyhow::Result;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

mod programs;
mod update_check_history;
mod update_history;

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
}

#[cfg(test)]
mod tests {

    use sqlx::SqlitePool;

    use super::Db;

    pub fn db(pool: SqlitePool) -> Db {
        Db { pool }
    }
}
