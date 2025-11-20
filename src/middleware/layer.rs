// src/middleware/layer.rs
use axum::{
    body::Body,
    extract::{Extension, FromRequestParts},
    http::{Request},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

use crate::{middleware::auth::Authenticated, state::AppState};

/// JWT Auth Middleware – works with `from_fn`
pub async fn auth_middleware(
    Extension(state): Extension<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let (mut parts, body) = req.into_parts();

    let auth: Authenticated = Authenticated::from_request_parts(&mut parts, &state)
        .await
        .map_err(|e| e.into_response())?;

    let mut req: Request<Body> = Request::from_parts(parts, body);
    req.extensions_mut().insert(auth);

    Ok(next.run(req).await)
}
