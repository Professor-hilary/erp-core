// src/bin/init_master_db.rs
use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load Postgres connection string to default DB (not master yet)
    let default_db_url = env::var("POSTGRES_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/postgres".into());

    let pool = PgPool::connect(&default_db_url).await?;

    // Check if master DB exists
    let (exists,): (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname='master')")
            .fetch_one(&pool)
            .await?;

    if exists {
        println!("Master database already exists. Skipping creation.");
    } else {
        println!("Creating master database...");
        sqlx::query("CREATE DATABASE master").execute(&pool).await?;
        println!("Master database created!");
    }

    // Connect to master DB to run migrations
    let master_db_url = default_db_url.replace("postgres", "master");
    let master_pool = PgPool::connect(&master_db_url).await?;

    // This migration is only once during setup
    println!("Running migrations for master database...");
    sqlx::migrate!("./migrations/master")
        .run(&master_pool)
        .await?;
    println!("Master database ready!");

    Ok(())
}
