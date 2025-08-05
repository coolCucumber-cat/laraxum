pub mod backend;
pub mod error;
pub mod frontend;
pub mod macros;
pub mod request;

pub use backend::{AnyDb, Collection, Db, Id, ManyModel, Model, Table};
pub use error::{AuthError, Error, ModelError};
pub use frontend::{Auth, AuthHeader, Controller, Json};
pub use request::Request;

#[cfg(feature = "macros")]
pub use laraxum_macros::{db, router};
