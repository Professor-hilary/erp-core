// src/middleware/mod.rs
pub mod auth;

pub use auth::{auth_layer, Authenticated};