use crate::{Error, ModelError};

pub type Id = u64;

pub trait Db<Model> {}

pub trait AnyDb: Sized {
    type Db;
    type Driver: sqlx::Database;
    type ConnectionOptions: sqlx::ConnectOptions;
    fn default_options() -> Self::ConnectionOptions;
    async fn connect_with_options(options: Self::ConnectionOptions) -> Result<Self, sqlx::Error>;
    async fn connect() -> Result<Self, sqlx::Error> {
        let url = std::env::var("DATABASE_URL");
        let options = match url {
            Ok(url) => url
                .parse()
                .map_err(|e| sqlx::Error::Configuration(Box::new(e))),
            Err(std::env::VarError::NotPresent) => Ok(Self::default_options()),
            Err(e) => Err(sqlx::Error::Configuration(Box::new(e))),
        }?;
        Self::connect_with_options(options).await
    }
    fn db(&self) -> &Self::Db;
}

pub trait Table: Sized {
    type Db: Db<Self> + Send + Sync;
    type Response: Send + Sync;
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
// pub trait Collection2: Table {
//     type GetAllRequestQuery;
//     type CreateRequest: Send;
//     type CreateRequestError;
//
//     fn get_all(db: &Self::Db) -> impl Future<Output = Result<Vec<Self::Response>, Error>> + Send;
//     fn create_one(
//         db: &Self::Db,
//         r: Self::CreateRequest,
//     ) -> impl Future<Output = Result<(), ModelError<Self::CreateRequestError>>> + Send;
// }

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

// pub trait Model2: Collection2 {
//     type Id: Copy + Send + Sync;
//     type UpdateRequest: Send;
//     type UpdateRequestError;
//
//     fn get_one(
//         db: &Self::Db,
//         id: Self::Id,
//     ) -> impl Future<Output = Result<Self::Response, Error>> + Send;
//     fn create_get_one(
//         db: &Self::Db,
//         rq: Self::CreateRequest,
//     ) -> impl Future<Output = Result<Self::Response, ModelError<Self::CreateRequestError>>> + Send;
//     fn update_one(
//         db: &Self::Db,
//         rq: Self::UpdateRequest,
//         id: Self::Id,
//     ) -> impl Future<Output = Result<(), ModelError<Self::UpdateRequestError>>> + Send;
//     fn update_get_one(
//         db: &Self::Db,
//         rq: Self::UpdateRequest,
//         id: Self::Id,
//     ) -> impl Future<Output = Result<Self::Response, ModelError<Self::UpdateRequestError>>> + Send
//     {
//         async move {
//             Self::update_one(db, rq, id).await?;
//             let rs = Self::get_one(db, id).await?;
//             Ok(rs)
//         }
//     }
//     fn delete_one(db: &Self::Db, id: Self::Id) -> impl Future<Output = Result<(), Error>> + Send;
// }

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

pub trait Decode {
    type Decode;
    fn decode(decode: Self::Decode) -> Self;
}
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
