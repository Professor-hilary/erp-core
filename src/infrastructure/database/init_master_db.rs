// src/infrastructure/database/init_master_db.rs

use sqlx::{PgPool, migrate::Migrator, postgres::PgPoolOptions};
use std::error::Error;

pub async fn init_master(
    super_psql_url: &str,
    master_db_name: &str,
    master_db_user: &str,
    master_db_pass: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Connecting to PostgreSQL as superuser...");
    let super_pool: PgPool = PgPool::connect(super_psql_url)
        .await
        .map_err(|e| format!("Failed to connect as superuser: {e}"))?;

    // 1. Ensure database exists
    let db_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)")
            .bind(master_db_name)
            .fetch_one(&super_pool)
            .await?;

    if !db_exists {
        println!("Creating database '{master_db_name}'...");
        sqlx::query(&format!("CREATE DATABASE \"{master_db_name}\""))
            .execute(&super_pool)
            .await?;
    } else {
        println!("Database '{master_db_name}' already exists.");
    }

    // 2. Ensure role exists
    let user_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_roles WHERE rolname = $1)")
            .bind(master_db_user)
            .fetch_one(&super_pool)
            .await?;

    if !user_exists {
        println!("Creating role '{master_db_user}'...");
        sqlx::query(&format!(
            "CREATE ROLE {master_db_user} WITH LOGIN PASSWORD '{master_db_pass}'"
        ))
        .execute(&super_pool)
        .await?;
    } else {
        // Safely update password on re-run (optional but nice)
        println!("Updating password for '{master_db_user}' (idempotent)...");
        sqlx::query(&format!(
            "ALTER ROLE {master_db_user} WITH PASSWORD '{master_db_pass}'"
        ))
        .execute(&super_pool)
        .await?;
    }

    // 3. Ownership & privileges — ALL done as superuser
    println!("Setting ownership and privileges...");

    let queries: [String; 5] = [
        format!("GRANT ALL PRIVILEGES ON DATABASE \"{master_db_name}\" TO {master_db_user}"),
        format!("ALTER DATABASE \"{master_db_name}\" OWNER TO {master_db_user}"),
        format!("ALTER SCHEMA public OWNER TO {master_db_user}"),
        format!("GRANT ALL ON SCHEMA public TO {master_db_user}"),
        // Future objects in public will belong to master_user by default
        format!("ALTER ROLE {master_db_user} SET search_path = public"),
    ];

    for query in queries {
        // Ignore errors like "already owner" — PostgreSQL doesn't have IF NOT EXISTS for these
        let _ = sqlx::query(&query).execute(&super_pool).await;
    }

    // 4. Connect as the app user and run migrations
    let master_url: String =
        format!("postgres://{master_db_user}:{master_db_pass}@localhost:5432/{master_db_name}");

    println!("Connecting as '{master_db_user}' to run migrations...");
    let master_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&master_url)
        .await
        .map_err(|e| {
            format!("Failed to connect as app user. Did ownership transfer succeed?\n{e}")
        })?;

    println!("Running migrations...");
    static MIGRATOR: Migrator = sqlx::migrate!("./migrations/master");
    MIGRATOR.run(&master_pool).await?;

    println!("Master database setup complete!");
    println!("Connection string: {master_url}");

    Ok(())
}
