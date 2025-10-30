//! Authentication and authorization for the controller.
//!
//! Implement [Authenticate] for credentials to authenticate them.
//! Implement [Authorize] to authorize the authenticated credentials.
//! Any type that implements [Authenticate] also automatically implements [Authorize].
//! Only authorization needs to be implemented for a type and
//! it delegates the authentication to the the [Authorize::Authenticate] type.
//! Therefore all credentials must implement [Authorize]
//! but only one type has to implement [Authenticate].
//! One type can have shared authentication logic but
//! there can be many types for authorization logic,
//! for example to have many different levels of access.
//!
//! Implement [AuthenticateToken] for credentials
//! to encode them into and decode them from a JSON Web Token.
//!
//! [`AuthToken<T>`] is an Axum [Extractor](axum::extract)
//! which extracts and decodes the credentials from a JSON Web Token,
//! then authenticates and authorizes them.
//! When a user logs in or registers, encode their credentials into a JSON Web Token and
//! return them so the user can use it to authenticate themselves in subsequent requests.
//!
//! ⚠️ JSON Web Tokens are *not* encrypted.
//! Do not send sensitive information like credit card details, passwords or emails.
//! Anyone can read them, but only you can create a valid one.
//! Changing a JSON Web Token will invalidate it so it is impossible to fake.
//!
//! ⚠️ JSON Web Tokens have security vulnerabilities if implemented incorrectly.
//! They can't be faked but they can be stolen.
//! Always make sure you have good security practices on the frontend *and* on the backend.
//!
//! # Example
//!
//! ```
//! #[derive(serde::Serialize, serde::Deserialize)]
//! struct UserCredentials {
//!     is_admin: bool,
//! }
//! /// Authenticate user credentials.
//! impl Authenticate for UserCredentials {
//!     type State = AppDb;
//!     /// Authenticate the user credentials.
//!     ///
//!     /// Extra optional authentication logic for user credentials,
//!     /// for example to check if the user has been banned, blacklisted or logged out.  
//!     fn authenticate(&self, state: &Arc<Self::State>) -> Result<(), AuthError> {
//!         // No extra logic needed, can be left empty.
//!         Ok(())
//!     }
//! }
//! /// Authenticate the user with a JSON Web Token.
//! ///
//! /// All values have sensible defaults.
//! impl AuthenticateToken for UserCredentials {}
//!
//! /// A struct using the `UserCredentials` token for authentication and extending it with authorization logic.
//! struct UserAdminCredentials;
//! impl Authorize for UserAdminCredentials {
//!     /// Use `UserCredentials` for authentication.
//!     type Authenticate = UserCredentials;
//!     /// Add authorization logic.
//!     fn authorize(authorize: Self::Authenticate) -> Result<Self, AuthError> {
//!         if authorize.is_admin {
//!             Ok(UserAdminCredentials)
//!         } else {
//!             Err(AuthError::Unauthorized)
//!         }
//!     }
//! }
//!
//! #[db()]
//! mod AppDb {
//!     #[db(model(), controller())]
//!     struct Anyone {}
//!
//!     #[db(model(), controller(auth(UserCredentials)))]
//!     struct UserOnly {}
//!
//!     #[db(model(), controller(auth(UserAdminCredentials)))]
//!     struct AdminOnly {}
//! }
//! ```

use crate::error::AuthError;

use std::sync::Arc;

use axum::extract::FromRequestParts;
use serde::{Deserialize, Serialize};

/// Authenticate the user credentials.
///
/// Provides extra optional authentication logic for user credentials
/// independently of the technology used to implement the credentials such as JSON Web Tokens.  
pub trait Authenticate {
    type State;
    /// Authenticate the user credentials.
    ///
    /// Extra optional authentication logic for user credentials,
    /// for example to check if the user has been banned, blacklisted or logged out.  
    ///
    /// Can be left out for most use cases.  
    ///
    /// # Errors
    /// - Authentication fails, see [AuthError].
    #[expect(unused_variables)]
    fn authenticate(&self, state: &Arc<Self::State>) -> Result<(), AuthError> {
        Ok(())
    }
}

