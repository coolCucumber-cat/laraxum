mod kw {
    syn::custom_keyword! { Option }
}

use syn::{
    GenericArgument, Ident, Meta, Pat, Path, PathSegment, Token, Type, TypePath,
    parse::{Parse, ParseBuffer, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
};

/// Allow `syn::Pat` to be parsed by `syn::parse::Parse`
pub struct ParsePat(pub Pat);
impl Parse for ParsePat {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Pat::parse_multi(input).map(Self)
    }
}

pub struct TokenStreamAttr<T>(pub T)
where
    T: syn::parse::Parse;
impl<T> darling::FromMeta for TokenStreamAttr<T>
where
    T: syn::parse::Parse,
{
    fn from_meta(item: &Meta) -> darling::Result<Self> {
        (match *item {
            Meta::Path(_) => Self::from_word(),
            Meta::List(ref value) => {
                let parse2 = syn::parse2::<T>(value.tokens.to_owned())?;
                Ok(Self(parse2))
            }
            Meta::NameValue(ref value) => Self::from_expr(&value.value),
        })
        .map_err(|e| e.with_span(item))
    }
}

#[expect(dead_code)]
pub struct TokenStreamListAttr<T>(pub Vec<T>)
where
    T: syn::parse::Parse;
impl<T> darling::FromMeta for TokenStreamListAttr<T>
where
    T: syn::parse::Parse,
{
    fn from_meta(item: &Meta) -> darling::Result<Self> {
        (match *item {
            Meta::Path(_) => Self::from_word(),
            Meta::List(ref value) => {
                let punctuated = syn::parse::Parser::parse2(
                    syn::punctuated::Punctuated::<T, Token![,]>::parse_terminated,
                    value.tokens.to_owned(),
                )?;
                let collect = punctuated.into_iter().collect::<Vec<T>>();
                Ok(Self(collect))
            }
            Meta::NameValue(ref value) => Self::from_expr(&value.value),
        })
        .map_err(|e| e.with_span(item))
    }
}

/// Allow enum variants to be parsed in a list by `darling`
///
/// Darling only allows enum variants to be parsed in a list as the only element.
/// To bypass this, we put it in its own list with `core::slice::windows`.
/// The expected way to handle this would be to call `darling::FromMeta::from_nested_meta`,
/// but this doesn't work since it assumes you are calling it from one level higher.
pub struct EnumMetaListAttr<T>(pub Vec<T>)
where
    T: darling::FromMeta;
impl<T> darling::FromMeta for EnumMetaListAttr<T>
where
    T: darling::FromMeta,
{
    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        items
            .windows(1)
            .map(T::from_list)
            // .iter()
            // .map(T::from_nested_meta)
            .collect::<Result<Vec<T>, darling::Error>>()
            .map(Self)
    }
}
impl<T> Default for EnumMetaListAttr<T>
where
    T: darling::FromMeta,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

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

pub fn parse_curly_brackets(input: ParseStream) -> syn::Result<ParseBuffer> {
    Ok(syn::__private::parse_braces(input)?.content)
}

pub fn parse_exactly_one_punctuated<T, P>(punctuated: &Punctuated<T, P>) -> Option<&T> {
    match punctuated.first() {
        Some(ident) if punctuated.len() == 1 => Some(ident),
        _ => None,
    }
}

pub fn parse_path_segments_from_type_path(
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
pub fn parse_path_segments_from_type(ty: &Type) -> Option<&Punctuated<PathSegment, Token![::]>> {
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
    if let Some(path_segments) = path_segments {
        parse_ident_from_path_segments(path_segments)
    } else {
        Err(syn::Error::new(ty.span(), EXPECTED_IDENT))
    }
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
    let Some(GenericArgument::Type(ty2)) = parse_exactly_one_punctuated(args) else {
        return None;
    };
    Some((ident, ty2))
}
