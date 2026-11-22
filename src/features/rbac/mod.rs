//! Tenant RBAC: roles, modules, Actor resolution with Redis cache.
pub mod actor;
pub mod handlers;
pub mod service;

pub use actor::{Actor, Module};
pub use service::RbacService;
#[allow(unused_imports)]
pub use service::UpsertMembershipRequest;
