use syn::Type;

pub enum Multiplicity {
    One,
    OneOrZero,
    Many,
}
impl Multiplicity {
    pub const fn is_optional(&self) -> bool {
        match self {
            Self::OneOrZero => true,
            Self::One | Self::Many => false,
        }
    }
}

pub fn multiplicity(ty: &Type) -> (&Type, Multiplicity) {
    super::syn::parse_path_segments_from_type(ty)
        .and_then(super::syn::parse_type_single_arg_from_path_segments)
        .and_then(|(ident, ty)| {
            if ident == "Vec" {
                Some((ty, Multiplicity::Many))
            } else if ident == "Option" {
                Some((ty, Multiplicity::OneOrZero))
            } else {
                None
            }
        })
        .unwrap_or((ty, Multiplicity::One))
}

pub fn is_optional(ty: &Type) -> (&Type, bool) {
    let (ty_inner, multiplicity) = multiplicity(ty);
    let is_optional = multiplicity.is_optional();
    (if is_optional { ty_inner } else { ty }, is_optional)
}
