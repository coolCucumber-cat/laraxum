#![doc = include_str!("../../README.md")]

pub mod controller;
mod env;
pub mod error;
pub mod model;

pub use controller::{
    Controller,
    auth::{AuthToken, Authenticate, AuthenticateToken, Authorize},
    extract::Json,
};
pub use error::{AppError, AuthError, Error, ModelError};
pub use model::{AggregateMany, AggregateOne, Collection, Connect, Db, ManyModel, Model, Table};

#[cfg(feature = "macros")]
#[doc(inline)]
pub use laraxum_macros::{db, router};
