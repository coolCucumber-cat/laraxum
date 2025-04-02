use crate::utils::{is_type_optional, parse_ident_from_ty};

use darling::{FromAttributes, FromMeta};
use syn::{
    Attribute, Expr, Field, FieldMutability, Ident, Item, ItemMod, ItemStruct, Type, Visibility,
    parse::Parse, spanned::Spanned,
};

const DB_ITEM_MUST_BE_MOD: &str = "db item must be module";
const DB_MOD_MUST_HAVE_CONTENT: &str = "db mod must have content";
const DB_MOD_MUST_NOT_HAVE_ATTRS: &str = "db mod must not have attrs";
const DB_MOD_MUST_NOT_BE_UNSAFE: &str = "db mod must not be unsafe";
const TABLE_MUST_BE_STRUCT: &str = "item must be struct";
const TABLE_MUST_BE_FIELD_STRUCT: &str = "table must be field struct";
const FIELD_MUST_NOT_HAVE_VIS: &str = "field must not have visibility";
const FIELD_MUST_NOT_BE_MUT: &str = "field must not be mutable";
const UNKNOWN_TYPE: &str = "unknown type";

pub type StringLen = u16;

macro_rules! ty_enum {
    {
        $(#[$meta:meta])*
        $vis:vis enum $enum:ident {
            $(
                $(#[$variant_meta:meta])*
                $ident:ident => $ty:ty
            ),* $(,)?
        }
        // ;
        // $(#[$mod_meta:meta])*
        // $mod_vis:vis mod $mod_ident:ident;
    } => {
        $(#[$meta])*
        $vis enum $enum {
            $(
                $(#[$variant_meta])*
                $ident,
            )*
        }

        impl ::core::convert::TryFrom::<::syn::Type> for $enum {
            type Error = ::syn::Error;

            fn try_from(ty: ::syn::Type) -> ::core::result::Result::<Self, Self::Error> {
                $(
                    {
                        let ty_cmp: ::syn::Type = ::syn::parse_quote! { $ty };
                        if ty == ty_cmp {
                            return ::core::result::Result::Ok(Self::$ident);
                        }
                    }
                )*
                let span = ::syn::spanned::Spanned::span(&ty);
                ::core::result::Result::Err(::syn::Error::new(span, UNKNOWN_TYPE))
            }
        }

        // $(#[$mod_meta])*
        // $mod_vis mod $mod_ident {
        //     $(
        //         $(#[$variant_meta])*
        //         struct $ident;
        //
        //     )*
        // }
    };
}

ty_enum! {
    #[allow(non_camel_case_types)]
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum ScalarTy {
        String => String,
        bool => bool,
        u8 => u8,
        i8 => i8,
        u16 => u16,
        i16 => i16,
        u32 => u32,
        i32 => i32,
        u64 => u64,
        i64 => i64,
        f32 => f32,
        f64 => f64,

        /// TIMESTAMP
        TimeOffsetDateTime => time::OffsetDateTime,
        /// DATETIME
        TimeDateTime => time::PrimitiveDateTime,
        /// DATE
        TimeDate => time::Date,
        /// TIME
        TimeTime => time::Time,
        /// TIME
        TimeDuration => time::Duration,

        /// TIMESTAMP
        ChronoDateTimeUtc => chrono::DateTime<chrono::offset::Utc>,
        /// TIMESTAMP
        ChronoDateTimeLocal => chrono::DateTime<chrono::offset::Local>,
        /// DATETIME
        ChronoNaiveDateTime => chrono::NaiveDateTime,
        /// DATE
        ChronoNaiveDate => chrono::NaiveDate,
        /// TIME
        ChronoNaiveTime => chrono::NaiveTime,
        /// TIME
        ChronoTimeDelta => chrono::TimeDelta,
    }
    // ;
    // pub mod scalar_ty;
}

#[derive(Clone)]
pub struct RealTy {
    pub ty: ScalarTy,
    pub optional: bool,
}

impl TryFrom<Type> for RealTy {
    type Error = syn::Error;
    fn try_from(input: Type) -> Result<Self, Self::Error> {
        let (ty, optional) = is_type_optional(input);
        let ty = ScalarTy::try_from(ty)?;
        Ok(Self { ty, optional })
    }
}

impl TryFrom<&Type> for RealTy {
    type Error = syn::Error;
    fn try_from(input: &Type) -> Result<Self, Self::Error> {
        // let (ty, optional) = is_type_optional(input);
        // let ty = ScalarTy::try_from(ty)?;
        let ty = ScalarTy::bool;
        let optional = true;
        Ok(Self { ty, optional })
    }
}

#[derive(Clone)]
pub struct ForeignTy {
    pub ty: Ident,
    pub optional: bool,
}

impl TryFrom<Type> for ForeignTy {
    type Error = syn::Error;
    fn try_from(input: Type) -> Result<Self, Self::Error> {
        let (ty, optional) = is_type_optional(input);
        let ty = parse_ident_from_ty(&ty)?.clone();
        Ok(Self { ty, optional })
    }
}

#[derive(darling::FromAttributes, Default)]
#[darling(attributes(db), forward_attrs(cfg, doc, allow), default)]
pub struct ColumnAttrs {
    // type
    pub id: bool,
    pub on_update: bool,
    pub on_create: bool,
    pub foreign: bool,
    pub varchar: Option<StringLen>,
    pub char: Option<StringLen>,
    pub text: bool,
    // name
    pub name: Option<Ident>,
    // response
    pub response: Option<Expr>,

    // forwarded attrs
    pub attrs: Vec<Attribute>,
}

pub struct Column {
    pub ident: Ident,
    pub ty: Type,
    pub attrs: ColumnAttrs,
}

impl TryFrom<Field> for Column {
    type Error = syn::Error;
    fn try_from(field: Field) -> Result<Self, Self::Error> {
        let span = field.span();
        let Field {
            attrs,
            vis,
            mutability,
            ident,
            colon_token: _,
            ty,
        } = field;

        let Some(ident) = ident else {
            return Err(syn::Error::new(span, TABLE_MUST_BE_FIELD_STRUCT));
        };

        if !matches!(vis, Visibility::Inherited) {
            return Err(syn::Error::new(vis.span(), FIELD_MUST_NOT_HAVE_VIS));
        }

        if !matches!(mutability, FieldMutability::None) {
            return Err(syn::Error::new(span, FIELD_MUST_NOT_BE_MUT));
        }

        let attrs = ColumnAttrs::from_attributes(&attrs)?;

        Ok(Self { ident, ty, attrs })
    }
}

#[cfg_attr(debug_assertions, derive(PartialEq, Eq, Debug))]
#[derive(darling::FromAttributes)]
#[darling(attributes(db), forward_attrs(allow, doc, cfg))]
pub struct TableAttrs {
    #[darling(default, rename = "auto")]
    pub auto_impl_controller: bool,
    pub name: Option<String>,

    pub attrs: Vec<Attribute>,
}

pub struct Table {
    pub ident: Ident,
    pub columns: Vec<Column>,
    pub attrs: TableAttrs,
    pub vis: Visibility,
}

impl TryFrom<Item> for Table {
    type Error = syn::Error;
    fn try_from(item: Item) -> Result<Self, Self::Error> {
        let Item::Struct(item_struct) = item else {
            return Err(syn::Error::new(item.span(), TABLE_MUST_BE_STRUCT));
        };
        let ItemStruct {
            attrs,
            vis,
            struct_token: _,
            ident,
            generics: _,
            fields,
            semi_token: _,
        } = item_struct;

        let attrs = TableAttrs::from_attributes(&attrs)?;

        // TODO: check theres no generics

        let columns = fields.into_iter().map(Column::try_from);
        let columns: Result<Vec<Column>, syn::Error> = columns.collect();
        let columns = columns?;

        Ok(Self {
            ident,
            columns,
            attrs,
            vis,
        })
    }
}

#[derive(darling::FromMeta)]
pub struct DbAttr {
    pub name: Option<String>,
}

impl TryFrom<proc_macro2::TokenStream> for DbAttr {
    type Error = syn::Error;
    fn try_from(input: proc_macro2::TokenStream) -> Result<Self, Self::Error> {
        let metas = darling::ast::NestedMeta::parse_meta_list(input)?;
        let db_attr = Self::from_list(&metas)?;
        Ok(db_attr)
    }
}

pub struct Db {
    pub ident: Ident,
    pub tables: Vec<Table>,
    pub vis: Visibility,
}

impl TryFrom<Item> for Db {
    type Error = syn::Error;
    fn try_from(item: Item) -> Result<Self, Self::Error> {
        let span = item.span();
        let Item::Mod(ItemMod {
            attrs,
            vis,
            unsafety,
            mod_token: _,
            ident,
            content,
            semi: _,
        }) = item
        else {
            return Err(syn::Error::new(item.span(), DB_ITEM_MUST_BE_MOD));
        };

        let Some((_, items)) = content else {
            return Err(syn::Error::new(span, DB_MOD_MUST_HAVE_CONTENT));
        };
        let tables = items.into_iter().map(Table::try_from);
        let tables: Result<Vec<Table>, syn::Error> = tables.collect();
        let tables = tables?;

        if !attrs.is_empty() {
            return Err(syn::Error::new(span, DB_MOD_MUST_NOT_HAVE_ATTRS));
        }

        if let Some(unsafety) = unsafety {
            return Err(syn::Error::new(unsafety.span(), DB_MOD_MUST_NOT_BE_UNSAFE));
        };

        Ok(Self { ident, tables, vis })
    }
}

impl Parse for Db {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Item>().and_then(Db::try_from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use syn::Attribute;

    #[test]
    fn table_attr() {
        let attr: Attribute = syn::parse_quote!(#[db(name = "groups")]);
        let table_attr = TableAttrs::from_attributes(&[attr]).unwrap();
        assert_eq!(
            table_attr,
            TableAttrs {
                attrs: vec![],
                auto_impl_controller: false,
                name: Some("groups".into())
            }
        );
    }

    // #[test]
    // fn db_attr() {
    //     let attr: Attribute = syn::parse_quote!(#[name = "dossier"]);
    //     let meta = attr.meta;
    //     let db_attr: DbAttr = DbAttr::from_meta(&meta).unwrap();
    //     // let name = db_attr.name.as_deref();
    //     assert_eq!(db_attr.name.as_deref(), Some("dossier"));
    // }
}
