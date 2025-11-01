// src/middleware/mod.rs
pub mod auth;
pub mod layer;

#[allow(dead_code)]
pub use auth::{Authenticated};