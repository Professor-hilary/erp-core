pub mod client;
pub mod context;
pub mod fixtures;
pub mod state;

use std::error::Error;

pub use client::*;
pub use context::*;

pub type E2eResult<T> = Result<T, Box<dyn Error + Send + Sync>>;