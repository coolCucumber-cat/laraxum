use crate::utils::syn::{is_optional_type, parse_ident_from_type};

use darling::{FromAttributes, FromMeta};
use syn::{
    Attribute, Field, FieldMutability, Ident, Item, ItemMod, ItemStruct, Type, Visibility,
    parse::Parse, spanned::Spanned,
};

const DB_ITEM_MUST_BE_MOD: &str = "db item must be module";
const DB_MOD_MUST_HAVE_CONTENT: &str = "db mod must have content";
const DB_MOD_MUST_NOT_HAVE_ATTRS: &str = "db mod must not have attrs";
const DB_MOD_MUST_NOT_BE_UNSAFE: &str = "db mod must not be unsafe";
const TABLE_MUST_BE_STRUCT: &str = "item must be struct";
const TABLE_MUST_BE_FIELD_STRUCT: &str = "table must be field struct";
const GENERICS_ARE_NOT_ALLOWED: &str = "generics are not allowed";
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

        impl<'ty> ::core::convert::TryFrom::<&'ty ::syn::Type> for $enum {
            type Error = ::syn::Error;

            fn try_from(ty: &'ty ::syn::Type) -> ::core::result::Result::<Self, Self::Error> {
                $(
                    {
                        let ty_cmp: ::syn::Type = ::syn::parse_quote! { $ty };
                        if ty == &ty_cmp {
                            return ::core::result::Result::Ok(Self::$ident);
                        }
                    }
                )*
                let span = ::syn::spanned::Spanned::span(&ty);
                ::core::result::Result::Err(::syn::Error::new(span, UNKNOWN_TYPE))
            }
        }
    };
}

ty_enum! {
    #[allow(non_camel_case_types)]
    #[derive(PartialEq, Eq)]
    pub enum AtomicTy {
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
        ChronoDateTimeUtc => chrono::DateTime<chrono::Utc>,
        /// TIMESTAMP
        ChronoDateTimeLocal => chrono::DateTime<chrono::Local>,
        /// DATETIME
        ChronoNaiveDateTime => chrono::NaiveDateTime,
        /// DATE
        ChronoNaiveDate => chrono::NaiveDate,
        /// TIME
        ChronoNaiveTime => chrono::NaiveTime,
        /// TIME
        ChronoTimeDelta => chrono::TimeDelta,
    }
}

pub struct TyElementValue {
    pub ty: AtomicTy,
    pub optional: bool,
}
impl TryFrom<&Type> for TyElementValue {
    type Error = syn::Error;
    fn try_from(rs_ty: &Type) -> Result<Self, Self::Error> {
        let (rs_ty, optional) = is_optional_type(rs_ty);
        let ty = AtomicTy::try_from(rs_ty)?;
        Ok(Self { ty, optional })
    }
}

pub struct TyCompound {
    pub ty: Ident,
    pub optional: bool,
}
impl TryFrom<&Type> for TyCompound {
    type Error = syn::Error;
    fn try_from(rs_ty: &Type) -> Result<Self, Self::Error> {
        let (rs_ty, optional) = is_optional_type(rs_ty);
        let ty = parse_ident_from_type(rs_ty)?.clone();
        Ok(Self { ty, optional })
    }
}

#[derive(darling::FromMeta)]
// #[darling(default)]
pub enum ColumnAttrTy {
    #[darling(rename = "value")]
    Value(Type),
    #[darling(rename = "id")]
    Id,
    #[darling(rename = "foreign")]
    Foreign,
    #[darling(rename = "on_create")]
    OnCreate,
    #[darling(rename = "on_update")]
    OnUpdate,
    #[darling(rename = "varchar")]
    Varchar(StringLen),
    #[darling(rename = "char")]
    Char(StringLen),
    #[darling(rename = "text")]
    Text,
}

#[derive(darling::FromMeta, Default)]
#[darling(default)]
pub struct ColumnAttrResponse {
    pub name: Option<String>,
    pub ty: Option<Type>,
    #[darling(default)]
    pub skip: bool,
}

#[derive(darling::FromMeta, Default)]
#[darling(default)]
pub struct ColumnAttrRequest {
    pub name: Option<String>,
    pub ty: Option<Type>,
    #[darling(default)]
    pub skip: bool,
}

#[derive(darling::FromAttributes, Default)]
#[darling(attributes(db), forward_attrs(doc, allow), default)]
pub struct ColumnAttr {
    pub name: Option<String>,
    pub ty: Option<ColumnAttrTy>,
    #[darling(rename = "response")]
    pub attr_response: ColumnAttrResponse,
    #[darling(rename = "request")]
    pub attr_request: ColumnAttrRequest,

    pub attrs: Vec<Attribute>,
}

