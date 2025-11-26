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
use std::{env, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::infrastructure::database::init_master_db::init_master;
use crate::state::{AppState, TenantConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();

    // --------------------------------------------------
    // Read environment: Master DB Variables Preset
    // --------------------------------------------------
    let master_db_name: String = env::var("MASTER_DB_NAME").expect("Master database name missing");
    let master_db_pass: String = env::var("MASTER_DB_PASS").expect("Master database pass missing");
    let master_db_user: String = env::var("MASTER_DB_USER").expect("Master database user missing");
    let master_db_url: String = env::var("DATABASE_URL").expect("Master url missing");
    let super_psql_url: String =
        env::var("POSTGRES_SUPER_URL").expect("POSTGRES_SUPER_URL missing");

    // Ensure master DB exists -> Create if running first time
    let _ = init_master(
        &super_psql_url,
        &master_db_name,
        &master_db_user,
        &master_db_pass,
    )
    .await
    .expect("Master DB initialization failed");

    // Create master pool
    let master_pool: sqlx::Pool<sqlx::Postgres> = PgPoolOptions::new()
        .max_connections(10)
        .connect(&master_db_url)
        .await?;

    /********************************************************************************
        -----------------------------------------------------------------------------
        Tenant configuration structure, will be set after login/signup in user module
        -----------------------------------------------------------------------------
        let tenant_config: TenantConfig = TenantConfig {
            user: "",
            password: "",
            host: "",
            port: "",
            base_url: format!(
                "TENANT_DB_USER",    // USER   set within company module
                "TENANT_DB_PASSWORD" // PASSWD set within company module
                "TENANT_HOST"        // Default = localhost, for desktop
                "SERVER_PORT"        // Default = 5432 setup dynamically
                "postgres://{}:{}@{}:{}/", // Full tenant connection URL
            ),
        };
    *********************************************************************************/

    // JWT secret set inside .env, generated with encryption algorithm
    let jwt_secret: String = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    // --------------------------------------------------
    // Build application state
    // --------------------------------------------------
    let app_state: Arc<AppState> = Arc::new(AppState {
        master_pool,
        jwt_secret,
        tenant_pools: DashMap::new(),
        tenant_config: TenantConfig {
            user: "".to_string(),
            password: "".to_string(),
            host: "".to_string(),
            port: "".to_string(),
            base_url: "".to_string(),
        },
    });

    // --------------------------------------------------
    // Start HTTP server, entry point for global APIs
    // --------------------------------------------------
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let app: axum::Router = routes::create_router(app_state);
    let addr: SocketAddr = "127.0.0.1:8080".parse()?;

    println!("Listening on {}", addr);
    tracing::info!("Server running on {}", addr);

    let listener: TcpListener = TcpListener::bind(addr).await?;
    serve(listener, app.into_make_service()).await?;

    Ok(())
}
