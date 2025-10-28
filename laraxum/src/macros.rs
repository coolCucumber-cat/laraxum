#[macro_export]
macro_rules! impl_encode_decode {
    { $ty:ty => $inner:ty => $decode:expr => $encode:expr $(, $($tt:tt)+)? $(,)? } => {
        impl $crate::model::Decode for $ty {
            type Decode = <$inner as $crate::model::Decode>::Decode;
            #[inline]
            fn decode(decode: Self::Decode) -> Self {
                let decode = <$inner as $crate::model::Decode>::decode(decode);
                ($decode)(decode)
            }
        }
        impl $crate::model::Encode for $ty {
            type Encode = <$inner as $crate::model::Encode>::Encode;
            #[inline]
            fn encode(self) -> Self::Encode {
                let encode = ($encode)(self);
                <$inner as $crate::model::Encode>::encode(encode)
            }
        }
        $( $crate::impl_encode_decode! { $($tt)+ } )?
    };
    { $ty:ty => $inner:ty $(, $($tt:tt)+)? $(,)? } => {
        $crate::impl_encode_decode! {
            $ty
            => $inner
            => ::core::convert::From::from
            => ::core::convert::From::from
            $(, $($tt)+)?
        }
    };
}

#[macro_export]
macro_rules! impl_encode_decode_self {
    { $($ty:ty),* $(,)? } => {
        $(
            impl $crate::model::Decode for $ty {
                type Decode = $ty;
                #[inline]
                fn decode(decode: Self::Decode) -> Self {
                    ::core::convert::From::from(decode)
                }
            }
            impl $crate::model::Encode for $ty {
                type Encode = $ty;
                #[inline]
                fn encode(self) -> Self::Encode {
                    ::core::convert::From::from(self)
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! impl_serde {
    { $ty:ty => $inner:ty => $deserialize:expr => $serialize:expr $(, $($tt:tt)+)? $(,)? } => {
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
        $( $crate::impl_serde! { $($tt)+ } )?
    };
    { $ty:ty => $inner:ty $(, $($tt:tt)+)? $(,)? } => {
        $crate::impl_serde! {
            $ty
            => $inner
            => <$ty as ::core::convert::From<$inner>>::from
            => <$inner as ::core::convert::From<$ty>>::from
        }
    };
}

/// Implement traits for wrapper type.
#[macro_export]
macro_rules! transparent {
    { $($ty:ty => $inner:ty $(=> $decode:expr => $encode:expr)?),* $(,)? } => {
        $(
            $crate::impl_encode_decode! { $ty => $inner $(=> $decode => $encode)? }
            $crate::impl_serde! { $ty => $inner $(=> $decode => $encode)? }
        )*
    };
}

/// Create an enum wrapper type for integers and implement traits.
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

/// Get environment variable.
///
/// # Panics
/// - Not present.
/// - Not unicode.
macro_rules! env_var {
    ($env_var:expr) => {
        match ::std::env::var($env_var) {
            ::core::result::Result::Ok(ok) => ok,
            ::core::result::Result::Err(::std::env::VarError::NotPresent) => {
                ::core::panic!(::core::concat!(
                    "environment variable \"",
                    $env_var,
                    "\" not found"
                ));
            }
            ::core::result::Result::Err(::std::env::VarError::NotUnicode(ref s)) => {
                ::core::panic!(
                    ::core::concat!(
                        "environment variable \"",
                        $env_var,
                        "\" was not valid unicode: {:?}"
                    ),
                    s
                );
            }
        }
    };
}
pub(crate) use env_var;
/// Get optional environment variable.
///
/// # Panics
/// - Not unicode.
macro_rules! env_var_opt {
    ($env_var:expr) => {
        match ::std::env::var($env_var) {
            ::core::result::Result::Ok(ok) => ::core::option::Option::Some(ok),
            ::core::result::Result::Err(::std::env::VarError::NotPresent) => {
                ::core::option::Option::None
            }
            ::core::result::Result::Err(::std::env::VarError::NotUnicode(ref s)) => {
                ::core::panic!(
                    ::core::concat!(
                        "environment variable \"",
                        $env_var,
                        "\" was not valid unicode: {:?}"
                    ),
                    s
                );
            }
        }
    };
}
pub(crate) use env_var_opt;
/// Get environment variable with default.
///
/// # Panics
/// - Not unicode.
macro_rules! env_var_default {
    ($env_var:expr, $default:expr) => {
        $crate::macros::env_var_opt!($env_var)
            .map(::std::borrow::Cow::Owned)
            .unwrap_or(::std::borrow::Cow::Borrowed($default))
    };
}
pub(crate) use env_var_default;
