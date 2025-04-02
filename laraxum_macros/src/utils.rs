mod kw {
    syn::custom_keyword! { Option }
}

use core::{borrow::Borrow, ops::Deref};
use std::borrow::Cow;

use syn::{
    GenericArgument, Ident, Path, PathSegment, Token, Type, TypePath,
    parse::{ParseBuffer, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
};

const EXPECTED_IDENT: &str = "expected identifier";

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

pub trait Push<T> {
    fn push(&mut self, value: T);
    fn new_and_push(value: T) -> Self;
}

impl<T> Push<T> for Vec<T> {
    fn push(&mut self, value: T) {
        self.push(value);
    }
    fn new_and_push(value: T) -> Self {
        vec![value]
    }
}

impl Push<syn::Error> for syn::Error {
    fn push(&mut self, error: Self) {
        self.combine(error);
    }
    fn new_and_push(value: Self) -> Self {
        value
    }
}

// impl<T, U> Push<T> for Option<U> where U:Push<T>{
//     fn push(&mut self, value: T) {
//         match self {
//             Some(u)=>u.push(value),
//             None=>
//         }
//     }
// }

pub trait TryCollectAll<T, CollectT, E, CollectE>: Iterator<Item = Result<T, E>> + Sized
where
    CollectT: Push<T>,
    CollectE: Push<E>,
{
    fn try_collect_all(mut self) -> Result<Option<CollectT>, CollectE> {
        let e = 'ok: {
            let mut collect_t = match self.next() {
                Some(Ok(t)) => CollectT::new_and_push(t),
                Some(Err(e)) => break 'ok e,
                None => return Ok(None),
            };
            for value in &mut self {
                match value {
                    Ok(t) => collect_t.push(t),
                    Err(e) => break 'ok e,
                }
            }
            return Ok(Some(collect_t));
        };
        let mut collect_e = CollectE::new_and_push(e);
        for value in self {
            if let Err(e) = value {
                collect_e.push(e);
            }
        }
        Err(collect_e)
    }
    fn try_collect_all_default(self) -> Result<CollectT, CollectE>
    where
        CollectT: Default,
    {
        self.try_collect_all().map(Option::unwrap_or_default)
    }
}
impl<I, T, CollectT, E, CollectE> TryCollectAll<T, CollectT, E, CollectE> for I
where
    I: Iterator<Item = Result<T, E>> + Sized,
    CollectT: Push<T> + Default,
    CollectE: Push<E>,
{
}

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

pub fn is_type_optional<'ty>(ty: Cow<'ty, Type>) -> (Cow<'ty, Type>, bool) {
    match ty {
        Cow::Owned(ty2) => match parse_option_from_ty(&ty2) {
            Some(ty3) => (Cow::Owned(ty3.to_owned()), true),
            None => (Cow::Owned(ty2), false),
        },
        Cow::Borrowed(ty2) => match parse_option_from_ty(ty2) {
            Some(ty3) => (Cow::Borrowed(ty3), true),
            None => (Cow::Borrowed(ty2), false),
        },
    }
}

pub fn cow_try_and_then<'t, T>(ty: Cow<'t, T>, f: F) -> (Cow<'t, T>, bool)
where
    F: for<'t1> FnOnce(&'t1 T) -> Result<&T>,
{
}
