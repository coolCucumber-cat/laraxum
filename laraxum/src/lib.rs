#[doc = include_str!("../../README.md")]
// TODO
// - docs: validate min_len
// - docs: aggregate example
pub mod controller;
pub mod error;
pub mod macros;
pub mod model;
pub mod request;

pub use controller::{
    Controller,
    auth::{AuthToken, Authenticate, AuthenticateToken, Authorize},
    extract::Json,
};
pub use error::{AppError, AuthError, Error, ModelError};
pub use model::{AggregateMany, AggregateOne, Collection, Connect, Db, ManyModel, Model, Table};
pub use request::Request;

#[cfg(feature = "macros")]
#[doc(inline)]
pub use laraxum_macros::{db, router};
