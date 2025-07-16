use crate::utils::{multiplicity, syn::parse_type};

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
const GENERICS_ARE_NOT_ALLOWED: &str = "generics are not allowed";
const FIELD_MUST_NOT_HAVE_VIS: &str = "field must not have visibility";
const FIELD_MUST_NOT_BE_MUT: &str = "field must not be mutable";
const UNKNOWN_TYPE: &str = "unknown type";

pub type StringLen = u16;

#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq)]
pub enum AtomicTy {
    String,
    bool,
    u8,
    i8,
    u16,
    i16,
    u32,
    i32,
    u64,
    i64,
    f32,
    f64,

    /// TIMESTAMP
    TimeOffsetDateTime,
    /// DATETIME
    TimeDateTime,
    /// DATE
    TimeDate,
    /// TIME
    TimeTime,
    /// TIME
    TimeDuration,

    /// TIMESTAMP
    ChronoDateTimeUtc,
    /// TIMESTAMP
    ChronoDateTimeLocal,
    /// DATETIME
    ChronoNaiveDateTime,
    /// DATE
    ChronoNaiveDate,
    /// TIME
    ChronoNaiveTime,
    /// TIME
    ChronoTimeDelta,
}
impl TryFrom<&Type> for AtomicTy {
    type Error = syn::Error;
    fn try_from(ty: &Type) -> Result<Self, Self::Error> {
        if ty == &parse_type!(String) {
            Ok(Self::String)
        } else if ty == &parse_type!(bool) {
            Ok(Self::bool)
        } else if ty == &parse_type!(u8) {
            Ok(Self::u8)
        } else if ty == &parse_type!(i8) {
            Ok(Self::i8)
        } else if ty == &parse_type!(u16) {
            Ok(Self::u16)
        } else if ty == &parse_type!(i16) {
            Ok(Self::i16)
        } else if ty == &parse_type!(u32) {
            Ok(Self::u32)
        } else if ty == &parse_type!(i32) {
            Ok(Self::i32)
        } else if ty == &parse_type!(u64) {
            Ok(Self::u64)
        } else if ty == &parse_type!(i64) {
            Ok(Self::i64)
        } else if ty == &parse_type!(f32) {
            Ok(Self::f32)
        } else if ty == &parse_type!(f64) {
            Ok(Self::f64)
        } else if ty == &parse_type!(time::OffsetDateTime) {
            Ok(Self::TimeOffsetDateTime)
        } else if ty == &parse_type!(time::PrimitiveDateTime) {
            Ok(Self::TimeDateTime)
        } else if ty == &parse_type!(time::Date) {
            Ok(Self::TimeDate)
        } else if ty == &parse_type!(time::Time) {
            Ok(Self::TimeTime)
        } else if ty == &parse_type!(time::Duration) {
            Ok(Self::TimeDuration)
        } else if ty == &parse_type!(chrono::DateTime<chrono::Utc>) {
            Ok(Self::ChronoDateTimeUtc)
        } else if ty == &parse_type!(chrono::DateTime<chrono::Local>) {
            Ok(Self::ChronoDateTimeLocal)
        } else if ty == &parse_type!(chrono::NaiveDateTime) {
            Ok(Self::ChronoNaiveDateTime)
        } else if ty == &parse_type!(chrono::NaiveDate) {
            Ok(Self::ChronoNaiveDate)
        } else if ty == &parse_type!(chrono::NaiveTime) {
            Ok(Self::ChronoNaiveTime)
        } else if ty == &parse_type!(chrono::TimeDelta) {
            Ok(Self::ChronoTimeDelta)
        } else {
            Err(syn::Error::new(ty.span(), UNKNOWN_TYPE))
        }
    }
}

pub struct TyElementValue {
    pub ty: AtomicTy,
    pub optional: bool,
}
impl TryFrom<&Type> for TyElementValue {
    type Error = syn::Error;
    fn try_from(rs_ty: &Type) -> Result<Self, Self::Error> {
        let (rs_ty, optional) = multiplicity::optional(rs_ty);
        let ty = AtomicTy::try_from(rs_ty)?;
        Ok(Self { ty, optional })
    }
}

