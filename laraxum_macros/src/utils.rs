use syn::{punctuated::Punctuated, Ident, Path, PathSegment, Type, TypePath, TypeReference};

pub fn try_from_type_to_sql_type(t: &Type) -> Option<&str> {
    if let Type::Path(path) = t {
        try_from_path_to_ident(path).and_then(|ident| {
            let ident = &*ident.to_string();
            match ident {
                "String" => Some("VARCHAR(255)"),
                "bool" => Some("BOOL"),
                "u8" => Some("UNSIGNED TINYINT"),
                "i8" => Some("TINYINT"),
                "u32" => Some("UNSIGNED INT"),
                "i32" => Some("INT"),
                "u64" => Some("UNSIGNED BIGINT"),
                "i64" => Some("BIGINT"),
                "f32" => Some("FLOAT"),
                "f64" => Some("DOUBLE"),
                _ => None,
            }
        })
    } else {
        None
    }
}

pub fn maybe_reference(input: Type) -> (Type, bool) {
    match input {
        Type::Reference(TypeReference {
            elem,
            lifetime: None,
            mutability: None,
            ..
        }) => (*elem, true),
        value_type => (value_type, false),
    }
}

pub fn maybe_optional(input: &Type) -> (&Type, bool) {
    // check if type is path
    if let Type::Path(TypePath {
        path: Path {
            segments,
            leading_colon: None,
        },
        qself: None,
    }) = input
    {
        // check if path has exactly one segment
        match exactly_one_punctuated(segments) {
            // check if is Option
            Some(PathSegment { ident, arguments }) if ident == "Option" => {
                // check if Option has angle brackets
                let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    args,
                    ..
                }) = arguments
                else {
                    panic!("Option must have angle brackets");
                };
                // check if Option has exactly one type argument
                let Some(syn::GenericArgument::Type(arg)) = exactly_one_punctuated(args) else {
                    panic!("Option must have exactly one type argument");
                };
                (arg, true)
            }
            _ => (input, false),
        }
    } else {
        (input, false)
    }
}

pub fn exactly_one_punctuated<T, P>(punctuated: &Punctuated<T, P>) -> Option<&T> {
    match punctuated.first() {
        Some(ident) if punctuated.len() == 1 => Some(ident),
        _ => None,
    }
}

pub fn try_from_path_to_ident(path: &TypePath) -> Option<&Ident> {
    // check if is simple path
    if let TypePath {
        path: Path {
            segments,
            leading_colon: None,
        },
        qself: None,
    } = path
    {
        // check if path has exactly one segment and no args
        match exactly_one_punctuated(segments) {
            Some(PathSegment {
                ident,
                arguments: syn::PathArguments::None,
            }) => Some(ident),
            _ => None,
        }
    } else {
        None
    }
}
