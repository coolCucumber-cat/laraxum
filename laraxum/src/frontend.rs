use crate::{
    backend::{Collection, Model, Table},
    error::{AuthError, Error, ModelError},
};

use core::ops::Deref;
use std::{borrow::Cow, sync::Arc};

use axum::{
    RequestExt, RequestPartsExt,
    extract::{FromRequest, FromRequestParts, OptionalFromRequest, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

/// Get the URL environment variable. Defaults to `localhost:80`.
///
/// # Panics
/// - Invalid environment variable
pub fn url() -> Cow<'static, str> {
    crate::env_var_opt!("URL")
        .map(Cow::Owned)
        .unwrap_or(Cow::Borrowed("localhost:80"))
}

/// Accept and process requests.
///
/// Every function represents a function in a REST API and has a state and auth token.
/// The state contains the database.
/// The auth token authenticates and authorizes the user with a token.
/// Use a empty tuple `()` to not authenticate the user and accept all users.
/// The [`crate::Authenticate`](Authenticate) and [`crate::Authorize`](Authorize) traits
/// can be implemented to add custom authentication and authorization behaviour.
pub trait Controller: Model
where
    <Self as Table>::Response: Serialize,
    <Self as Collection>::CreateRequest: for<'a> Deserialize<'a>,
    <Self as Collection>::CreateRequestError: Serialize,
    <Self as Model>::Id: for<'a> Deserialize<'a>,
    <Self as Model>::UpdateRequest: for<'a> Deserialize<'a>,
    <Self as Model>::UpdateRequestError: Serialize,
    AuthToken<Self::Auth>: FromRequestParts<Arc<Self::State>>,
{
    type State: Deref<Target = Self::Db>;
    type Auth;
    type GetAllRequestQuery: for<'a> Deserialize<'a>;

    #[expect(unused_variables)]
    async fn get_many(
        State(state): State<Arc<Self::State>>,
        AuthToken(_): AuthToken<Self::Auth>,
        Query(query): Query<Self::GetAllRequestQuery>,
    ) -> Result<Json<Vec<Self::Response>>, Error> {
        let rs = Self::get_all(&*state).await?;
        Ok(Json(rs))
    }
    async fn get(
        State(state): State<Arc<Self::State>>,
        AuthToken(_): AuthToken<Self::Auth>,
        Path(id): Path<Self::Id>,
    ) -> Result<Json<Self::Response>, Error> {
        let rs = Self::get_one(&*state, id).await?;
        Ok(Json(rs))
    }
    async fn create(
        State(state): State<Arc<Self::State>>,
        AuthToken(_): AuthToken<Self::Auth>,
        Json(rq): Json<Self::CreateRequest>,
    ) -> Result<Json<Self::Response>, ModelError<Self::CreateRequestError>> {
        let rs = Self::create_get_one(&*state, rq).await?;
        Ok(Json(rs))
    }
    async fn update(
        State(state): State<Arc<Self::State>>,
        AuthToken(_): AuthToken<Self::Auth>,
        Path(id): Path<Self::Id>,
        Json(rq): Json<Self::UpdateRequest>,
    ) -> Result<Json<Self::Response>, ModelError<Self::UpdateRequestError>> {
        let rs = Self::update_get_one(&*state, rq, id).await?;
        Ok(Json(rs))
    }
    async fn delete(
        State(state): State<Arc<Self::State>>,
        AuthToken(_): AuthToken<Self::Auth>,
        Path(id): Path<Self::Id>,
    ) -> Result<(), Error> {
        Self::delete_one(&*state, id).await?;
        Ok(())
    }
}

/// JSON Extractor / Response.
///
/// When used as an extractor, it can deserialize request bodies into some type that
/// implements [`serde::de::DeserializeOwned`]. The request will be rejected (and a [`DeserializeRequestError`] will
/// be returned) if:
///
/// # Errors
/// - The request doesn't have a `Content-Type: application/json` (or similar) header.
/// - The body doesn't contain syntactically valid JSON.
/// - The body contains syntactically valid JSON, but it couldn't be deserialized into the target type.
/// - Buffering the request body fails.
///
/// ⚠️ Since parsing JSON requires consuming the request body, the `Json` extractor must be
/// *last* if there are multiple extractors in a handler.
/// See [`the order of extractors`](axum::extract#the-order-of-extractors)
///
/// See [`DeserializeRequestError`] for more details.
#[must_use]
#[derive(Debug, Clone, Copy, Default)]
pub struct Json<T>(pub T);
impl<T> Json<T>
where
    T: serde::de::DeserializeOwned,
{
    /// Deserialize JSON from bytes.
    ///
    /// # Errors
    /// - Deserialization fails.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        let mut deserializer = serde_json::Deserializer::from_slice(bytes);
        T::deserialize(&mut deserializer).map(Json)
    }
}
impl<T, State> FromRequest<State> for Json<T>
where
    T: serde::de::DeserializeOwned,
    State: Send + Sync,
{
    type Rejection = DeserializeRequestError<serde_json::Error>;
    async fn from_request(
        mut req: axum::extract::Request,
        state: &State,
    ) -> Result<Self, Self::Rejection> {
        use axum_extra::{TypedHeader, headers::ContentType};
        let TypedHeader(content_type) = req
            .extract_parts::<TypedHeader<ContentType>>()
            .await
            .map_err(|_| DeserializeRequestError::ContentType)?;
        let mime = mime::Mime::from(content_type);
        if json_mime(&mime) {
            let bytes = bytes::Bytes::from_request(req, state)
                .await
                .map_err(DeserializeRequestError::Bytes)?;
            Self::from_bytes(&bytes).map_err(DeserializeRequestError::Serde)
        } else {
            Err(DeserializeRequestError::ContentType)
        }

        // let content_type = req.extract_parts::<TypedHeader<ContentType>>().await;
        // let mime = content_type.map(|TypedHeader(content_type)| mime::Mime::from(content_type));
        //
        // match mime {
        //     Ok(mime) if json_mime(mime) => {
        //         let bytes = bytes::Bytes::from_request(req, state)
        //             .await
        //             .map_err(DeserializeRequestError::Bytes)?;
        //         Self::from_bytes(&bytes).map_err(DeserializeRequestError::Serde)
        //     }
        //     _ => Err(DeserializeRequestError::ContentType),
        // }
        // if content_type.is_ok_and(|TypedHeader(content_type)| json_mime(content_type.into())) {
        //     let bytes = bytes::Bytes::from_request(req, state)
        //         .await
        //         .map_err(DeserializeRequestError::Bytes)?;
        //     Self::from_bytes(&bytes).map_err(DeserializeRequestError::Serde)
        // } else {
        //     Err(DeserializeRequestError::ContentType)
        // }
        // match req.extract_parts::<TypedHeader<ContentType>>().await {
        //     Ok(TypedHeader(content_type)) => {
        //         let bytes = bytes::Bytes::from_request(req, state)
        //             .await
        //             .map_err(DeserializeRequestError::Bytes)?;
        //         Self::from_bytes(&bytes).map_err(DeserializeRequestError::Serde)
        //     }
        //     _ => Err(DeserializeRequestError::ContentType),
        // }
        // match req.headers().get(axum::http::header::CONTENT_TYPE) {
        //     Some(content_type_header) if json_content_type(content_type_header) => {
        //         let bytes = bytes::Bytes::from_request(req, state)
        //             .await
        //             .map_err(DeserializeRequestError::Bytes)?;
        //         Self::from_bytes(&bytes).map_err(DeserializeRequestError::Serde)
        //     }
        //     _ => Err(DeserializeRequestError::ContentType),
        // }
    }
}
impl<T, State> OptionalFromRequest<State> for Json<T>
where
    T: serde::de::DeserializeOwned,
    State: Send + Sync,
{
    type Rejection = DeserializeRequestError<serde_json::Error>;
    async fn from_request(
        mut req: axum::extract::Request,
        state: &State,
    ) -> Result<Option<Self>, Self::Rejection> {
        use axum_extra::{TypedHeader, headers::ContentType};
        let content_type = req
            .extract_parts::<Option<TypedHeader<ContentType>>>()
            .await
            .map_err(|_| DeserializeRequestError::ContentType)?;

        match content_type {
            Some(TypedHeader(content_type)) => {
                if json_mime(&mime::Mime::from(content_type)) {
                    let bytes = bytes::Bytes::from_request(req, state)
                        .await
                        .map_err(DeserializeRequestError::Bytes)?;
                    let t = Self::from_bytes(&bytes).map_err(DeserializeRequestError::Serde)?;
                    Ok(Some(t))
                } else {
                    Err(DeserializeRequestError::ContentType)
                }
            }
            None => Ok(None),
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
fn json_mime(mime: &mime::Mime) -> bool {
    mime.type_() == "application"
        && (mime.subtype() == "json" || mime.suffix().is_some_and(|name| name == "json"))
}
pub fn json_content_type(content_type_header: &axum::http::HeaderValue) -> bool {
    content_type(content_type_header)
        .as_ref()
        .is_some_and(json_mime)
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

pub trait Authenticate {
    type State;
    /// Authenticate the user.
    ///
    /// To authorize the user, use the [`Authorize`] trait instead so this trait only has to be implemented once.
    ///
    /// # Errors
    /// - Authentication fails.
    #[expect(unused_variables)]
    fn authenticate(&self, state: &Arc<Self::State>) -> Result<(), AuthError> {
        Ok(())
    }
}

pub trait Authorize: Sized {
    type Authenticate: Authenticate;
    /// Authorize the user.
    ///
    /// # Errors
    /// - Authorization fails.
    fn authorize(authorize: Self::Authenticate) -> Result<Self, AuthError>;
}
impl<T> Authorize for T
where
    T: Authenticate,
{
    type Authenticate = Self;
    fn authorize(authenticate: Self::Authenticate) -> Result<Self, AuthError> {
        Ok(authenticate)
    }
}

pub trait AuthenticateToken: Authenticate + Serialize + for<'a> Deserialize<'a> + Sized {
    #[must_use]
    fn exp_duration() -> core::time::Duration {
        core::time::Duration::from_secs(60 * 4)
    }
    #[must_use]
    fn authentication_keys() -> &'static AuthKeys {
        static KEYS: std::sync::LazyLock<AuthKeys> = std::sync::LazyLock::new(AuthKeys::new);
        &KEYS
    }
    #[must_use]
    fn authentication_validation() -> &'static jsonwebtoken::Validation {
        static VALIDATION: std::sync::LazyLock<jsonwebtoken::Validation> =
            std::sync::LazyLock::new(jsonwebtoken::Validation::default);
        &VALIDATION
    }
    #[must_use]
    fn authentication_header() -> &'static jsonwebtoken::Header {
        static HEADER: std::sync::LazyLock<jsonwebtoken::Header> =
            std::sync::LazyLock::new(jsonwebtoken::Header::default);
        &HEADER
    }
}

#[derive(Serialize, Deserialize)]
pub struct AuthTokenExp<T>
where
    T: AuthenticateToken,
{
    pub exp: u128,
    #[serde(bound = "T: AuthenticateToken")]
    pub token: T,
}
impl<T> AuthTokenExp<T>
where
    T: AuthenticateToken,
{
    pub fn new(token: T, duration: core::time::Duration) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let exp = now
            .checked_add(duration)
            .unwrap_or(core::time::Duration::MAX)
            .as_millis();
        Self { exp, token }
    }
    pub const fn new_with_millis(token: T, millis: u128) -> Self {
        Self { exp: millis, token }
    }
    /// Encode the token with the expiration date given by the [Authenticate] trait
    ///
    /// # Errors
    /// - encoding fails, see [`jsonwebtoken::errors::Error`].
    pub fn encode(&self) -> Result<String, jsonwebtoken::errors::Error> {
        jsonwebtoken::encode::<Self>(
            T::authentication_header(),
            self,
            &T::authentication_keys().encoding,
        )
    }
    fn decode(token: &str) -> Result<Self, jsonwebtoken::errors::Error> {
        jsonwebtoken::decode::<Self>(
            token,
            &T::authentication_keys().decoding,
            T::authentication_validation(),
        )
        .map(|token| token.claims)
    }
}
impl<T> From<AuthToken<T>> for AuthTokenExp<T>
where
    T: AuthenticateToken,
{
    fn from(AuthToken(token): AuthToken<T>) -> Self {
        Self::new(token, T::exp_duration())
    }
}

