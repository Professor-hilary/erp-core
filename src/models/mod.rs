// src/models/mod.rs
pub mod user;
pub mod account;
pub mod transaction;
pub mod dto;

pub use transaction::Transaction;


// // models.rs
// use chrono::{DateTime, Utc, NaiveDate};
// use serde::{Deserialize, Serialize};
// use sqlx::FromRow;
// use uuid::Uuid;

// #[derive(Debug, Serialize)]
// pub struct JwtClaims {
//     pub sub: Uuid,
//     pub exp: usize,
// }

// #[derive(Debug, Serialize)]
// pub struct TrialBalance {
//     pub account_id: Uuid,
//     pub name: String,
//     pub balance: f64,
// }
