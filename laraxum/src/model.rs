//! A model manages the data storage and interacts with the database.

use crate::{Error, ModelError};

/// A database and a table that belongs to it.
pub trait Db<Model> {}

/// Get the `DATABASE_URL` environment variable.
///
/// # Panics
/// - Invalid environment variable.
#[must_use]
pub fn database_url() -> Option<String> {
    crate::macros::env_var_opt!("DATABASE_URL")
}

/// Connect to a database.
pub trait Connect: Sized {
    type Error;
    async fn connect() -> Result<Self, Self::Error>;
}

/// A table in a database.
pub trait Table: Sized {
    type Db: Db<Self> + Send + Sync;
    type Response: Send + Sync;
}

/// A table with records.
///
/// These operations don't require the records to be uniquely identifiable for them to work.
pub trait Collection: Table {
    /// Request to create record.
    type CreateRequest;
    /// Error when creating record.
    type CreateRequestError;

    /// Return all records.
    async fn get_all(db: &Self::Db) -> Result<Vec<Self::Response>, Error>;
    /// Create a record.
    async fn create_one(
        db: &Self::Db,
        rq: Self::CreateRequest,
    ) -> Result<(), ModelError<Self::CreateRequestError>>;
}

/// A table with uniquely identifiable records using an identifier column.
///
/// These operations require the records to be uniquely identifiable for them to work.
pub trait Model: Collection {
    /// The identifier column to identify a record.
    type Id: Copy;
    /// Request to update a record.
    type UpdateRequest;
    /// Error when updating record.
    type UpdateRequestError;
    /// Request to patch update a record.
    type PatchRequest;
    /// Error when patch updating a record.
    type PatchRequestError;

    /// Return a record.
    async fn get_one(db: &Self::Db, id: Self::Id) -> Result<Self::Response, Error>;
    /// Create a record and return it.
    async fn create_get_one(
        db: &Self::Db,
        rq: Self::CreateRequest,
    ) -> Result<Self::Response, ModelError<Self::CreateRequestError>>;
    /// Update a record.
    async fn update_one(
        db: &Self::Db,
        rq: Self::UpdateRequest,
        id: Self::Id,
    ) -> Result<(), ModelError<Self::UpdateRequestError>>;
    /// Update a record and return it.
    async fn update_get_one(
        db: &Self::Db,
        rq: Self::UpdateRequest,
        id: Self::Id,
    ) -> Result<Self::Response, ModelError<Self::UpdateRequestError>> {
        Self::update_one(db, rq, id).await?;
        let rs = Self::get_one(db, id).await?;
        Ok(rs)
    }
    /// Patch update a record.
    async fn patch_one(
        db: &Self::Db,
        rq: Self::PatchRequest,
        id: Self::Id,
    ) -> Result<(), ModelError<Self::PatchRequestError>>;
    /// Patch update a record and return it.
    async fn patch_get_one(
        db: &Self::Db,
        rq: Self::PatchRequest,
        id: Self::Id,
    ) -> Result<Self::Response, ModelError<Self::PatchRequestError>> {
        Self::patch_one(db, rq, id).await?;
        let rs = Self::get_one(db, id).await?;
        Ok(rs)
    }
    /// Delete a record.
    async fn delete_one(db: &Self::Db, id: Self::Id) -> Result<(), Error>;
}

/// A table with a value column and an identifier column to identify multiple values.
///
/// It can be implemented for each column.
/// The `AggregateBy` type generic is a marker type for the identifier column.  
///
/// This can be used to create many-to-many relationships.
pub trait ManyModel<AggregateBy>: Table {
    /// The identifier column used to identify many value columns.
    type OneRequest;
    /// The value column in a request.
    type ManyRequest;
    /// The value column in a response.
    type ManyResponse;

    /// Return many value columns.
    async fn get_many(
        db: &Self::Db,
        one: Self::OneRequest,
    ) -> Result<Vec<Self::ManyResponse>, Error>;
    /// Create many value columns.
    async fn create_many(
        db: &Self::Db,
        one: Self::OneRequest,
        many: &[Self::ManyRequest],
    ) -> Result<(), Error>;
    /// Update many value columns.
    async fn update_many(
        db: &Self::Db,
        one: Self::OneRequest,
        many: &[Self::ManyRequest],
    ) -> Result<(), Error>;
    /// Delete many value columns.
    async fn delete_many(db: &Self::Db, one: Self::OneRequest) -> Result<(), Error>;
}

/// A collection where many records can be aggregated.
pub trait AggregateMany<AggregateBy>: Collection {
    type OneRequest<'a>;
    type ManyResponse;
    async fn aggregate_many<'a>(
        db: &Self::Db,
        one: Self::OneRequest<'a>,
    ) -> Result<Vec<Self::ManyResponse>, Error>;
}
/// A collection where a single record can be aggregated.
pub trait AggregateOne<AggregateBy>: Collection {
    type OneRequest<'a>;
    type OneResponse;
    async fn aggregate_one<'a>(
        db: &Self::Db,
        one: Self::OneRequest<'a>,
    ) -> Result<Self::OneResponse, Error>;
    async fn aggregate_option<'a>(
        db: &Self::Db,
        one: Self::OneRequest<'a>,
    ) -> Result<Option<Self::OneResponse>, Error> {
        match Self::aggregate_one(db, one).await {
            Ok(rs) => Ok(Some(rs)),
            Err(Error::NotFound) => Ok(None),
            Err(err) => Err(err),
        }
    }
    async fn aggregate_one_vec<'a>(
        db: &Self::Db,
        one: Self::OneRequest<'a>,
    ) -> Result<Vec<Self::OneResponse>, Error> {
        let response = Self::aggregate_option(db, one).await?;
        let response = response.into_iter().collect();
        Ok(response)
    }
}

#[derive(serde::Deserialize, Clone, Copy, Default)]
pub enum Sort {
    #[default]
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}

/// Decode from the value stored in the database.
pub trait Decode {
    type Decode;
    fn decode(decode: Self::Decode) -> Self;
}
/// Encode into the value stored in the database.
pub trait Encode {
    type Encode;
    fn encode(self) -> Self::Encode;
}

crate::impl_encode_decode_self! {
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
crate::impl_encode_decode_self! {
    time::OffsetDateTime,
    time::PrimitiveDateTime,
    time::Date,
    time::Time,
    time::Duration,
}
#[cfg(feature = "chrono")]
crate::impl_encode_decode_self! {
    chrono::DateTime::<chrono::Utc>,
    chrono::DateTime::<chrono::Local>,
    chrono::NaiveDateTime,
    chrono::NaiveDate,
    chrono::NaiveTime,
    chrono::TimeDelta,
}

// mysql stores `bool`s as `i8`, so we need to convert it.
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
            i8::from(self)
        }
    }
}
