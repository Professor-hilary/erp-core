use axum::{
    Extension, Router,
    extract::Request,
    middleware,
    response::{IntoResponse, Response},
    routing::get,
};

use sqlx::PgPool;
use std::{sync::Arc, time::Duration};
use tower_http::{classify::ServerErrorsFailureClass, cors::CorsLayer, trace::TraceLayer};

use crate::middleware::{Authenticated, layer::auth_middleware};

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
    // Protected Routes
    let vendorsx: Router<Arc<AppState>> =
        crate::features::vendors::handlers::router().layer(middleware::from_fn(auth_middleware));
    let customersx: Router<Arc<AppState>> =
        crate::features::customers::handlers::router().layer(middleware::from_fn(auth_middleware));
    let employeesx: Router<Arc<AppState>> =
        crate::features::hr::handler::router().layer(middleware::from_fn(auth_middleware));
    let transactionsx: Router<Arc<AppState>> = crate::features::transactions::handlers::router()
        .layer(middleware::from_fn(auth_middleware));
    let accountsx: Router<Arc<AppState>> =
        crate::features::accounts::handlers::router().layer(middleware::from_fn(auth_middleware));
    let inventoryx: Router<Arc<AppState>> =
        crate::features::inventory::handler::router().layer(middleware::from_fn(auth_middleware));

    Router::new()
        // CHECK THAT SERVER IS UP AND RUNING
        .route(
            "/",
            get(|| async { "Yey! Service Up And Runing Successfully!\n" }),
        )
        // PUBLIC ROUTES (no auth required)
        .nest("/api/auth", crate::features::auth::handlers::router())
        // PROTECTED ROUTES (require auth)
        .nest("/api/transactions", transactionsx)
        .nest("/api/accounts", accountsx)
        .nest("/api/customers", customersx)
        .nest("/api/inventory", inventoryx)
        .nest("/api/vendors", vendorsx)
        .nest("/api/employees", employeesx)
        // CORS & global state
        .layer(Extension(state.clone()))
        .layer(CorsLayer::permissive())
        // Get log info for audit trail
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request| {
                    let user_id = request
                        .extensions()
                        .get::<Authenticated>()
                        .map(|auth: &Authenticated| auth.0.to_string())
                        .unwrap_or_else(|| "anonymous".to_string());

                    tracing::info_span!(
                        "request",
                        method = %request.method(),
                        uri = %request.uri(),
                        user_id = %user_id,
                    )
                })
                .on_response(
                    |response: &Response, latency: Duration, _span: &tracing::Span| {
                        tracing::info!(
                            status = response.status().as_u16(),
                            latency = format!("{:.2?}", latency),
                            "response sent"
                        );
                    },
                )
                .on_failure(
                    |error: ServerErrorsFailureClass, latency: Duration, _span: &tracing::Span| {
                        tracing::error!(
                            status = 500,
                            latency = format!("{:.2?}", latency),
                            error = %error,
                            "request failed"
                        );
                    },
                ),
        )
        .with_state(state)
        .fallback(handler_404) // ← ADD THIS
}

// 404 Fallback Handler
async fn handler_404() -> impl IntoResponse {
    let error: crate::errors::AppError = crate::errors::AppError::NotFound(
        "Oops! The page you're looking for doesn't exist. CHECK THE URI".into(),
    );
    error.into_response()
}
