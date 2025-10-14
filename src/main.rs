// main.rs
mod errors;
mod models;
mod repositories;
mod services;
mod handlers;
mod responses;

use axum::serve;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use dotenv::dotenv;
use handlers::AppState;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Run migrations on startup (in prod, use separate process)
    // sqlx::migrate!("./migrations").run(&pool).await?;

    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let state = Arc::new(AppState { pool, jwt_secret });

    let app = handlers::create_router(state);

    let addr: SocketAddr = "127.0.0.1:8080".parse()?;
    println!("Listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    serve(listener, app.into_make_service()).await?;

    Ok(())
}
