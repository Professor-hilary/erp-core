use axum::{Router, routing::get};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
}

#[allow(dead_code)]
impl AppState {
    pub fn new(pool: PgPool, jwt_secret: String) -> Self {
        Self { pool, jwt_secret }
    }
}

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/",
            get(|| async { "Chiefalry Accountant Running Successfully\n" }),
        )
        .nest("/api/auth", crate::features::auth::handlers::router()) // no state here
        .nest(
            "/api/transactions",
            crate::features::transactions::handlers::router(),
        )
        .nest(
            "/api/accounts",
            crate::features::accounts::handlers::router(),
        )
        .layer(CorsLayer::permissive())
        .with_state(state) // single source of truth
}