pub struct TyCompound {
    pub ty: Ident,
    pub multiplicity: multiplicity::Multiplicity,
}
impl TryFrom<&Type> for TyCompound {
    type Error = syn::Error;
    fn try_from(rs_ty: &Type) -> Result<Self, Self::Error> {
        let (rs_ty, multiplicity) = multiplicity::multiplicity(rs_ty);
        let ty = crate::utils::syn::parse_ident_from_type(rs_ty)?.clone();
        Ok(Self { ty, multiplicity })
    }
}

#[derive(darling::FromMeta, Default)]
#[darling(default)]
pub struct ColumnAttrTyCompound {
    pub many: Option<crate::utils::syn::TokenStreamAttr<Ident>>,
}

#[derive(darling::FromMeta)]
#[darling(rename_all = "snake_case")]
pub enum ColumnAttrTy {
    #[darling(rename = "foreign")]
    Compound(ColumnAttrTyCompound),

    Id,

    Varchar(StringLen),
    Char(StringLen),
    Text,

    OnCreate,
    OnUpdate,
}

#[derive(darling::FromMeta, Default)]
#[darling(default)]
pub struct ColumnAttrResponse {
    pub name: Option<String>,
    #[darling(default)]
    pub skip: bool,
}

// use `TokenStreamAttr` because it can be parsed by `darling`
#[derive(darling::FromMeta)]
#[darling(rename_all = "snake_case")]
pub enum ValidateRule {
    MinLen(u16),
    Func(crate::utils::syn::TokenStreamAttr<Expr>),
    Matches(crate::utils::syn::TokenStreamAttr<crate::utils::syn::ParsePat>),
    NMatches(crate::utils::syn::TokenStreamAttr<crate::utils::syn::ParsePat>),
    Eq(crate::utils::syn::TokenStreamAttr<Expr>),
    NEq(crate::utils::syn::TokenStreamAttr<Expr>),
    Gt(crate::utils::syn::TokenStreamAttr<Expr>),
    Lt(crate::utils::syn::TokenStreamAttr<Expr>),
    Gte(crate::utils::syn::TokenStreamAttr<Expr>),
    Lte(crate::utils::syn::TokenStreamAttr<Expr>),
}

// use `EnumMetaListAttr` because it can be parsed by `darling`
#[derive(darling::FromMeta, Default)]
#[darling(default)]
pub struct ColumnAttrRequest {
    pub name: Option<String>,
    pub validate: crate::utils::syn::EnumMetaListAttr<ValidateRule>,
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
    pub real_ty: Option<Box<crate::utils::syn::TokenStreamAttr<Type>>>,

    pub attrs: Vec<Attribute>,
}

pub struct Column {
    pub rs_name: Ident,
    pub rs_ty: Box<Type>,
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

        let attr = <ColumnAttr as darling::FromAttributes>::from_attributes(&rs_attrs)?;

        Ok(Self {
            rs_name,
            rs_ty: Box::new(rs_ty),
            attr,
        })
    }
}

// #[cfg_attr(debug_assertions, derive(PartialEq, Eq, Debug))]
#[derive(darling::FromAttributes)]
#[darling(attributes(db), forward_attrs(allow, doc))]
pub struct TableAttr {
    // TODO: this was removed for simplicity, add it back
    // pub collection: Option<darling::util::SpannedValue<()>>,
    pub model: Option<darling::util::SpannedValue<()>>,
    pub controller: Option<darling::util::SpannedValue<()>>,
    pub many_model: Option<darling::util::SpannedValue<()>>,
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

        let attr = <TableAttr as darling::FromAttributes>::from_attributes(&rs_attrs)?;

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
        let attr = <Self as darling::FromMeta>::from_list(&metas)?;
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

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn table_attr() {
//         let attr: Attribute = syn::parse_quote!(#[db(name = "groups")]);
//         let table_attr = <TableAttr as darling::FromAttributes>::from_attributes(&[attr]).unwrap();
//         assert_eq!(
//             table_attr,
//             TableAttr {
//                 attrs: vec![],
//                 collection: None,
//                 controller: None,
//                 model: None,
//                 many_model: None,
//                 name: Some("groups".into())
//             }
//         );
//     }
// }