/// Authorize the user credentials.
///
/// It delegates the authentication to the the [Authorize::Authenticate] type
/// so that authentication logic can easily be extended and reused.  
pub trait Authorize: Sized {
    /// The type to delegate authentication to, before authorization.
    type Authenticate: Authenticate;
    /// Authorize the user.
    ///
    /// For example, checking if the user is an admin.
    ///
    /// # Errors
    /// - Authorization fails, see [AuthError].
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

/// Authenticate the user with a JSON Web Token.
///
/// All values have sensible defaults.
///
/// See [jsonwebtoken].
pub trait AuthenticateToken: Authenticate + Serialize + for<'a> Deserialize<'a> + Sized {
    /// Duration after which the token expires.
    ///
    /// Default is 4 hours.  
    #[must_use]
    fn exp_duration() -> core::time::Duration {
        core::time::Duration::from_secs(60 * 60 * 4)
    }
    /// Encryption and decryption keys.
    ///
    /// See [AuthKeys].  
    /// See [jsonwebtoken::EncodingKey].  
    /// See [jsonwebtoken::DecodingKey].  
    #[must_use]
    #[cfg(feature = "auth_token")]
    fn authentication_keys() -> &'static AuthKeys {
        static KEYS: std::sync::LazyLock<AuthKeys> = std::sync::LazyLock::new(AuthKeys::new);
        &KEYS
    }
    /// JSON Web Token validation options.
    ///
    /// See [jsonwebtoken::Validation].  
    #[must_use]
    #[cfg(feature = "auth_token")]
    fn authentication_validation() -> &'static jsonwebtoken::Validation {
        static VALIDATION: std::sync::LazyLock<jsonwebtoken::Validation> =
            std::sync::LazyLock::new(jsonwebtoken::Validation::default);
        &VALIDATION
    }
    /// JSON Web Token header.
    ///
    /// See [jsonwebtoken::Header].  
    #[must_use]
    #[cfg(feature = "auth_token")]
    fn authentication_header() -> &'static jsonwebtoken::Header {
        static HEADER: std::sync::LazyLock<jsonwebtoken::Header> =
            std::sync::LazyLock::new(jsonwebtoken::Header::default);
        &HEADER
    }
}

