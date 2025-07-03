#![allow(async_fn_in_trait)]

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

pub type Id = u64;

#[derive(Debug)]
pub enum Error {
    /// [400 Bad Request](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.1)
    BadRequest,
    /// [401 Unauthorized](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.2)
    ///
    /// Although the status code is called Unauthorized, it means the identity of the user is unknown and therefore unauthenticated
    Unauthenticated,
    /// [403 Forbidden](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.4)
    ///
    /// Although this has the name of the name of the `401` status code, it means the identity of the user is known and unauthorized
    Unauthorized,
    /// [404 Not Found](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.5)
    NotFound,
    /// [409 Conflict](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.10)
    Conflict,
    /// [429 Too Many Requests](https://datatracker.ietf.org/doc/html/rfc6585#section-4)
    TooManyRequests,
    /// [500 Internal Server Error](https://datatracker.ietf.org/doc/html/rfc9110#section-15.6.1)
    Internal,
}

impl Error {
    const fn status_code(self) -> StatusCode {
        match self {
            Self::BadRequest => StatusCode::BAD_REQUEST,
            Self::Unauthenticated => StatusCode::UNAUTHORIZED,
            Self::Unauthorized => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict => StatusCode::CONFLICT,
            Self::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
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
                #[cfg(debug_assertions)]
                panic!("sql error: {error:?}");
                #[cfg(not(debug_assertions))]
                {
                    eprintln!("sql error: {error:?}");
                    Self::Internal
                }
            }
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        self.status_code().into_response()
    }
}

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

impl<UnprocessableEntity> IntoResponse for ModelError<UnprocessableEntity>
where
    UnprocessableEntity: Serialize,
{
    fn into_response(self) -> Response {
        match self {
            Self::UnprocessableEntity(err) => Json(err).into_response(),
            Self::Other(err) => err.status_code().into_response(),
        }
    }
}

pub trait Db<Model> {}

pub trait AnyDb: Sized {
    type Db;
    async fn connect_with_str(s: &str) -> Result<Self, sqlx::Error>;
    async fn connect() -> Result<Self, sqlx::Error> {
        let url = std::env::var("DATABASE_URL");
        let url = url.map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;
        Self::connect_with_str(&url).await
    }
    fn db(&self) -> &Self::Db;
}

pub trait Table: Sized {
    type Db: Db<Self>;
    type Response;
}

pub trait Collection: Table {
    type GetAllRequestQuery;
    type CreateRequest;
    type CreateRequestError;

    async fn get_all(db: &Self::Db) -> Result<Vec<Self::Response>, Error>;
    async fn create_one(
        db: &Self::Db,
        r: Self::CreateRequest,
    ) -> Result<(), ModelError<Self::CreateRequestError>>;
}

pub trait Model: Collection {
    type Id: Copy;
    type UpdateRequest;
    type UpdateRequestError;

    async fn get_one(db: &Self::Db, id: Self::Id) -> Result<Self::Response, Error>;
    async fn create_get_one(
        db: &Self::Db,
        rq: Self::CreateRequest,
    ) -> Result<Self::Response, ModelError<Self::CreateRequestError>>;
    async fn update_one(
        db: &Self::Db,
        rq: Self::UpdateRequest,
        id: Self::Id,
    ) -> Result<(), ModelError<Self::UpdateRequestError>>;
    async fn update_get_one(
        db: &Self::Db,
        rq: Self::UpdateRequest,
        id: Self::Id,
    ) -> Result<Self::Response, ModelError<Self::UpdateRequestError>> {
        Self::update_one(db, rq, id).await?;
        let rs = Self::get_one(db, id).await?;
        Ok(rs)
    }
    async fn delete_one(db: &Self::Db, id: Self::Id) -> Result<(), Error>;
}

