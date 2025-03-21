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

pub enum Error {
    /// http `401 Unauthorized`
    ///
    /// Although the status code is called Unauthorized, it means the identity of the user is unknown and therefore unauthenticated
    Unauthenticated,
    /// http `403 Forbidden`
    ///
    /// Although this has the name of the name of the `401` status code, it means the identity of the user is known and unauthorized
    Unauthorized,
    /// http `404 Not Found`
    ///
    /// The resource could not be found
    NotFound,
    /// http `500 Internal Server Error`
    ///
    /// There was an issue with the server
    Server,
}

impl Error {
    const fn status_code(self) -> StatusCode {
        match self {
            Self::Unauthenticated => StatusCode::UNAUTHORIZED,
            Self::Unauthorized => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Server => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => Self::NotFound,
            #[cfg(debug_assertions)]
            error => {
                panic!("sql error: {error:?}");
            }
            #[cfg(not(debug_assertions))]
            _ => Self::Server,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        self.status_code().into_response()
    }
}

pub enum ModelError<BadRequest> {
    /// http `400 Bad Request`
    ///
    /// Data in a request is invalid
    BadRequest(BadRequest),
    Other(Error),
}

impl<E> From<Error> for ModelError<E> {
    fn from(other: Error) -> Self {
        Self::Other(other)
    }
}

impl<E> IntoResponse for ModelError<E>
where
    E: Serialize,
{
    fn into_response(self) -> Response {
        match self {
            Self::BadRequest(err) => Json(err).into_response(),
            Self::Other(err) => err.into_response(),
        }
    }
}

pub trait Db<Model> {}

pub trait AnyDb: Sized {
    type Db;
    async fn connect_with_str(s: &str) -> Result<Self, sqlx::Error>;
    async fn connect() -> Result<Self, sqlx::Error> {
        let url = std::env::var("DATABASE_URL");
        let url = url.expect("a valid DATABASE_URL");
        // let url = url.map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;
        Self::connect_with_str(&url).await
    }
    fn db(&self) -> &Self::Db;
}

pub trait Table {
    type Db: Db<Self::Response>;
    type Response: Serialize;
    type Request: for<'a> Deserialize<'a>;
    type RequestError: Serialize;
    type RequestQuery: for<'a> Deserialize<'a>;
}

pub trait Model: Table {
    async fn get_all(db: &Self::Db) -> Result<Vec<Self::Response>, Error>;
    async fn get_one(db: &Self::Db, id: Id) -> Result<Self::Response, Error>;
    async fn create_one(db: &Self::Db, r: Self::Request) -> Result<Id, Error>;
    async fn create_one_return(db: &Self::Db, r: Self::Request) -> Result<Self::Response, Error> {
        match Self::create_one(db, r).await {
            Ok(id) => Self::get_one(db, id).await,
            Err(err) => Err(err),
        }
    }
    async fn update_one(db: &Self::Db, r: Self::Request, id: Id) -> Result<(), Error>;
    async fn update_one_return(
        db: &Self::Db,
        r: Self::Request,
        id: Id,
    ) -> Result<Self::Response, Error> {
        match Self::update_one(db, r, id).await {
            Ok(()) => Self::get_one(db, id).await,
            Err(err) => Err(err),
        }
    }
    async fn delete_one(db: &Self::Db, id: Id) -> Result<(), Error>;
}

pub trait Controller: Model {
    type State: AnyDb<Db = Self::Db>;

    async fn index(
        State(state): State<Arc<Self::State>>,
        Query(_query): Query<Self::RequestQuery>,
    ) -> Result<Json<Vec<Self::Response>>, Error> {
        let rs = Self::get_all(state.db()).await?;
        Ok(Json(rs))
    }
    async fn get(
        State(state): State<Arc<Self::State>>,
        Path(id): Path<Id>,
    ) -> Result<Json<Self::Response>, Error> {
        let rs = Self::get_one(state.db(), id).await?;
        Ok(Json(rs))
    }
    async fn create(
        State(state): State<Arc<Self::State>>,
        Json(rq): Json<Self::Request>,
    ) -> Result<Json<Self::Response>, ModelError<Self::RequestError>> {
        let rs = Self::create_one_return(state.db(), rq).await?;
        Ok(Json(rs))
    }
    async fn update(
        State(state): State<Arc<Self::State>>,
        Path(id): Path<Id>,
        Json(rq): Json<Self::Request>,
    ) -> Result<Json<Self::Response>, ModelError<Self::RequestError>> {
        let rs = Self::update_one_return(state.db(), rq, id).await?;
        Ok(Json(rs))
    }
    async fn delete(
        State(state): State<Arc<Self::State>>,
        Path(id): Path<Id>,
    ) -> Result<(), Error> {
        Self::delete_one(state.db(), id).await?;
        Ok(())
    }
}

// pub trait Route<S = (), E = core::convert::Infallible> {
//     fn method_router() -> MethodRouter<S, E>;
// }
// pub trait RouteId<S = (), E = core::convert::Infallible> {
//     fn method_router_id() -> MethodRouter<S, E>;
// }

// impl<C> Route<C::State> for C
// where
//     C: Controller,
// {
//     fn method_router() -> MethodRouter<C::State> {
//         MethodRouter::new().get(C::index).post(C::create)
//     }
// }
// impl<C> RouteId<C::State> for C
// where
//     C: Controller,
// {
//     fn method_router_id() -> MethodRouter<C::State> {
//         MethodRouter::new()
//             .get(C::get)
//             .patch(C::update)
//             .delete(C::delete)
//     }
// }
// impl<C, E> Route<C::State, E> for C
// where
//     C: Controller,
// {
//     fn method_router() -> MethodRouter<C::State, E> {
//         MethodRouter::new().get(C::index).post(C::create)
//     }
// }
// impl<C, E> RouteId<C::State, E> for C
// where
//     C: Controller,
// {
//     fn method_router_id() -> MethodRouter<C::State, E> {
//         MethodRouter::new()
//             .get(C::get)
//             .patch(C::update)
//             .delete(C::delete)
//     }
// }
