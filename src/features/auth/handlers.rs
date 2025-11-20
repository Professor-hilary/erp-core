// src/features/auth/handlers.rs
use axum::{
    Router, extract::{Json, State}, routing::post,
    response::IntoResponse,
};
use std::sync::Arc;
use crate::{
    errors::AppError,
    features::auth::{AuthService, repository::PostgresUserRepo},
    infrastructure::responses::ApiResponse,
    models::{dto::JwtClaims, user::{CreateUser, LoginUser}},
    state::AppState,
};
use uuid::Uuid;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/switch-company", post(switch_company))  // ← NEW
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let repo = PostgresUserRepo::new(state.master_pool.clone());
    let service = AuthService::new(repo, state.clone());
    let (user, token) = service.register(&payload).await?;
    Ok(ApiResponse::created_with_token(user, token, "Registered successfully"))
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginUser>,
) -> Result<impl IntoResponse, AppError> {
    let repo = PostgresUserRepo::new(state.master_pool.clone());
    let service = AuthService::new(repo, state.clone());
    let token = service.login(&payload).await?;

    // If user has no company → guide them
    let claims: JwtClaims = jsonwebtoken::decode(
        &token,
        &jsonwebtoken::DecodingKey::from_secret(state.jwt_secret.as_ref()),
        &jsonwebtoken::Validation::default(),
    ).unwrap().claims;

    if claims.company_id.is_none() {
        return Ok(ApiResponse::success_with_meta(
            token,
            "Login successful. Create or join a company to continue.",
            serde_json::json!({ "requires_company": true })
        ));
    }

    Ok(ApiResponse::success(token, "Login successful"))
}

// POST /api/auth/switch-company { "company_id": "..." }
async fn switch_company(
    State(state): State<Arc<AppState>>,
    crate::middleware::Authenticated { user_id, .. }: crate::middleware::Authenticated,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let company_id: Uuid = payload["company_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::Validation("Invalid company_id".into()))?;

    let repo = PostgresUserRepo::new(state.master_pool.clone());
    let service = AuthService::new(repo, state.clone());
    let new_token = service.switch_company(user_id, company_id).await?;

    Ok(ApiResponse::success(new_token, "Company switched successfully"))
}

// // src/features/auth/handlers.rs
// use axum::{
//     Router, extract::{Json, State}, response::Response, routing::post
// };
// use std::sync::Arc;
// use crate::{
//     errors::AppError,
//     features::auth::{AuthService, repository::PostgresUserRepo},
//     infrastructure::responses::ApiResponse,
//     models::user::{CreateUser, LoginUser},
//     state::AppState,
// };

// pub fn router() -> Router<Arc<AppState>> {
//     Router::new()
//         .route("/register", post(register))
//         .route("/login", post(login))
// }

// async fn register(
//     State(state): State<Arc<AppState>>,
//     Json(payload): Json<CreateUser>,
// ) -> Result<Response, AppError> {
//     let repo = PostgresUserRepo::new(state.master_pool.clone());
//     let service = AuthService::new(repo, state.clone());
//     let user = service.register(&payload).await?;
//     Ok(ApiResponse::created(user, "User registered successfully"))
// }

// async fn login(
//     State(state): State<Arc<AppState>>,
//     Json(payload): Json<LoginUser>,
// ) -> Result<Response, AppError> {
//     let repo = PostgresUserRepo::new(state.master_pool.clone());
//     let service = AuthService::new(repo, state.clone());
//     let token = service.login(&payload).await?;
//     Ok(ApiResponse::success(token, "Login successful"))
// }
