//! A controller manages the connection between model and view.  
//!
//! It handles incoming requests to the endpoints of the resource.

pub mod auth;
pub mod extract;
mod serve;

use auth::AuthToken;
use extract::Json;

use crate::{
    error::{Error, ModelError},
    model::{Collection, Model, Table},
};

use core::ops::Deref;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};

/// Get the URL environment variable. Defaults to `"localhost:80"`.
///
/// # Panics
/// - Not unicode.
pub fn url() -> std::borrow::Cow<'static, str> {
    crate::env::env_var_default!("URL", "localhost:80")
}

/// A controller manages the connection between model and view.  
///
/// Every function corresponds to a web endpoint for the resource.  
/// The view makes a request to a resource's web endpoint,
/// which the controller handles using the resource's model.
#[expect(unused_variables)]
pub trait Controller: Model
where
    <Self as Table>::Response: Serialize,
    <Self as Collection>::CreateRequest: for<'a> Deserialize<'a>,
    <Self as Collection>::CreateRequestError: Serialize,
    <Self as Model>::Id: for<'a> Deserialize<'a>,
    <Self as Model>::UpdateRequest: for<'a> Deserialize<'a>,
    <Self as Model>::UpdateRequestError: Serialize,
    AuthToken<Self::Auth>: axum::extract::FromRequestParts<Arc<Self::State>>,
{
    /// Stateful context which includes the database connection to interact with the resource's model.
    type State: Deref<Target = Self::Db>;
    /// Authenticate and authorize the user making the request.
    ///
    /// Must implement [Authorize] or be `()` to skip entirely.
    /// The [Authorize] trait performs authentication
    /// by delegating it to [Authorize::Authenticate][auth::Authorize::Authenticate],
    /// then performs authorization.
    /// Anything that implements [Authenticate] also automatically implements [Authorize]
    /// by delegating authentication to itself.
    ///
    /// [Authorize]: auth::Authorize
    /// [Authenticate]: auth::Authenticate
    type Auth;
    /// Request query for [get_many](Controller::get_many).
    type GetManyRequestQuery: for<'a> Deserialize<'a>;

    /// Get many records.
    ///
    /// method: `GET` \
    /// id: no \
    /// body: no \
    ///
    /// Default action is to get all records.  
    /// [GetManyRequestQuery][Controller::GetManyRequestQuery] can be used for custom requests.
    async fn get_many(
        state: State<Arc<Self::State>>,
        auth: AuthToken<Self::Auth>,
        query: Query<Self::GetManyRequestQuery>,
    ) -> Result<Json<Vec<Self::Response>>, Error> {
        let rs = Self::get_all(&*state.0).await?;
        Ok(Json(rs))
    }
    /// Get a record.
    ///
    /// method: `GET` \
    /// id: yes -> [Self::Id][Model::Id] \
    /// body: no \
    async fn get(
        state: State<Arc<Self::State>>,
        auth: AuthToken<Self::Auth>,
        id: Path<Self::Id>,
    ) -> Result<Json<Self::Response>, Error> {
        let rs = Self::get_one(&*state.0, id.0).await?;
        Ok(Json(rs))
    }
    /// Create a record.
    ///
    /// method: `POST` \
    /// id: no \
    /// body: yes -> [Self::CreateRequest][Collection::CreateRequest] \
    async fn create(
        state: State<Arc<Self::State>>,
        auth: AuthToken<Self::Auth>,
        rq: Json<Self::CreateRequest>,
    ) -> Result<Json<Self::Response>, ModelError<Self::CreateRequestError>> {
        let rs = Self::create_get_one(&*state.0, rq.0).await?;
        Ok(Json(rs))
    }
    /// Update a record.
    ///
    /// method: `PUT` \
    /// id: yes -> [Self::Id][Model::Id] \
    /// body: yes -> [Self::UpdateRequest][Model::UpdateRequest] \
    async fn update(
        state: State<Arc<Self::State>>,
        auth: AuthToken<Self::Auth>,
        id: Path<Self::Id>,
        rq: Json<Self::UpdateRequest>,
    ) -> Result<Json<Self::Response>, ModelError<Self::UpdateRequestError>> {
        let rs = Self::update_get_one(&*state.0, rq.0, id.0).await?;
        Ok(Json(rs))
    }
    /// Patch update a record.
    ///
    /// method: `PATCH` \
    /// id: yes -> [Self::Id][Model::Id] \
    /// body: yes, only fields that need to be updated -> [Self::PatchRequest][Model::PatchRequest] \
    async fn patch(
        state: State<Arc<Self::State>>,
        auth: AuthToken<Self::Auth>,
        id: Path<Self::Id>,
        rq: Json<Self::PatchRequest>,
    ) -> Result<Json<Self::Response>, ModelError<Self::PatchRequestError>> {
        let rs = Self::patch_get_one(&*state.0, rq.0, id.0).await?;
        Ok(Json(rs))
    }
    /// Delete a record.
    ///
    /// method: `DELETE` \
    /// id: yes -> [Self::Id][Model::Id] \
    /// body: no \
    async fn delete(
        state: State<Arc<Self::State>>,
        auth: AuthToken<Self::Auth>,
        id: Path<Self::Id>,
    ) -> Result<(), Error> {
        Self::delete_one(&*state.0, id.0).await?;
        Ok(())
    }
}
