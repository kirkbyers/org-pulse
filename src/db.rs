use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use anyhow::{Result};

const DB_URL: &str = "sqlite://org-pulse.db?mode=rwc";

pub async fn new_pool () -> Result<SqlitePool> {
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect(DB_URL)
        .await.map_err(|e| e.into())
    
}
