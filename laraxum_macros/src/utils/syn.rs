mod kw {
    syn::custom_keyword! { Option }
}

use syn::{
    Ident, Meta, Path, PathSegment, Token, Type, TypePath, punctuated::Punctuated, spanned::Spanned,
};

/// Allow anything that implements [`syn::parse::Parse`] to be used as an attribute by [`darling`].
///
/// Useful when you want an attribute to not need quotes,
/// like `function(a::b::c)` instead of `function = "a::b::c"`.
pub struct TokenStreamAttr<T>(pub T);
impl<T> TokenStreamAttr<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}
impl<T> TokenStreamAttr<T>
where
    T: syn::parse::Parse,
{
    pub fn transform(item: Meta) -> darling::Result<T> {
        <Self as darling::FromMeta>::from_meta(&item).map(Self::into_inner)
    }
    pub fn transform_option(item: Option<Meta>) -> darling::Result<Option<T>> {
        item.map(Self::transform).transpose()
    }
}
impl<T> darling::FromMeta for TokenStreamAttr<T>
where
    T: syn::parse::Parse,
{
    fn from_meta(item: &Meta) -> darling::Result<Self> {
        (match *item {
            Meta::Path(_) => Self::from_word(),
            Meta::List(ref value) => {
                let parse2 = syn::parse2::<T>(value.tokens.clone())?;
                Ok(Self(parse2))
            }
            Meta::NameValue(ref value) => Self::from_expr(&value.value),
        })
        .map_err(|e| e.with_span(item))
    }
}

/// Allow anything that implements [`syn::parse::Parse`] to be used as an optional attribute by [`darling`].
///
/// Like [`TokenStreamAttr<T>`] but for [`Option<T>`] instead of [`T`].
pub struct TokenStreamAttrOption<T>(pub Option<T>);
impl<T> TokenStreamAttrOption<T> {
    pub fn into_inner(self) -> Option<T> {
        self.0
    }
}
impl<T> TokenStreamAttrOption<T>
where
    T: syn::parse::Parse,
{
    pub fn transform(item: Meta) -> darling::Result<Option<T>> {
        <Self as darling::FromMeta>::from_meta(&item).map(Self::into_inner)
    }
    pub fn transform_option(item: Option<Meta>) -> darling::Result<Option<Option<T>>> {
        item.map(Self::transform).transpose()
    }
}
impl<T> darling::FromMeta for TokenStreamAttrOption<T>
where
    T: syn::parse::Parse,
{
    fn from_meta(item: &Meta) -> darling::Result<Self> {
        (match *item {
            Meta::Path(_) => Self::from_word(),
            Meta::List(ref value) => {
                let parse2 = syn::parse2::<T>(value.tokens.clone())?;
                Ok(Self(Some(parse2)))
            }
            Meta::NameValue(ref value) => Self::from_expr(&value.value),
        })
        .map_err(|e| e.with_span(item))
    }
    fn from_word() -> darling::Result<Self> {
        Ok(Self(None))
    }
}

// /// Allow anything that implements [`syn::parse::Parse`] to be used as a list of attributes by [`darling`].
// ///
// /// Like [`TokenStreamAttr<T>`] but for [`Vec<T>`] instead of [`T`].
// #[expect(dead_code)]
// pub struct TokenStreamAttrVec<T>(pub Vec<T>);
// impl<T> TokenStreamAttrVec<T> {
//     pub fn into_inner(self) -> Vec<T> {
//         self.0
//     }
// }
// impl<T> TokenStreamAttrVec<T>
// where
//     T: syn::parse::Parse,
// {
//     pub fn transform(item: Meta) -> darling::Result<Vec<T>> {
//         <Self as darling::FromMeta>::from_meta(&item).map(Self::into_inner)
//     }
//     pub fn transform_option(item: Option<Meta>) -> darling::Result<Option<Vec<T>>> {
//         item.map(Self::transform).transpose()
//     }
// }
// impl<T> darling::FromMeta for TokenStreamAttrVec<T>
// where
//     T: syn::parse::Parse,
// {
//     fn from_meta(item: &Meta) -> darling::Result<Self> {
//         (match *item {
//             Meta::Path(_) => Self::from_word(),
//             Meta::List(ref value) => {
//                 let punctuated = syn::parse::Parser::parse2(
//                     syn::punctuated::Punctuated::<T, Token![,]>::parse_terminated,
//                     value.tokens.clone(),
//                 )?;
//                 let collect = punctuated.into_iter().collect::<Vec<T>>();
//                 Ok(Self(collect))
//             }
//             Meta::NameValue(ref value) => Self::from_expr(&value.value),
//         })
//         .map_err(|e| e.with_span(item))
//     }
// }

