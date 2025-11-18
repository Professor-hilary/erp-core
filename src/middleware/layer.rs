// src/middleware/layer.rs
use axum::{
    body::Body,
    extract::Extension,
    extract::FromRequestParts,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::{middleware::auth::Authenticated, routes::AppState};

/// JWT Auth Middleware – works with `from_fn`
pub async fn auth_middleware(
    Extension(state): Extension<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let (mut parts, body) = req.into_parts();

    let auth: Authenticated = Authenticated::from_request_parts(&mut parts, &state)
        .await
        .map_err(|(s, m)| (s, m.to_string()))?;

    let mut req: Request<Body> = Request::from_parts(parts, body);
    req.extensions_mut().insert(auth);

    Ok(next.run(req).await)
}
