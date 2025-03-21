mod kw {
    syn::custom_keyword! { Option }
}

use syn::{
    GenericArgument, Ident, Path, PathSegment, Token, Type, TypePath,
    parse::{ParseBuffer, ParseStream},
    punctuated::Punctuated,
};

macro_rules! helper_attribute_macro {
    ($parent_macro:ident => $self_macro:ident => $self_ty:ty => $input:expr) => {
        ::proc_macro::TokenStream::from(::syn::Error::into_compile_error(::syn::Error::new(
            match ::syn::parse::<$self_ty>($input) {
                ::core::result::Result::Ok(item) => ::syn::spanned::Spanned::span(&item),
                ::core::result::Result::Err(err) => ::syn::Error::span(&err),
            },
            ::core::concat!(
                "used helper attribute macro ",
                "`",
                ::core::stringify!($self_macro),
                "`",
                " outside of parent attribute macro ",
                "`",
                ::core::stringify!($parent_macro),
                "`",
                // "\n\n",
                // "it should be used on an item of the type ",
                // "`",
                // ::core::stringify!($self_ty),
                // "`"
            ),
        )))
    };
}
pub(crate) use helper_attribute_macro;

pub fn parse_curly_brackets(input: ParseStream) -> syn::Result<ParseBuffer> {
    Ok(syn::__private::parse_braces(input)?.content)
}
// pub fn parse_square_brackets(input: ParseStream) -> syn::Result<ParseBuffer> {
//     Ok(syn::__private::parse_brackets(input)?.content)
// }
// pub fn parse_round_brackets(input: ParseStream) -> syn::Result<ParseBuffer> {
//     Ok(syn::__private::parse_parens(input)?.content)
// }

fn parse_exactly_one_punctuated<T, P>(punctuated: &Punctuated<T, P>) -> Option<&T> {
    match punctuated.first() {
        Some(ident) if punctuated.len() == 1 => Some(ident),
        _ => None,
    }
}

fn parse_path_segments_from_type_path(
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
) -> Option<&Ident> {
    if let Some(PathSegment {
        ident,
        arguments: syn::PathArguments::None,
    }) = parse_exactly_one_punctuated(path_segments)
    {
        Some(ident)
    } else {
        None
    }
}
pub fn parse_ident_from_ty(ty: &Type) -> Option<&Ident> {
    if let Type::Path(path) = ty {
        let path_segments = parse_path_segments_from_type_path(path)?;
        parse_ident_from_path_segments(path_segments)
    } else {
        None
    }
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
    if let Type::Path(path) = ty {
        let path_segments = parse_path_segments_from_type_path(path)?;
        parse_option_from_path_segments(path_segments)
    } else {
        None
    }
}

pub fn is_type_optional(ty: Type) -> (Type, bool) {
    match parse_option_from_ty(&ty) {
        Some(ty2) => (ty2.clone(), true),
        None => (ty, false),
    }
}
