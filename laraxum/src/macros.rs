#[macro_export]
macro_rules! impl_encode_decode {
    { $($ty:ty => $inner:ty),* $(,)? } => {
        $(
            impl $crate::backend::Decode for $ty {
                type Decode = $inner;
                #[inline]
                fn decode(decode: Self::Decode) -> Self {
                    <Self as ::core::convert::From<Self::Decode>>::from(decode)
                }
            }
            impl $crate::backend::Encode for $ty {
                type Encode = $inner;
                #[inline]
                fn encode(self) -> Self::Encode {
                    <Self::Encode as ::core::convert::From<Self>>::from(self)
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! impl_encode_decode_self {
    { $($ty:ty),* $(,)? } => {
        $crate::impl_encode_decode! {
            $($ty => $ty),*
        }
    };
}

#[macro_export]
macro_rules! impl_serde {
    { $($ty:ty => $inner:ty),* $(,)? } => {
        $(
            impl ::serde::Serialize for $ty {
                fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
                where
                    S: ::serde::Serializer,
                {
                    <$inner as ::serde::Serialize>::serialize(
                        &<$inner as ::core::convert::From<$ty>>::from(*self),
                        serializer,
                    )
                }
            }
            impl<'de> ::serde::Deserialize<'de> for $ty {
                fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
                where
                    D: ::serde::Deserializer<'de>,
                {
                    ::core::result::Result::map(
                        <$inner as ::serde::Deserialize>::deserialize(deserializer),
                        <$ty as ::core::convert::From<$inner>>::from,
                    )
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! impl_serde_encode_decode {
    { $($ty:ty => $inner:ty),* $(,)? } => {
        $(
            $crate::impl_encode_decode! { $ty => $inner }
            $crate::impl_serde! { $ty => $inner }
        )*
    };
}
