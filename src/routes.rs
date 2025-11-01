use axum::{Extension, Router, middleware, routing::get};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::middleware::layer::auth_middleware;

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
    let protected_transactions: Router<Arc<AppState>> = crate::features::transactions::handlers::router()
        .layer(middleware::from_fn(auth_middleware));
    let protected_accounts: Router<Arc<AppState>> =
        crate::features::accounts::handlers::router().layer(middleware::from_fn(auth_middleware));

    Router::new()
        // CHECK THAT SERVER IS UP AND RUNING
        .route(
            "/",
            get(|| async { "Yey! Service Up And Runing Successfully!\n" }),
        )
        // PUBLIC ROUTES (no auth required)
        .nest("/api/auth", crate::features::auth::handlers::router())
        // PROTECTED ROUTES (require auth)
        .nest("/api/transactions", protected_transactions)
        .nest("/api/accounts", protected_accounts)
        // CORS & global state
        .layer(Extension(state.clone()))
        .layer(CorsLayer::permissive())
        .with_state(state)
}
