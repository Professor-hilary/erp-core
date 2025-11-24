// src/bin/init_master_db.rs
use sqlx::{PgPool, postgres::PgPoolOptions};

const MASTER_DB_NAME: &str = "master_db";
const DEFAULT_SUPER_URL: &str = "postgres://postgres:mJql0TD29mzZ@localhost:5432/postgres";

#[allow(unused)]
pub async fn init_master(super_url: &str, db_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to PostgreSQL cluster using default superuser...");

    let super_pool: sqlx::Pool<sqlx::Postgres> = PgPool::connect(DEFAULT_SUPER_URL).await.map_err(|e| {
        format!(
            "Failed to connect to PostgreSQL. Is it running?\n  Error: {}",
            e
        )
    })?;

    // 1. Create master database if not exists
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)")
            .bind(MASTER_DB_NAME)
            .fetch_one(&super_pool)
            .await?;

    if exists {
        println!("Master database '{}' already exists.", MASTER_DB_NAME);
    } else {
        println!("Creating master database '{}'...", MASTER_DB_NAME);
        sqlx::query(&format!(r#"CREATE DATABASE "{}""#, MASTER_DB_NAME))
            .execute(&super_pool)
            .await?;
        println!("Master database created.");
    }

    // 2. Create limited master DB user (app runtime user)
    let master_user: &str = "master_user";
    let master_pass: &str = "AodMzT6aEM0dX9HuZpyvChPsylI0ugcLmIIL"; // In prod: generate or use vault

    let user_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_roles WHERE rolname = $1)")
            .bind(master_user)
            .fetch_one(&super_pool)
            .await?;

    if user_exists {
        println!("User '{}' already exists.", master_user);
    } else {
        println!("Creating application master user '{}'...", master_user);
        sqlx::query(&format!(
            "CREATE USER {} WITH PASSWORD '{}'",
            master_user, master_pass
        ))
        .execute(&super_pool)
        .await?;
        sqlx::query(&format!(
            "GRANT ALL ON DATABASE \"{}\" TO {}",
            MASTER_DB_NAME, master_user
        ))
        .execute(&super_pool)
        .await?;
        println!("Master user created and granted access.");
    }

    // 4. Connect to master DB and run migrations
    let master_url = format!(
        "postgres://{}:{}@localhost/{}",
        master_user, master_pass, MASTER_DB_NAME
    );
    let master_pool: sqlx::Pool<sqlx::Postgres> = PgPoolOptions::new()
        .max_connections(10)
        .connect(&master_url)
        .await?;

    println!("Running master database migrations...");
    sqlx::migrate!("./migrations/master")
        .run(&master_pool)
        .await?;

    println!();
    println!("Setup complete!");
    println!("Your master database is ready: {}", MASTER_DB_NAME);
    println!("Start the API with: cargo run");
    println!();
    println!(
        "DATABASE_URL=postgres://{}:{}@localhost/{}",
        master_user, master_pass, MASTER_DB_NAME
    );

    Ok(())
}
