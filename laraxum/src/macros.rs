#[macro_export]
macro_rules! impl_encode_decode {
    { $ty:ty => $inner:ty => $decode:expr => $encode:expr $(, $($tt:tt)+)? $(,)? } => {
        impl $crate::backend::Decode for $ty {
            type Decode = <$inner as $crate::backend::Decode>::Decode;
            #[inline]
            fn decode(decode: Self::Decode) -> Self {
                let decode = <$inner as $crate::backend::Decode>::decode(decode);
                ($decode)(decode)
            }
        }
        impl $crate::backend::Encode for $ty {
            type Encode = <$inner as $crate::backend::Encode>::Encode;
            #[inline]
            fn encode(self) -> Self::Encode {
                let encode = ($encode)(self);
                <$inner as $crate::backend::Encode>::encode(encode)
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
            impl $crate::backend::Decode for $ty {
                type Decode = $ty;
                #[inline]
                fn decode(decode: Self::Decode) -> Self {
                    ::core::convert::From::from(decode)
                }
            }
            impl $crate::backend::Encode for $ty {
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

/// Get environment variable, else panic.
#[macro_export]
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
/// Get optional environment variable, else panic.
#[macro_export]
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

/// Serve the router.
#[macro_export]
macro_rules! serve {
    ($app:expr) => {
        async {
            let url = $crate::frontend::url();
            let url = &*url;
            let app_listener = ::tokio::net::TcpListener::bind(url).await?;
            ::std::println!("Listening at: {url:?}");
            ::axum::serve(app_listener, $app).await?;
            ::core::result::Result::Ok(())
        }
    };
}

/// Implement authorization for a type that can be compared to the authentication type.
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
