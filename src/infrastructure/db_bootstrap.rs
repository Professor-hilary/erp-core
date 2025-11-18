// src/infrustructure/db_bootstrap.rs
use sqlx::{Pool, postgres::PgPoolOptions};

/// # Run Master DB
/// Check existence of master db, if undefined or unexistent, attempt creating it.
pub async fn ensure_database_exists(
    super_url: &str,
    db_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let super_pool: Pool<sqlx::Postgres> = PgPoolOptions::new()
        .max_connections(5)
        .connect(super_url)
        .await?;

    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)")
            .bind(db_name)
            .fetch_one(&super_pool)
            .await?;

    if !exists {
        println!("Creating master database: {}", db_name);

        sqlx::query(format!("CREATE DATABASE {}", db_name).as_str())
            .execute(&super_pool)
            .await?;
    }

    Ok(())
}
