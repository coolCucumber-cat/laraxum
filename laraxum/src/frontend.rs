use crate::{
    backend::{AnyDb, Collection, Model, Table},
    error::{Error, ModelError},
};

use std::sync::Arc;

use axum::{
    extract::{FromRequest, OptionalFromRequest, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

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
    type Headers;

    #[allow(unused_variables)]
    async fn index(
        State(state): State<Arc<Self::State>>,
        Query(query): Query<Self::GetAllRequestQuery>,
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

// pub trait Controller2: crate::backend::Model2
// where
//     <Self as Table>::Response: Serialize,
//     <Self as crate::backend::Collection2>::GetAllRequestQuery: for<'a> Deserialize<'a>,
//     <Self as crate::backend::Collection2>::CreateRequest: for<'a> Deserialize<'a>,
//     <Self as crate::backend::Collection2>::CreateRequestError: Serialize,
//     <Self as crate::backend::Model2>::Id: Serialize + for<'a> Deserialize<'a>,
//     <Self as crate::backend::Model2>::UpdateRequest: for<'a> Deserialize<'a>,
//     <Self as crate::backend::Model2>::UpdateRequestError: Serialize,
// {
//     type State: AnyDb<Db = Self::Db> + Sync + Send;
//     type Headers;
//
//     #[allow(unused_variables)]
//     fn index(
//         State(state): State<Arc<Self::State>>,
//         Query(query): Query<Self::GetAllRequestQuery>,
//     ) -> impl Future<Output = Result<Json<Vec<Self::Response>>, Error>> + Send {
//         async move {
//             let rs = Self::get_all(state.db()).await?;
//             Ok(Json(rs))
//         }
//     }
//     fn get(
//         State(state): State<Arc<Self::State>>,
//         Path(id): Path<Self::Id>,
//     ) -> impl Future<Output = Result<Json<Self::Response>, Error>> + Send {
//         async move {
//             let rs = Self::get_one(state.db(), id).await?;
//             Ok(Json(rs))
//         }
//     }
//     fn create(
//         State(state): State<Arc<Self::State>>,
//         Json(rq): Json<Self::CreateRequest>,
//     ) -> impl Future<Output = Result<Json<Self::Response>, ModelError<Self::CreateRequestError>>> + Send
//     {
//         async move {
//             let rs = Self::create_get_one(state.db(), rq).await?;
//             Ok(Json(rs))
//         }
//     }
//     fn update(
//         State(state): State<Arc<Self::State>>,
//         Path(id): Path<Self::Id>,
//         Json(rq): Json<Self::UpdateRequest>,
//     ) -> impl Future<Output = Result<Json<Self::Response>, ModelError<Self::UpdateRequestError>>> + Send
//     {
//         async move {
//             let rs = Self::update_get_one(state.db(), rq, id).await?;
//             Ok(Json(rs))
//         }
//     }
//     fn delete(
//         State(state): State<Arc<Self::State>>,
//         Path(id): Path<Self::Id>,
//     ) -> impl Future<Output = Result<(), Error>> + Send {
//         async move {
//             Self::delete_one(state.db(), id).await?;
//             Ok(())
//         }
//     }
// }

// pub trait DeserializeRequest: Sized {
//     type Item: for<'de> Deserialize<'de>;
//     type UnprocessableEntityError;
//
//     fn deserialize_request<'de, D>(
//         deserializer: D,
//     ) -> Result<Self, DeserializeRequestError<Self::UnprocessableEntityError, D::Error>>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         let item = Self::deserialize_item(deserializer).map_err(DeserializeRequestError::Serde)?;
//         Self::deserialize_request_from_item(item)
//             .map_err(DeserializeRequestError::UnprocessableEntity)
//     }
//     fn deserialize_item<'de, D>(deserializer: D) -> Result<Self::Item, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         Self::Item::deserialize(deserializer)
//     }
//     fn deserialize_request_from_item(
//         item: Self::Item,
//     ) -> Result<Self, Self::UnprocessableEntityError>;
// }

#[must_use]
#[derive(Debug, Clone, Copy, Default)]
pub struct Json<T>(pub T);
impl<T> Json<T>
where
    T: serde::de::DeserializeOwned,
{
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        let mut deserializer = serde_json::Deserializer::from_slice(bytes);
        T::deserialize(&mut deserializer).map(Json)
    }
}
impl<T, S> FromRequest<S> for Json<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = DeserializeRequestError<serde_json::Error>;
    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        match req.headers().get(axum::http::header::CONTENT_TYPE) {
            Some(content_type_header) if json_content_type(content_type_header) => {
                let bytes = bytes::Bytes::from_request(req, state)
                    .await
                    .map_err(DeserializeRequestError::Bytes)?;
                Self::from_bytes(&bytes).map_err(DeserializeRequestError::Serde)
            }
            _ => Err(DeserializeRequestError::ContentType),
        }
    }
}
impl<T, S> OptionalFromRequest<S> for Json<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = DeserializeRequestError<serde_json::Error>;
    async fn from_request(
        req: axum::extract::Request,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        match req.headers().get(axum::http::header::CONTENT_TYPE) {
            Some(content_type_header) if json_content_type(content_type_header) => {
                let bytes = bytes::Bytes::from_request(req, state)
                    .await
                    .map_err(DeserializeRequestError::Bytes)?;
                Self::from_bytes(&bytes)
                    .map(Some)
                    .map_err(DeserializeRequestError::Serde)
            }
            None => Ok(None),
            _ => Err(DeserializeRequestError::ContentType),
        }
    }
}
impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        axum::extract::Json(self.0).into_response()
    }
}

