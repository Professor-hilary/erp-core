// src/infrastructure/appjson_errors.rs

use std::error::Error as StdError;

use axum::{
    Json as AxumJson,
    extract::{FromRequest, Request, rejection::JsonRejection},
};
use serde::de::DeserializeOwned;

use crate::interface::api::errors::AppError;

#[derive(Clone)]
pub struct AppJson<T>(pub T);

/// Helper to dig into the wrapped serd_path_to_error::Error<serd_json::Error>
fn serde_path_error_message<E>(err: &E) -> String
where
    E: StdError + 'static,
{
    // Axum wraps the serde error in serd_path_to_error::Error
    if let Some(path_err) = find_source::<serde_path_to_error::Error<serde_json::Error>>(err) {
        let path: String = path_err.path().to_string(); // This gives e.g. "users[0].email"

        let location: String = if path == "." || path.is_empty() {
            "at the root".to_string()
        } else {
            format!("at {}", path)
        };

        let inner: String = path_err.inner().to_string();

        let pos: String = if let Some(_) = path_err.inner().source() {
            if let Some(sje) = find_source::<serde_json::Error>(path_err.inner()) {
                format!(" (line {}, column {}", sje.line(), sje.column())
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        format!("Invalid JSON data {}: {}{}", location, inner, pos)
    } else {
        // Fallback if not wrapped that way
        err.to_string()
    }
}

// Find source error of specific type
fn find_source<E>(err: &(impl StdError + 'static)) -> Option<&E>
where
    E: StdError + 'static,
{
    let mut current = err.source();

    while let Some(src) = current {
        if let Some(downcast) = src.downcast_ref::<E>() {
            return Some(downcast);
        }
        current = src.source();
    }

    None
}

impl<S, T> FromRequest<S> for AppJson<T>
where
    AxumJson<T>: FromRequest<S, Rejection = JsonRejection>,
    T: DeserializeOwned + 'static,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match AxumJson::<T>::from_request(req, state).await {
            Ok(AxumJson(value)) => Ok(AppJson(value)),

            Err(err) => {
                let msg = match err {
                    JsonRejection::MissingJsonContentType(_) => {
                        "Missing `Content-Type: applicaction/json` header".to_string()
                    }
                    JsonRejection::JsonSyntaxError(e) => {
                        format!("Malformed JSON syntax: {}", e)
                    }
                    JsonRejection::JsonDataError(e) => {
                        // For wrong keys / types / missing fields
                        serde_path_error_message(&e)
                    }
                    JsonRejection::BytesRejection(_) => {
                        "Oops! Failed to read request body".to_string()
                    }
                    _ => "Invalid JSON payload".to_string(),
                };
                Err(AppError::Unprocessable(msg))
            }
        }
    }
}