pub struct Column {
    pub rs_name: Ident,
    pub rs_ty: Type,
    pub attr: ColumnAttr,
}
impl TryFrom<Field> for Column {
    type Error = syn::Error;
    fn try_from(field: Field) -> Result<Self, Self::Error> {
        let field_span = field.span();
        let Field {
            attrs: rs_attrs,
            vis,
            mutability,
            ident: rs_name,
            colon_token: _,
            ty: rs_ty,
        } = field;

        let Some(rs_name) = rs_name else {
            return Err(syn::Error::new(field_span, TABLE_MUST_BE_FIELD_STRUCT));
        };
        if !matches!(vis, Visibility::Inherited) {
            return Err(syn::Error::new(vis.span(), FIELD_MUST_NOT_HAVE_VIS));
        }
        if !matches!(mutability, FieldMutability::None) {
            return Err(syn::Error::new(field_span, FIELD_MUST_NOT_BE_MUT));
        }

        let attr = ColumnAttr::from_attributes(&rs_attrs)?;
        Ok(Self {
            rs_name,
            rs_ty,
            attr,
        })
    }
}

#[cfg_attr(debug_assertions, derive(PartialEq, Eq, Debug))]
#[derive(darling::FromAttributes)]
#[darling(attributes(db), forward_attrs(allow, doc))]
pub struct TableAttr {
    #[darling(default)]
    pub model: bool,
    #[darling(default)]
    pub controller: bool,
    pub name: Option<String>,

    pub attrs: Vec<Attribute>,
}

pub struct Table {
    pub rs_name: Ident,
    pub columns: Vec<Column>,
    pub attr: TableAttr,
    pub rs_vis: Visibility,
}
impl TryFrom<Item> for Table {
    type Error = syn::Error;
    fn try_from(item: Item) -> Result<Self, Self::Error> {
        let Item::Struct(item_struct) = item else {
            return Err(syn::Error::new(item.span(), TABLE_MUST_BE_STRUCT));
        };
        let ItemStruct {
            attrs: rs_attrs,
            vis: rs_vis,
            struct_token: _,
            ident: rs_name,
            generics: rs_generics,
            fields: rs_columns,
            semi_token: _,
        } = item_struct;

        if !rs_generics.params.is_empty() {
            return Err(syn::Error::new(
                rs_generics.params.span(),
                GENERICS_ARE_NOT_ALLOWED,
            ));
        }
        if let Some(where_clause) = rs_generics.where_clause {
            return Err(syn::Error::new(
                where_clause.span(),
                GENERICS_ARE_NOT_ALLOWED,
            ));
        }

        let columns = rs_columns.into_iter().map(Column::try_from);
        let columns: Result<Vec<Column>, syn::Error> = columns.collect();
        let columns = columns?;

        let attr = TableAttr::from_attributes(&rs_attrs)?;

        Ok(Self {
            rs_name,
            columns,
            attr,
            rs_vis,
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
        let attr = Self::from_list(&metas)?;
        Ok(attr)
    }
}

pub struct Db {
    pub rs_name: Ident,
    pub tables: Vec<Table>,
    pub rs_vis: Visibility,
}
impl Parse for Db {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Item>().and_then(Db::try_from)
    }
}
impl TryFrom<Item> for Db {
    type Error = syn::Error;
    fn try_from(item: Item) -> Result<Self, Self::Error> {
        let item_span = item.span();
        let Item::Mod(ItemMod {
            attrs: rs_attrs,
            vis: rs_vis,
            unsafety: rs_unsafety,
            mod_token: _,
            ident: rs_name,
            content: rs_tables,
            semi: _,
        }) = item
        else {
            return Err(syn::Error::new(item.span(), DB_ITEM_MUST_BE_MOD));
        };

        if !rs_attrs.is_empty() {
            return Err(syn::Error::new(item_span, DB_MOD_MUST_NOT_HAVE_ATTRS));
        }
        if let Some(unsafety) = rs_unsafety {
            return Err(syn::Error::new(unsafety.span(), DB_MOD_MUST_NOT_BE_UNSAFE));
        };

        let Some((_, items)) = rs_tables else {
            return Err(syn::Error::new(item_span, DB_MOD_MUST_HAVE_CONTENT));
        };
        let tables = items.into_iter().map(Table::try_from);
        let tables: Result<Vec<Table>, syn::Error> = tables.collect();
        let tables = tables?;

        Ok(Self {
            rs_name,
            tables,
            rs_vis,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_attr() {
        let attr: Attribute = syn::parse_quote!(#[db(name = "groups")]);
        let table_attr = TableAttr::from_attributes(&[attr]).unwrap();
        assert_eq!(
            table_attr,
            TableAttr {
                attrs: vec![],
                controller: false,
                model: false,
                name: Some("groups".into())
            }
        );
    }
}