pub trait Controller: Model
where
    <Self as Table>::Response: Serialize,
    <Self as Collection>::GetAllRequestQuery: for<'a> Deserialize<'a>,
    <Self as Collection>::CreateRequest: for<'a> Deserialize<'a>,
    <Self as Collection>::CreateRequestError: Serialize,
    <Self as Model>::Id: Serialize + for<'a> Deserialize<'a>,
    <Self as Model>::UpdateRequest: for<'a> Deserialize<'a>,
    <Self as Model>::UpdateRequestError: Serialize,
{
    type State: AnyDb<Db = Self::Db>;

    async fn index(
        State(state): State<Arc<Self::State>>,
        Query(_query): Query<Self::GetAllRequestQuery>,
    ) -> Result<Json<Vec<Self::Response>>, Error> {
        let rs = Self::get_all(state.db()).await?;
        Ok(Json(rs))
    }
    async fn get(
        State(state): State<Arc<Self::State>>,
        Path(id): Path<Self::Id>,
    ) -> Result<Json<Self::Response>, Error> {
        let rs = Self::get_one(state.db(), id).await?;
        Ok(Json(rs))
    }
    async fn create(
        State(state): State<Arc<Self::State>>,
        Json(rq): Json<Self::CreateRequest>,
    ) -> Result<Json<Self::Response>, ModelError<Self::CreateRequestError>> {
        let rs = Self::create_get_one(state.db(), rq).await?;
        Ok(Json(rs))
    }
    async fn update(
        State(state): State<Arc<Self::State>>,
        Path(id): Path<Self::Id>,
        Json(rq): Json<Self::UpdateRequest>,
    ) -> Result<Json<Self::Response>, ModelError<Self::UpdateRequestError>> {
        let rs = Self::update_get_one(state.db(), rq, id).await?;
        Ok(Json(rs))
    }
    async fn delete(
        State(state): State<Arc<Self::State>>,
        Path(id): Path<Self::Id>,
    ) -> Result<(), Error> {
        Self::delete_one(state.db(), id).await?;
        Ok(())
    }
}

pub trait ManyModel<OneResponse>: Table {
    type OneRequest;
    type ManyRequest;
    type ManyResponse;

    async fn get_many(
        db: &Self::Db,
        one: Self::OneRequest,
    ) -> Result<Vec<Self::ManyResponse>, Error>;
    async fn create_many(
        db: &Self::Db,
        one: Self::OneRequest,
        many: &[Self::ManyRequest],
    ) -> Result<(), Error>;
    async fn update_many(
        db: &Self::Db,
        one: Self::OneRequest,
        many: &[Self::ManyRequest],
    ) -> Result<(), Error>;
    async fn delete_many(db: &Self::Db, one: Self::OneRequest) -> Result<(), Error>;
}

// pub trait AdvancedModelMany<Id>: Table {
//     type AdvancedModelManyResponse: Serialize;
//
//     async fn get_many(db: &Self::Db, id: Id)
//     -> Result<Vec<Self::AdvancedModelManyResponse>, Error>;
// }

pub trait Decode {
    type Decode;
    fn decode(decode: Self::Decode) -> Self;
}
pub trait Encode {
    type Encode;
    fn encode(self) -> Self::Encode;
}

macro_rules! impl_encode_decode {
    { $($ty:ty),* $(,)* } => {
        $(
            impl Decode for $ty {
                type Decode = Self;
                #[inline]
                fn decode(decode: Self::Decode) -> Self {
                    decode
                }
            }
            impl Encode for $ty {
                type Encode = Self;
                #[inline]
                fn encode(self) -> Self::Encode {
                    self
                }
            }
        )*
    };
}

impl_encode_decode! {
    String,
    u8,
    i8,
    u16,
    i16,
    u32,
    i32,
    u64,
    i64,
    f32,
    f64,
}
#[cfg(feature = "time")]
impl_encode_decode! {
    time::OffsetDateTime,
    time::PrimitiveDateTime,
    time::Date,
    time::Time,
    time::Duration,
}
#[cfg(feature = "chrono")]
impl_encode_decode! {
    chrono::DateTime::<chrono::Utc>,
    chrono::DateTime::<chrono::Local>,
    chrono::NaiveDateTime,
    chrono::NaiveDate,
    chrono::NaiveTime,
    chrono::TimeDelta,
}

impl Decode for bool {
    #[cfg(not(feature = "mysql"))]
    type Decode = Self;
    #[cfg(feature = "mysql")]
    type Decode = i8;

    #[inline]
    fn decode(decode: Self::Decode) -> Self {
        #[cfg(not(feature = "mysql"))]
        {
            decode
        }
        #[cfg(feature = "mysql")]
        {
            decode != 0
        }
    }
}
impl Encode for bool {
    #[cfg(not(feature = "mysql"))]
    type Encode = Self;
    #[cfg(feature = "mysql")]
    type Encode = i8;

    #[inline]
    fn encode(self) -> Self::Encode {
        #[cfg(not(feature = "mysql"))]
        {
            self
        }
        #[cfg(feature = "mysql")]
        {
            self as i8
        }
    }
}