/// The underlying JSON Web Token to serialize and deserialize.
///
/// [jsonwebtoken] uses the `exp` field to verify the token is not expired.
#[derive(Serialize, Deserialize, Clone)]
struct AuthTokenExp<T>
where
    T: AuthenticateToken,
{
    pub exp: usize,
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
            .as_secs();
        let exp = usize::try_from(exp).unwrap_or(usize::MAX);
        Self { exp, token }
    }
    pub const fn new_with_secs(token: T, secs: usize) -> Self {
        Self { exp: secs, token }
    }
}
impl<T> AuthTokenExp<T>
where
    T: AuthenticateToken + Clone,
{
    /// Encode the token with the expiration date given by the [AuthenticateToken] trait.
    ///
    /// # Errors
    /// - Encoding fails, see [jsonwebtoken::errors::Error].
    #[cfg(feature = "auth_token")]
    pub fn encode(&self) -> Result<String, jsonwebtoken::errors::Error> {
        jsonwebtoken::encode::<Self>(
            T::authentication_header(),
            self,
            &T::authentication_keys().encoding,
        )
    }
    /// Decode the token.
    ///
    /// # Errors
    /// - Decoding fails, see [jsonwebtoken::errors::Error].
    #[cfg(feature = "auth_token")]
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

/// Authenticate and authorize the user with a JSON Web Token.
///
/// See [AuthenticateToken] for JSON Web Token authentication.  
/// See [Authenticate] for authentication.  
/// See [Authorize] for authorization.  
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
    T: AuthenticateToken + Clone,
{
    /// Encode the token with the expiration date given by the [AuthenticateToken] trait.
    ///
    /// # Errors
    /// - Encoding fails, see [jsonwebtoken::errors::Error].
    #[cfg(feature = "auth_token")]
    pub fn encode(self) -> Result<String, AuthError> {
        AuthTokenExp::encode(&AuthTokenExp::from(self)).map_err(|_| AuthError::Unauthenticated)
    }
    /// Decode the token.
    ///
    /// # Errors
    /// - Decoding fails, see [jsonwebtoken::errors::Error].
    #[cfg(feature = "auth_token")]
    fn decode(token: &str) -> Result<Self, AuthError> {
        AuthTokenExp::decode(token)
            .map(Self::from)
            .map_err(|_| AuthError::Unauthenticated)
    }
}

#[cfg(feature = "auth_token")]
impl<T, U, State> FromRequestParts<Arc<State>> for AuthToken<T>
where
    T: Authorize<Authenticate = U>,
    U: Authenticate<State = State> + AuthenticateToken + Clone,
    State: Send + Sync,
{
    type Rejection = AuthError;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &Arc<State>,
    ) -> Result<Self, Self::Rejection> {
        use axum::RequestPartsExt;
        use axum_extra::{
            TypedHeader,
            headers::{Authorization, authorization::Bearer},
        };
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::Unauthenticated)?;
        let token = bearer.token();
        let token = AuthToken::<U>::decode(token)?;
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

/// Get the `AUTH_SECRET` environment variable.
///
/// # Panics
/// - Missing environment variable.
/// - Invalid environment variable.
#[must_use]
pub fn auth_secret() -> String {
    crate::env::env_var!("AUTH_SECRET")
}

/// Encryption and decryption keys for encoding and decoding JSON Web Tokens.
///
/// Gets secret from `AUTH_SECRET` environment variable.  
///
/// See [jsonwebtoken].  
/// See [jsonwebtoken::EncodingKey].  
/// See [jsonwebtoken::DecodingKey].  
#[cfg(feature = "auth_token")]
pub struct AuthKeys {
    pub encoding: jsonwebtoken::EncodingKey,
    pub decoding: jsonwebtoken::DecodingKey,
}
#[cfg(feature = "auth_token")]
impl AuthKeys {
    #[must_use]
    pub fn from_secret(secret: &[u8]) -> Self {
        Self {
            encoding: jsonwebtoken::EncodingKey::from_secret(secret),
            decoding: jsonwebtoken::DecodingKey::from_secret(secret),
        }
    }
    /// Gets secret from `AUTH_SECRET` environment variable.  
    ///
    /// See [jsonwebtoken].  
    /// See [jsonwebtoken::EncodingKey].  
    /// See [jsonwebtoken::DecodingKey].  
    ///
    /// # Panics
    /// - Missing environment variable.
    /// - Invalid environment variable.
    #[must_use]
    pub fn new() -> Self {
        let auth_secret = auth_secret();
        Self::from_secret(auth_secret.as_bytes())
    }
}

/// Implement authorization for a type that can be compared to the authentication type.
#[cfg_attr(not(feature = "macros"), docs(hidden))]
#[macro_export]
macro_rules! authorize {
    {
        $(
            $ty:ty => $var_ty:ident => $var:expr
        ),* $(,)?
    } => {
        $(
            impl $crate::Authorize for $var_ty {
                type Authenticate = $ty;
                fn authorize(
                    authenticate: Self::Authenticate,
                ) -> ::core::result::Result<Self, $crate::error::AuthError> {
                    if authenticate >= $var {
                        ::core::result::Result::Ok($var_ty)
                    } else {
                        ::core::result::Result::Err($crate::error::AuthError::Unauthorized)
                    }
                }
            }
        )*
    };
}
