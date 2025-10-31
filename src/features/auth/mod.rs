// src/features/auth/mod.rs
pub mod handlers;
pub mod services;
pub mod repository;

// Re-export for convenience
pub use services::AuthService;
