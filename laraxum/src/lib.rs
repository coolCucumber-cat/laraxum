#![allow(async_fn_in_trait)]

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

pub type Id = u64;

pub type Sql = ();

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

pub trait Table {
    type Db: Db<Self::Response>;
    type Response: Serialize;
    type Request: for<'a> Deserialize<'a>;
}

pub trait Model: Table {
    type RequestQuery: for<'a> Deserialize<'a>;

    async fn get_all(db: &Self::Db) -> Result<Vec<Self::Response>, sqlx::Error>;
    async fn get_one(db: &Self::Db, id: Id) -> Result<Option<Self::Response>, sqlx::Error>;
    async fn get_one_exact(db: &Self::Db, id: Id) -> Result<Self::Response, sqlx::Error>;
    async fn create_one(db: &Self::Db, r: Self::Request) -> Result<Id, sqlx::Error>;
    async fn create_one_return(
        db: &Self::Db,
        r: Self::Request,
    ) -> Result<Self::Response, sqlx::Error> {
        match Self::create_one(db, r).await {
            Ok(id) => Self::get_one_exact(db, id).await,
            Err(err) => Err(err),
        }
    }
    async fn update_one(db: &Self::Db, r: Self::Request, id: Id) -> Result<(), sqlx::Error>;
    async fn update_one_return(
        db: &Self::Db,
        r: Self::Request,
        id: Id,
    ) -> Result<Self::Response, sqlx::Error> {
        match Self::update_one(db, r, id).await {
            Ok(()) => Self::get_one_exact(db, id).await,
            Err(err) => Err(err),
        }
    }
    async fn delete_one(db: &Self::Db, id: Id) -> Result<(), sqlx::Error>;
}

pub trait Controller: Model {
    type State: AnyDb<Db = Self::Db>;

    #[allow(unused_variables)]
    async fn index(
        State(state): State<Arc<Self::State>>,
        Query(query): Query<Self::RequestQuery>,
    ) -> Result<Json<Vec<Self::Response>>, impl IntoResponse> {
        let db = state.db();
        match Self::get_all(db).await {
            Ok(r) => Ok(Json(r)),
            Err(e) => Err(e.to_string()),
        }
    }
    async fn get(
        State(state): State<Arc<Self::State>>,
        Path(id): Path<Id>,
    ) -> Result<Json<Self::Response>, impl IntoResponse> {
        let db = state.db();
        let map_err = Self::get_one(db, id)
            .await
            .map_err(|e| e.to_string().into_response())?;
        match map_err {
            Some(v) => Ok(Json(v)),
            None => Err(StatusCode::NOT_FOUND.into_response()),
        }
    }
    async fn create(
        State(state): State<Arc<Self::State>>,
        Json(r): Json<Self::Request>,
    ) -> Result<Json<Self::Response>, impl IntoResponse> {
        let db = state.db();
        match Self::create_one_return(db, r).await {
            Ok(r) => Ok(Json(r)),
            Err(e) => Err(e.to_string()),
        }
    }
    async fn update(
        State(state): State<Arc<Self::State>>,
        Path(id): Path<Id>,
        Json(r): Json<Self::Request>,
    ) -> Result<Json<Self::Response>, impl IntoResponse> {
        let db = state.db();
        match Self::update_one_return(db, r, id).await {
            Ok(r) => Ok(Json(r)),
            Err(e) => Err(e.to_string()),
        }
    }
    async fn delete(
        State(state): State<Arc<Self::State>>,
        Path(id): Path<Id>,
    ) -> Result<(), impl IntoResponse> {
        let db = state.db();
        match Self::delete_one(db, id).await {
            Ok(()) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
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
