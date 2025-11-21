// src/main.rs
mod errors;
mod features;
mod infrastructure;
mod middleware;
mod models;
mod routes;
mod state;

use axum::serve;
use dashmap::DashMap;
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::features::auth::AuthService;
use crate::features::auth::repository::PostgresUserRepo;
use crate::infrastructure::db_bootstrap::ensure_database_exists;
use crate::state::{AppState, TenantConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();

    // --------------------------------------------------
    // Read environment: Master DB
    // --------------------------------------------------
    let master_db_name = env::var("MASTER_DB_NAME").expect("MASTER_DB_NAME missing");
    let master_db_url = env::var("MASTER_DB_URL").expect("MASTER_DB_URL missing");
    let super_url = env::var("POSTGRES_SUPER_URL").expect("POSTGRES_SUPER_URL missing");

    // Ensure master DB exists
    ensure_database_exists(&super_url, &master_db_name).await?;

    // Create master pool
    let master_pool: sqlx::Pool<sqlx::Postgres> = PgPoolOptions::new()
        .max_connections(10)
        .connect(&master_db_url)
        .await?;

    // Run migrations for master DB
    sqlx::migrate!("./migrations/master")
        .run(&master_pool)
        .await?;

    // --------------------------------------------------
    // Tenant configuration
    // --------------------------------------------------
    let tenant_config: TenantConfig = TenantConfig {
        user: env::var("TENANT_DB_USER").expect("TENANT_DB_USER required"),
        password: env::var("TENANT_DB_PASSWORD").expect("TENANT_DB_PASSWORD required"),
        host: env::var("TENANT_DB_HOST").unwrap_or("localhost".into()),
        port: env::var("TENANT_DB_PORT").unwrap_or("5432".into()),
        base_url: format!(
            "postgres://{}:{}@{}:{}/",
            urlencoding::encode(&env::var("TENANT_DB_USER").unwrap()),
            urlencoding::encode(&env::var("TENANT_DB_PASSWORD").unwrap()),
            env::var("TENANT_DB_HOST").unwrap_or("localhost".into()),
            env::var("TENANT_DB_PORT").unwrap_or("5432".into()),
        ),
    };

    // JWT secret
    let jwt_secret: String = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    // --------------------------------------------------
    // Build application state
    // --------------------------------------------------

    let app_state = Arc::new(AppState {
        master_pool,
        jwt_secret,
        tenant_pools: DashMap::new(),
        tenant_config,
        // auth_service,
    });
    let user_repo = PostgresUserRepo::new(master_pool.clone());
    let auth_service = Arc::new(AuthService::new(user_repo, app_state.clone()));

    // --------------------------------------------------
    // Start HTTP server
    // --------------------------------------------------
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let app = routes::create_router(app_state);
    let addr: SocketAddr = "127.0.0.1:8080".parse()?;

    println!("Listening on {}", addr);
    tracing::info!("Server running on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    serve(listener, app.into_make_service()).await?;

    Ok(())
}
