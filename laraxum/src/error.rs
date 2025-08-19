use crate::frontend::Json;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

/// An error in the controller.
#[derive(Debug)]
pub enum Error {
    // /// [400 Bad Request](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.1)
    // BadRequest,
    /// [404 Not Found](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.5)
    NotFound,
    /// [409 Conflict](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.10)
    Conflict,
    // /// [429 Too Many Requests](https://datatracker.ietf.org/doc/html/rfc6585#section-4)
    // TooManyRequests,
    /// [500 Internal Server Error](https://datatracker.ietf.org/doc/html/rfc9110#section-15.6.1)
    Internal,
}
impl Error {
    const fn status_code(self) -> StatusCode {
        match self {
            // Self::BadRequest => StatusCode::BAD_REQUEST,
            // Self::Unauthenticated => StatusCode::UNAUTHORIZED,
            // Self::Unauthorized => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict => StatusCode::CONFLICT,
            // Self::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        match error {
            // sqlx::Error::RowNotFound => Self::NotFound,
            sqlx::Error::Database(error) => {
                eprintln!("sql database error: {error:?}");
                Self::Conflict
            }
            error => {
                eprintln!("sql error: {error:?}");
                Self::Internal
            }
        }
    }
}
impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        self.status_code().into_response()
    }
}

/// An error during authentication.
#[derive(Debug)]
pub enum AuthError {
    /// [401 Unauthorized](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.2)
    ///
    /// Although the status code is called Unauthorized, it means the identity of the user is unknown and therefore unauthenticated
    Unauthenticated,
    /// [403 Forbidden](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.4)
    ///
    /// Although this has the name of the name of the `401` status code, it means the identity of the user is known and unauthorized
    Unauthorized,
}
impl AuthError {
    const fn status_code(self) -> StatusCode {
        match self {
            Self::Unauthenticated => StatusCode::UNAUTHORIZED,
            Self::Unauthorized => StatusCode::FORBIDDEN,
        }
    }
}
impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        self.status_code().into_response()
    }
}

/// An error in the controller with an unprocessable entity.
#[derive(Debug)]
pub enum ModelError<UnprocessableEntity> {
    /// [422 Unprocessable Entity](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.21)
    UnprocessableEntity(UnprocessableEntity),
    Other(Error),
}
impl<UnprocessableEntity, E> From<E> for ModelError<UnprocessableEntity>
where
    E: Into<Error>,
{
    fn from(other: E) -> Self {
        Self::Other(other.into())
    }
}
impl From<()> for ModelError<()> {
    fn from(value: ()) -> Self {
        Self::UnprocessableEntity(value)
    }
}
impl From<core::convert::Infallible> for ModelError<core::convert::Infallible> {
    fn from(value: core::convert::Infallible) -> Self {
        Self::UnprocessableEntity(value)
    }
}
impl<UnprocessableEntity> IntoResponse for ModelError<UnprocessableEntity>
where
    UnprocessableEntity: Serialize,
{
    fn into_response(self) -> Response {
        match self {
            Self::UnprocessableEntity(err) => {
                (StatusCode::UNPROCESSABLE_ENTITY, Json(err)).into_response()
            }
            Self::Other(err) => err.status_code().into_response(),
        }
    }
}

/// Error in an app that uses this crate.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AppError {
    #[error("io error: {0}")]
    Io(
        #[from]
        #[source]
        std::io::Error,
    ),
    #[error("sql error: {0}")]
    Sql(
        #[from]
        #[source]
        sqlx::Error,
    ),
}
