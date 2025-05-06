mod kw {
    syn::custom_keyword! { Option }
}

use syn::{
    GenericArgument, Ident, Path, PathSegment, Token, Type, TypePath,
    parse::{ParseBuffer, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
};

const EXPECTED_IDENT: &str = "expected identifier";

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

pub fn parse_option_from_ty(ty: &Type) -> Option<&Type> {
    let path_segments = parse_path_segments_from_type(ty)?;
    let ty = parse_type_single_arg_from_path_segments(path_segments);
    ty.and_then(|(ident, ty)| (ident == "Option").then_some(ty))
}
pub fn is_optional_type(ty: &Type) -> (&Type, bool) {
    super::map_is_some(ty, parse_option_from_ty)
}