// impl<T, E> Json<T>
// where
//     T: DeserializeRequest<UnprocessableEntityError = E>,
//     // T: serde::de::DeserializeOwned,
// {
//     pub fn from_bytes(bytes: &[u8]) -> Result<Self, DeserializeRequestError<E, serde_json::Error>> {
//         let mut deserializer = serde_json::Deserializer::from_slice(bytes);
//         T::deserialize_request(&mut deserializer).map(Json)
//
//         // fn map_err(err: serde_json::Error) -> DeserializeRequestError<E> {
//         //     match err.classify() {
//         //         serde_json::error::Category::Data => DeserializeRequestError::Data(err),
//         //         serde_json::error::Category::Syntax | serde_json::error::Category::Eof => {
//         //             DeserializeRequestError::Syntax(err)
//         //         }
//         //         serde_json::error::Category::Io => {
//         //             #[cfg(debug_assertions)]
//         //             {
//         //                 // we don't use `serde_json::from_reader` and instead always buffer
//         //                 // bodies first, so we shouldn't encounter any IO errors
//         //                 unreachable!()
//         //             }
//         //             #[cfg(not(debug_assertions))]
//         //             {
//         //                 DeserializeRequestError::Syntax(err)
//         //             }
//         //         }
//         //     }
//         // }
//         // let mut deserializer = serde_json::Deserializer::from_slice(bytes);
//         // T::deserialize(&mut deserializer).map(Json).map_err(map_err)
//     }
// }
// impl<T, S, UnprocessableEntityError> FromRequest<S> for Json<T>
// where
//     T: DeserializeRequest<UnprocessableEntityError = UnprocessableEntityError>,
//     S: Send + Sync,
//     UnprocessableEntityError: Serialize,
// {
//     type Rejection = DeserializeRequestError<UnprocessableEntityError, serde_json::Error>;
//     async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
//         match req.headers().get(axum::http::header::CONTENT_TYPE) {
//             Some(content_type_header) if json_content_type(content_type_header) => {
//                 let bytes = bytes::Bytes::from_request(req, state).await?;
//                 Self::from_bytes(&bytes)
//             }
//             _ => Err(DeserializeRequestError::ContentType),
//         }
//     }
// }

pub fn content_type(content_type_header: &axum::http::HeaderValue) -> Option<mime::Mime> {
    let content_type = content_type_header.to_str().ok()?;
    content_type.parse::<mime::Mime>().ok()
}
pub fn json_content_type(content_type_header: &axum::http::HeaderValue) -> bool {
    fn is_some_and(mime: mime::Mime) -> bool {
        mime.type_() == "application"
            && (mime.subtype() == "json" || mime.suffix().is_some_and(|name| name == "json"))
    }
    content_type(content_type_header).is_some_and(is_some_and)
}

#[non_exhaustive]
#[derive(Debug)]
pub enum DeserializeRequestError<Serde> {
    Serde(Serde),
    ContentType,
    Bytes(axum::extract::rejection::BytesRejection),
}
impl<Serde> IntoResponse for DeserializeRequestError<Serde>
where
    Serde: ToString,
{
    fn into_response(self) -> Response {
        match self {
            Self::Serde(serde) => (StatusCode::BAD_REQUEST, serde.to_string()).into_response(),
            Self::ContentType => StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response(),
            Self::Bytes(bytes) => bytes.into_response(),
        }
    }
}

// #[non_exhaustive]
// #[derive(Debug)]
// pub enum DeserializeRequestError<UnprocessableEntity, Serde> {
//     UnprocessableEntity(UnprocessableEntity),
//     Serde(Serde),
//     ContentType,
//     Bytes(axum::extract::rejection::BytesRejection),
// }
// impl<UnprocessableEntity, Serde> From<axum::extract::rejection::BytesRejection>
//     for DeserializeRequestError<UnprocessableEntity, Serde>
// {
//     fn from(error: axum::extract::rejection::BytesRejection) -> Self {
//         Self::Bytes(error)
//     }
// }
// impl<UnprocessableEntity, Serde> IntoResponse
//     for DeserializeRequestError<UnprocessableEntity, Serde>
// where
//     UnprocessableEntity: Serialize,
//     Serde: ToString,
// {
//     fn into_response(self) -> Response {
//         match self {
//             Self::UnprocessableEntity(unprocessable_entity) => (
//                 StatusCode::UNPROCESSABLE_ENTITY,
//                 axum::Json(unprocessable_entity),
//             )
//                 .into_response(),
//             Self::Serde(serde) => (StatusCode::BAD_REQUEST, serde.to_string()).into_response(),
//             // Self::Data(data) => {
//             //     (StatusCode::UNPROCESSABLE_ENTITY, data.to_string()).into_response()
//             // }
//             // Self::Syntax(syntax) => (StatusCode::BAD_REQUEST, syntax.to_string()).into_response(),
//             // Self::Data(error) | Self::Syntax(error) => {
//             //     (StatusCode::BAD_REQUEST, error.to_string()).into_response()
//             // }
//             Self::ContentType => StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response(),
//             Self::Bytes(bytes) => bytes.into_response(),
//         }
//     }
// }
