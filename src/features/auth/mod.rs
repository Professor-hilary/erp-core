// src/features/auth/mod.rs
pub mod handlers;
pub mod repository;
pub mod services;

// Re-export for convenience
pub use services::AuthService;