const EXPECTED_IDENT: &str = "expected identifier";

macro_rules! parse_type {
    ($ty:ty) => {{
        let ty: ::syn::Type = ::syn::parse_quote! { $ty };
        ty
    }};
}
pub(crate) use parse_type;

pub fn from_str_to_rs_ident(s: &str) -> Ident {
    quote::format_ident!("{s}")
}

pub fn parse_curly_brackets(
    input: syn::parse::ParseStream,
) -> syn::Result<syn::parse::ParseBuffer> {
    Ok(syn::__private::parse_braces(input)?.content)
}

pub fn parse_exactly_one_punctuated<T, P>(punctuated: &Punctuated<T, P>) -> Option<&T> {
    match punctuated.first() {
        Some(ident) if punctuated.len() == 1 => Some(ident),
        _ => None,
    }
}

pub const fn parse_path_segments_from_type_path(
    path: &TypePath,
) -> Option<&Punctuated<PathSegment, Token![::]>> {
    if let TypePath {
        path: Path {
            segments,
            leading_colon: None,
        },
        qself: None,
    } = path
    {
        Some(segments)
    } else {
        None
    }
}
pub const fn parse_path_segments_from_type(
    ty: &Type,
) -> Option<&Punctuated<PathSegment, Token![::]>> {
    if let Type::Path(path) = ty {
        parse_path_segments_from_type_path(path)
    } else {
        None
    }
}

pub fn parse_ident_from_path_segments(
    path_segments: &Punctuated<PathSegment, Token![::]>,
) -> Result<&Ident, syn::Error> {
    if let Some(PathSegment {
        ident,
        arguments: syn::PathArguments::None,
    }) = parse_exactly_one_punctuated(path_segments)
    {
        Ok(ident)
    } else {
        Err(syn::Error::new(path_segments.span(), EXPECTED_IDENT))
    }
}
pub fn parse_ident_from_type(ty: &Type) -> Result<&Ident, syn::Error> {
    let path_segments = parse_path_segments_from_type(ty);
    path_segments.map_or_else(
        || Err(syn::Error::new(ty.span(), EXPECTED_IDENT)),
        parse_ident_from_path_segments,
    )
}

pub fn parse_type_single_arg_from_path_segments(
    path_segments: &Punctuated<PathSegment, Token![::]>,
) -> Option<(&Ident, &Type)> {
    let Some(PathSegment {
        ident,
        arguments: syn::PathArguments::AngleBracketed(args),
    }) = parse_exactly_one_punctuated(path_segments)
    else {
        return None;
    };
    let args = &args.args;
    let Some(syn::GenericArgument::Type(ty2)) = parse_exactly_one_punctuated(args) else {
        return None;
    };
    Some((ident, ty2))
}

pub fn unzip_token_streams(
    token_streams: impl Iterator<Item = (proc_macro2::TokenStream, proc_macro2::TokenStream)>,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let mut token_streams_a = proc_macro2::TokenStream::new();
    let mut token_streams_b = proc_macro2::TokenStream::new();
    for (token_stream_a, token_stream_b) in token_streams {
        token_streams_a.extend(token_stream_a);
        token_streams_b.extend(token_stream_b);
    }
    (token_streams_a, token_streams_b)
}
