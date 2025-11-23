// use serde_json::json;
// src/bin/init_master_db.rs
use sqlx::{PgPool, postgres::PgPoolOptions};

const MASTER_DB_NAME: &str = "master_accountant_db";
const DEFAULT_SUPER_URL: &str = "postgres://postgres:postgres@localhost:5432/postgres";

#[allow(unused)]
async fn init_master() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to PostgreSQL cluster using default superuser...");

    let super_pool = PgPool::connect(DEFAULT_SUPER_URL).await.map_err(|e| {
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
    let master_user: &str = "master_accountant";
    let master_pass: &str = "app_master_pass_123"; // In prod: generate or use vault

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
    let master_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&master_url)
        .await?;

    println!("Running master database migrations...");
    sqlx::migrate!("./migrations/master")
        .run(&master_pool)
        .await?;

    // // Seed the tenant provisioner credentials into master DB
    // let provisioner_user = "tenant_provisioner";
    // let provisioner_pass = uuid::Uuid::new_v4().to_string(); // Random per deployment
    //
    // // Create the actual DB role with CREATEDB
    // sqlx::query(&format!(
    //     "CREATE ROLE {} WITH LOGIN PASSWORD '{}' CREATEDB",
    //     provisioner_user, provisioner_pass
    // ))
    // .execute(&super_pool)
    // .await
    // .ok(); // Idempotent
    //
    // // Store encrypted (or raw for now — use pgcrypto + KMS in prod)
    // let encrypted = provisioner_pass.clone(); // placeholder
    //
    // // Ensure the table exists (defensive)
    // sqlx::query(
    //     r#"
    // CREATE TABLE IF NOT EXISTS tenant_secrets (
    //     company_id uuid PRIMARY KEY,
    //     secret jsonb NOT NULL,
    //     created_at timestamptz DEFAULT now()
    // )
    // "#,
    // )
    // .execute(&master_pool)
    // .await
    // .ok(); // ignore error — table probably already exists from migration
    //
    // // Now safely insert
    // let secret_json = json!({
    //     "provisioner_username": provisioner_user,
    //     "provisioner_password": encrypted,
    //     "base_connection": "postgres://localhost:5432"
    // });
    //
    // sqlx::query(
    //     r#"
    // INSERT INTO tenant_secrets (company_id, secret)
    // VALUES ('00000000-0000-0000-0000-000000000000', $1)
    // ON CONFLICT (company_id) DO UPDATE SET secret = EXCLUDED.secret
    // "#,
    // )
    // .bind(&secret_json)
    // .execute(&master_pool)
    // .await
    // .expect("Failed to store tenant provisioner secrets — this is critical!");

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
