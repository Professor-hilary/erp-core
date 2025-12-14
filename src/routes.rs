// src/routes.rs
use axum::{
    Extension, Router,
    extract::Request,
    middleware,
    response::{IntoResponse, Response},
    routing::get,
};

use crate::{
    AppState,
    features::{
        accounts, company, customers, inventory, payroll, reports, transactions, vendors, workforce,
    },
    infrastructure::errors::AppError,
    middleware::{auth::AuthenticatedUser, layer::auth_middleware},
};

use std::{sync::Arc, time::Duration};
use tower_http::{classify::ServerErrorsFailureClass, cors::CorsLayer, trace::TraceLayer};


pub fn create_router(state: Arc<AppState>) -> Router {
    // Protected Routes - need a verified tenant (Company AppState)
    let company_routes: Router<Arc<AppState>> = company::handlers::router().layer(
        middleware::from_fn_with_state(state.clone(), auth_middleware),
    );
    let vendor_routes: Router<Arc<AppState>> = vendors::handlers::router().layer(
        middleware::from_fn_with_state(state.clone(), auth_middleware),
    );
    let customer_routes: Router<Arc<AppState>> = customers::handlers::router().layer(
        middleware::from_fn_with_state(state.clone(), auth_middleware),
    );
    let employee_routes: Router<Arc<AppState>> = workforce::handlers::router().layer(
        middleware::from_fn_with_state(state.clone(), auth_middleware),
    );
    let transaction_routes: Router<Arc<AppState>> = transactions::handlers::router().layer(
        middleware::from_fn_with_state(state.clone(), auth_middleware),
    );
    let account_routes: Router<Arc<AppState>> = accounts::handlers::router().layer(
        middleware::from_fn_with_state(state.clone(), auth_middleware),
    );
    let inventory_routes: Router<Arc<AppState>> = inventory::handlers::router().layer(
        middleware::from_fn_with_state(state.clone(), auth_middleware),
    );
    let payroll_routes: Router<Arc<AppState>> = payroll::handlers::router().layer(
        middleware::from_fn_with_state(state.clone(), auth_middleware),
    );
    let report_routes: Router<Arc<AppState>> = reports::handlers::router().layer(
        middleware::from_fn_with_state(state.clone(), auth_middleware),
    );

    Router::new()
        // CHECK THAT SERVER IS UP AND RUNING
        .route(
            "/",
            get(|| async { "[Howdy]: Smart accountant is up and Running!\n" }),
        )
        // PUBLIC ROUTES (no auth required)
        .nest("/api/auth", crate::features::auth::handlers::router())
        // PROTECTED ROUTES (require auth)
        .nest("/api/companies", company_routes)
        .nest("/api/transactions", transaction_routes)
        .nest("/api/accounts", account_routes)
        .nest("/api/customers", customer_routes)
        .nest("/api/inventory", inventory_routes)
        .nest("/api/payroll", payroll_routes)
        .nest("/api/reports", report_routes)
        .nest("/api/vendors", vendor_routes)
        .nest("/api/workforce", employee_routes)
        // CORS & global state
        .layer(Extension(state.clone()))
        .layer(CorsLayer::permissive())
        // Get log info for audit trail
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request| {
                    let user_id: String = request
                        .extensions()
                        .get::<AuthenticatedUser>()
                        .map(|auth: &AuthenticatedUser| auth.user_id.to_string())
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
        .fallback(handler_404)
}

// 404 Fallback Handler
async fn handler_404() -> impl IntoResponse {
    let error: AppError =
        AppError::NotFound("Oops! The page you're looking for doesn't exist. CHECK THE URI".into());
    error.into_response()
}
