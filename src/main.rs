// main.rs
mod errors;
mod features;
mod infrastructure;
mod middleware;
mod models;
mod routes;

use axum::serve;
use dashmap::DashMap;
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

use routes::{AppState, create_router};

use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::infrastructure::db_bootstrap::ensure_database_exists;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();
    // let database_url: String = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Attempt to connect to master database if it exists
    let master_db_name: String = env::var("MASTER_DB_NAME").expect("MASTER_DB_NAME missing");
    let master_db_url: String = env::var("MASTER_DB_URL").expect("MASTER_DB_URL missing");
    let postgres_super_url: String =
        env::var("POSTGRES_SUPER_URL").expect("POSTGRES_SUPER_URL missing");

    // -------------------------------------------------------------------------------
    // 1. Ensure MASTER DATABASE EXISTS
    // -------------------------------------------------------------------------------
    ensure_database_exists(&postgres_super_url, &master_db_name).await?;

    // -------------------------------------------------------------------------------
    // 2. Create master pool
    // -------------------------------------------------------------------------------
    let master_pool: sqlx::Pool<sqlx::Postgres> = PgPoolOptions::new()
        .max_connections(10)
        .connect(&master_db_url)
        .await?;

    // -------------------------------------------------------------------------------
    // 3. Run MASTER migrations
    // -------------------------------------------------------------------------------
    sqlx::migrate!("./migrations/master")
        .run(&master_pool)
        .await?;

    // -------------------------------------------------------------------------------
    // 4. Launch server
    // -------------------------------------------------------------------------------

    // Initialize logger
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    // Run migrations on startup (in prod, use separate process)
    let jwt_secret: String = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let state: Arc<AppState> = Arc::new(AppState {
        master_pool,
        jwt_secret,
        tenant_pools: DashMap::new(),
    });

    let app: axum::Router = create_router(state);

    let addr: SocketAddr = "127.0.0.1:8080".parse()?;
    println!("Listening on {}", addr);
    tracing::info!("Server running on {}", addr);

    let listener: TcpListener = TcpListener::bind(addr).await?;
    serve(listener, app.into_make_service()).await?;

    Ok(())
}
