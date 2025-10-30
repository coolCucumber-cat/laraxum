//! Database types and encoding/decoding.

/// Implement database encoding/decoding for a transparent wrapper type.
#[cfg_attr(not(feature = "macros"), docs(hidden))]
#[macro_export]
macro_rules! transparent_encode_decode {
    { $ty:ty $(,)? $(, $($tt:tt)+)? } => {
        impl $crate::model::types::Decode for $ty {
            type Decode = $ty;
            #[inline]
            fn decode(decode: Self::Decode) -> Self {
                decode
            }
        }
        impl $crate::model::types::Encode for $ty {
            type Encode = $ty;
            #[inline]
            fn encode(self) -> Self::Encode {
                self
            }
        }
        $( $crate::transparent_encode_decode! { $($tt)+ } )?
    };
    { $ty:ty => $inner:ty $(,)? $(, $($tt:tt)+)? } => {
        $crate::transparent_encode_decode! {
            $ty
            => $inner
            => ::core::convert::From::from
            => ::core::convert::From::from
        }
        $( $crate::transparent_encode_decode! { $($tt)+ } )?
    };
    { $ty:ty => $inner:ty => $decode:expr => $encode:expr $(,)? $(, $($tt:tt)+)? } => {
        impl $crate::model::types::Decode for $ty {
            type Decode = <$inner as $crate::model::types::Decode>::Decode;
            #[inline]
            fn decode(decode: Self::Decode) -> Self {
                let decode = <$inner as $crate::model::types::Decode>::decode(decode);
                ($decode)(decode)
            }
        }
        impl $crate::model::types::Encode for $ty {
            type Encode = <$inner as $crate::model::types::Encode>::Encode;
            #[inline]
            fn encode(self) -> Self::Encode {
                let encode = ($encode)(self);
                <$inner as $crate::model::types::Encode>::encode(encode)
            }
        }
        $( $crate::transparent_encode_decode! { $($tt)+ } )?
    };
}

/// Implement frontend serializing/deserializing for a transparent wrapper type.
#[cfg_attr(not(feature = "macros"), docs(hidden))]
#[macro_export]
macro_rules! transparent_serde {
    { $ty:ty => $inner:ty => $deserialize:expr => $serialize:expr $(,)? $(, $($tt:tt)+)? } => {
        impl<'de> ::serde::Deserialize<'de> for $ty {
            fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
                where
                    D: ::serde::Deserializer<'de>,
            {
                ::core::result::Result::map(
                    <$inner as ::serde::Deserialize>::deserialize(deserializer),
                    $deserialize,
                )
            }
        }
        impl ::serde::Serialize for $ty {
            fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
                where
                    S: ::serde::Serializer,
            {
                <$inner as ::serde::Serialize>::serialize(
                    &($serialize)(*self),
                    serializer,
                )
            }
        }
        $( $crate::transparent_serde! { $($tt)+ } )?
    };
    { $ty:ty => $inner:ty $(,)? $(, $($tt:tt)+)? } => {
        $crate::transparent_serde! {
            $ty
            => $inner
            => <$ty as ::core::convert::From<$inner>>::from
            => <$inner as ::core::convert::From<$ty>>::from
        }
        $( $crate::transparent_serde! { $($tt)+ } )?
    };
}

/// Implement database encoding/decoding and frontend serializing/deserializing for a transparent wrapper type.
#[cfg_attr(not(feature = "macros"), docs(hidden))]
#[macro_export]
macro_rules! transparent {
    { $($ty:ty => $inner:ty $(=> $decode:expr => $encode:expr)?),* $(,)? } => {
        $(
            $crate::transparent_encode_decode! { $ty => $inner $(=> $decode => $encode)? }
            $crate::transparent_serde! { $ty => $inner $(=> $decode => $encode)? }
        )*
    };
}

/// Define an enum wrapper type for an integer type and implement database encoding/decoding and frontend serializing/deserializing for it.
#[cfg_attr(not(feature = "macros"), docs(hidden))]
#[macro_export]
macro_rules! transparent_enum {
    {
        $(#[doc = $ty_doc:expr])*
        $vis:vis enum $ty:ident $tt:tt
    } => {
        $crate::transparent_enum! {
            $(#[doc = $ty_doc])*
            #[repr(u8)]
            $vis enum $ty $tt

        }
    };
    {
        $(#[doc = $ty_doc:expr])*
        #[repr($inner:ty)]
        $vis:vis enum $ty:ident {
            $(#[doc = $var0_doc:expr])*
            #[default]
            $var0:ident = $value0:expr
            $(,
                $(#[doc = $var_doc:expr])*
                $var:ident = $value:expr
            )* $(,)?
        }
    } => {
        $(#[doc = $ty_doc])*
        #[repr($inner)]
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
        $vis enum $ty {
            $(#[doc = $var0_doc])*
            #[default]
            $var0 = $value0,
            $(
                $(#[doc = $var_doc])*
                $var = $value,
            )*
        }
        impl $ty {
            $vis fn try_from_default(value: $inner) -> Self {
                <Self as ::core::convert::TryFrom<$inner>>::try_from(value).unwrap_or_default()
            }
        }
        impl ::core::convert::TryFrom<$inner> for $ty {
            type Error = ();
            fn try_from(value: $inner) -> ::core::result::Result<Self, Self::Error> {
                match value {
                    $value0 => ::core::result::Result::Ok(Self::$var0),
                    $(
                        $value => ::core::result::Result::Ok(Self::$var),
                    )*
                    _ => ::core::result::Result::Err(()),
                }
            }
        }
        impl ::core::convert::From<$ty> for $inner {
            fn from(value: $ty) -> Self {
                match value {
                    <$ty>::$var0 => $value0,
                    $(
                        <$ty>::$var => $value,
                    )*
                }
            }
        }
        $crate::transparent! {
            $ty
            => $inner
            => <$ty>::try_from_default
            => <$inner as ::core::convert::From<$ty>>::from
        }
    };
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

crate::transparent_encode_decode! {
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
crate::transparent_encode_decode! {
    time::OffsetDateTime,
    time::PrimitiveDateTime,
    time::Date,
    time::Time,
    time::Duration,
}
#[cfg(feature = "chrono")]
crate::transparent_encode_decode! {
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
