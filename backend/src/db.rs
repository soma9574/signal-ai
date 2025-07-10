use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

pub async fn init_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(5))
        .max_connections(5)
        .connect(database_url)
        .await?;
    // Run migrations at startup
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;
    Ok(pool)
} 