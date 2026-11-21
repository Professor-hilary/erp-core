//! Tenant RBAC: roles, modules, Actor resolution with in-memory cache.
pub mod actor;
pub mod service;

pub use actor::{Actor, Module};
pub use service::RbacService;
