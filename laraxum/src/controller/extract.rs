//! Axum [Extractors](axum::extract) for extracting data from requests for the controller.

use axum::{
    RequestExt,
    extract::{FromRequest, OptionalFromRequest},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Serialize, de::DeserializeOwned};

/// JSON [Extractor](axum::extract) / [Response](axum::response).
///
/// When used as an extractor, it can deserialize request bodies into some type that
/// implements [DeserializeOwned]. The request will be rejected (and a [DeserializeRequestError] will
/// be returned) if:
///
/// # Errors
/// - The request doesn't have a `Content-Type: application/json` (or similar) header.
/// - The body doesn't contain syntactically valid JSON.
/// - The body contains syntactically valid JSON, but it couldn't be deserialized into the target type.
/// - Buffering the request body fails.
///
/// <div class="warning">
///
/// Since parsing JSON requires consuming the request body, the [Json] extractor must be
/// *last* if there are multiple extractors in a handler.
/// See [`the order of extractors`](axum::extract#the-order-of-extractors).
///
/// </div>
///
/// See [DeserializeRequestError] for more details.
///
/// This struct is a modified version of [axum::Json].
/// See [axum::Json] and [axum::extract::rejection::JsonRejection] for more details.
#[must_use]
#[derive(Debug, Clone, Copy, Default)]
pub struct Json<T>(pub T);
impl<T> Json<T>
where
    T: DeserializeOwned,
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
    T: DeserializeOwned,
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
        if is_json_mime(&mime) {
            let bytes = bytes::Bytes::from_request(req, state)
                .await
                .map_err(DeserializeRequestError::Bytes)?;
            Self::from_bytes(&bytes).map_err(DeserializeRequestError::Serde)
        } else {
            Err(DeserializeRequestError::ContentType)
        }
    }
}
impl<T, State> OptionalFromRequest<State> for Json<T>
where
    T: DeserializeOwned,
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
                if is_json_mime(&mime::Mime::from(content_type)) {
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

/// Is the mime type for json.
fn is_json_mime(mime: &mime::Mime) -> bool {
    mime.type_() == "application"
        && (mime.subtype() == "json" || mime.suffix().is_some_and(|name| name == "json"))
}

/// Error when deserializing request.
#[non_exhaustive]
#[derive(Debug)]
pub enum DeserializeRequestError<Serde> {
    /// The request couldn't be deserialized into the target type.
    Serde(Serde),
    /// The request doesn't have a `Content-Type: application/json` (or similar) header.
    ContentType,
    /// Buffering the request body fails.
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
