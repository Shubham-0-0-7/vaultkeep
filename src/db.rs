use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;
use tracing::info;

pub async fn init_db_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    info!("Connection to PostgresSQL database ...");

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(30))
        .connect(database_url)
        .await?;

    info!("Running database migrations ...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    info!("Database migrations executed successfully.");
    Ok(pool)
}