pub struct AuthToken<T>(pub T);
impl<T> From<AuthTokenExp<T>> for AuthToken<T>
where
    T: AuthenticateToken,
{
    fn from(AuthTokenExp { token, .. }: AuthTokenExp<T>) -> Self {
        Self(token)
    }
}
impl<T> AuthToken<T>
where
    T: AuthenticateToken,
{
    /// Encode the token with the expiration date given by the [Authenticate] trait
    ///
    /// # Errors
    /// - encoding fails, see [`jsonwebtoken::errors::Error`].
    pub fn encode(self) -> Result<String, AuthError> {
        AuthTokenExp::encode(&AuthTokenExp::from(self)).map_err(|_| AuthError::Unauthenticated)
    }
    fn decode(token: &str) -> Result<Self, AuthError> {
        AuthTokenExp::decode(token)
            .map(Self::from)
            .map_err(|_| AuthError::Unauthenticated)
    }
}

impl<T, U, State> FromRequestParts<Arc<State>> for AuthToken<T>
where
    T: Authorize<Authenticate = U>,
    U: Authenticate<State = State> + AuthenticateToken,
    State: Send + Sync,
{
    type Rejection = AuthError;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &Arc<State>,
    ) -> Result<Self, Self::Rejection> {
        use axum_extra::{
            TypedHeader,
            headers::{Authorization, authorization::Bearer},
        };
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::Unauthenticated)?;
        let token = bearer.token();
        let token = AuthToken::<U>::decode(token).map_err(|_| AuthError::Unauthenticated)?;
        U::authenticate(&token.0, state)?;
        let token = Self(T::authorize(token.0)?);
        Ok(token)
    }
}
impl<State> FromRequestParts<State> for AuthToken<()>
where
    State: Send + Sync,
{
    type Rejection = core::convert::Infallible;
    async fn from_request_parts(
        _: &mut axum::http::request::Parts,
        _: &State,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(()))
    }
}

#[must_use]
pub fn auth_secret() -> String {
    crate::env_var!("AUTH_SECRET")
}

pub struct AuthKeys {
    pub encoding: jsonwebtoken::EncodingKey,
    pub decoding: jsonwebtoken::DecodingKey,
}
impl AuthKeys {
    #[must_use]
    pub fn from_secret(secret: &[u8]) -> Self {
        Self {
            encoding: jsonwebtoken::EncodingKey::from_secret(secret),
            decoding: jsonwebtoken::DecodingKey::from_secret(secret),
        }
    }
    #[must_use]
    pub fn new() -> Self {
        let auth_secret = auth_secret();
        Self::from_secret(auth_secret.as_bytes())
    }
}
