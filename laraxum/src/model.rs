use crate::{Error, ModelError};

/// The type used for for an id.
///
/// Currently this must be a `u64`.
/// This will be changed in the future to be more flexible, so don't rely on it too much.
pub type Id = u64;

/// A database and a table that belongs to it.
pub trait Db<Model> {}

/// Get the `DATABASE_URL` environment variable.
///
/// # Panics
/// - Invalid environment variable
#[must_use]
pub fn database_url() -> Option<String> {
    crate::env_var_opt!("DATABASE_URL")
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

/// A table without uniquely identifiable records.
///
/// These operations don't require the records to be uniquely identifiable for them to work.
pub trait Collection: Table {
    /// Request to create record.
    type CreateRequest;
    /// Error when creating record.
    type CreateRequestError;

    /// Get all records.
    async fn get_all(db: &Self::Db) -> Result<Vec<Self::Response>, Error>;
    /// Create a record.
    async fn create_one(
        db: &Self::Db,
        rq: Self::CreateRequest,
    ) -> Result<(), ModelError<Self::CreateRequestError>>;
}

/// A table with uniquely identifiable records.
///
/// These operations require the records to be uniquely identifiable for them to work.
pub trait Model: Collection {
    /// The type to identify an entity.
    type Id: Copy;
    /// Request to update a record
    type UpdateRequest;
    /// Error when updating record.
    type UpdateRequestError;

    /// Get a record with an id
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
    /// Delete a record.
    async fn delete_one(db: &Self::Db, id: Self::Id) -> Result<(), Error>;
}

/// A table with two columns where multiple entities are identified by the other column.
///
/// This can be used to create many-to-many relationships.
pub trait ManyModel<Index>: Table {
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

/// A collection where many records can be filtered by an index.
pub trait CollectionIndexMany<Index>: Collection {
    type OneRequest<'a>;
    type ManyResponse;
    async fn get_index_many<'a>(
        db: &Self::Db,
        one: Self::OneRequest<'a>,
    ) -> Result<Vec<Self::ManyResponse>, Error>;
}
/// A collection where a single record can be filtered by an index.
pub trait CollectionIndexOne<Index>: Collection {
    type OneRequest<'a>;
    type OneResponse;
    async fn get_index_one<'a>(
        db: &Self::Db,
        one: Self::OneRequest<'a>,
    ) -> Result<Self::OneResponse, Error>;
    async fn get_index_one_optional<'a>(
        db: &Self::Db,
        one: Self::OneRequest<'a>,
    ) -> Result<Option<Self::OneResponse>, Error> {
        match Self::get_index_one(db, one).await {
            Ok(rs) => Ok(Some(rs)),
            Err(Error::NotFound) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

pub enum Sort {
    Ascending,
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
