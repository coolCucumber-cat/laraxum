#![allow(async_fn_in_trait)]

pub mod backend;
pub mod error;
pub mod frontend;
pub mod macros;
pub mod request;

pub use backend::{AnyDb, Collection, Db, Id, ManyModel, Model, Table};
pub use error::{Error, ModelError};
pub use frontend::{Controller, Json};
pub use request::Request;
