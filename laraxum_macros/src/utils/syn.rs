use super::cow_try_and_then_is_some;

use std::borrow::Cow;

use syn::GenericArgument;

use syn::Type;

use super::EXPECTED_IDENT;

use syn::Ident;

use syn::Path;

use syn::PathSegment;

use syn::TypePath;

use syn::parse::ParseBuffer;

use syn::parse::ParseStream;

pub fn parse_curly_brackets(input: ParseStream) -> syn::Result<ParseBuffer> {
    Ok(syn::__private::parse_braces(input)?.content)
}

pub(crate) fn parse_exactly_one_punctuated<T, P>(punctuated: &Punctuated<T, P>) -> Option<&T> {
    match punctuated.first() {
        Some(ident) if punctuated.len() == 1 => Some(ident),
        _ => None,
    }
}

pub(crate) fn parse_path_segments_from_type_path(
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

pub fn parse_ident_from_ty(ty: &Type) -> Result<&Ident, syn::Error> {
    let path_segments = if let Type::Path(path) = ty {
        parse_path_segments_from_type_path(path)
    } else {
        None
    };
    if let Some(path_segments) = path_segments {
        parse_ident_from_path_segments(path_segments)
    } else {
        Err(syn::Error::new(ty.span(), EXPECTED_IDENT))
    }
    // if let Type::Path(path) = ty {
    //     if let Some(path_segments) = parse_path_segments_from_type_path(path) {
    //         parse_ident_from_path_segments(path_segments)
    //     } else {
    //         syn::Error::new(ty.span(), EXPECTED_IDENT)
    //     }
    // } else {
    //     syn::Error::new(ty.span(), EXPECTED_IDENT)
    // }
}

pub fn parse_option_from_path_segments(
    path_segments: &Punctuated<PathSegment, Token![::]>,
) -> Option<&Type> {
    let Some(PathSegment {
        ident,
        arguments: syn::PathArguments::AngleBracketed(args),
    }) = parse_exactly_one_punctuated(path_segments)
    else {
        return None;
    };
    if ident != "Option" {
        return None;
    }
    let args = &args.args;
    let Some(GenericArgument::Type(ty2)) = parse_exactly_one_punctuated(args) else {
        return None;
    };
    Some(ty2)
}

pub fn parse_option_from_ty(ty: &Type) -> Option<&Type> {
    let Type::Path(path) = ty else {
        return None;
    };
    let path_segments = parse_path_segments_from_type_path(path)?;
    parse_option_from_path_segments(path_segments)
}

pub fn is_type_optional(ty: &Type) -> (&Type, bool) {
    match parse_option_from_ty(ty) {
        Some(ty) => {}
        None => {}
    }
}

pub fn is_type_optional_cow<'ty>(ty: Cow<'ty, Type>) -> (Cow<'ty, Type>, bool) {
    cow_try_and_then_is_some(ty, |ty| parse_option_from_ty(ty).map(Cow::Borrowed))
}
