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

#[expect(non_camel_case_types)]
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

#[derive(darling::FromMeta)]
pub struct ColumnAttrTyCompounds {
    #[darling(
        and_then = "crate::utils::syn::TokenStreamAttr::transform",
        rename = "model"
    )]
    pub model_rs_name: Ident,
    #[darling(
        and_then = "crate::utils::syn::TokenStreamAttr::transform_option",
        rename = "index"
    )]
    pub index_rs_ty: Option<Ident>,
}

#[derive(darling::FromMeta, Default)]
#[darling(default)]
pub struct ColumnAttrTyCompound {
    pub many: Option<ColumnAttrTyCompounds>,
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

#[derive(darling::FromMeta)]
#[darling(rename_all = "snake_case")]
pub enum ValidateRule {
    MinLen(crate::utils::syn::TokenStreamAttr<Expr>),
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

#[derive(darling::FromMeta, Default)]
#[darling(default)]
pub struct ColumnAttrRequest {
    pub name: Option<String>,
    #[darling(and_then = "crate::utils::syn::TokenStreamEnumAttrVec::transform")]
    pub validate: Vec<ValidateRule>,
}

#[derive(darling::FromMeta, Default, Clone, Copy)]
#[darling(rename_all = "snake_case")]
pub enum ColumnAttrIndexFilter {
    #[default]
    None,
    Eq,
    Like,
    Gt,
    Lt,
    Gte,
    Lte,
}
impl ColumnAttrIndexFilter {
    pub fn parameter(&self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Eq => Some("eq"),
            Self::Like => Some("like"),
            Self::Gt => Some("gt"),
            Self::Lt => Some("lt"),
            Self::Gte => Some("gte"),
            Self::Lte => Some("lte"),
        }
    }
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
    pub fn is_eq(&self) -> bool {
        matches!(self, Self::Eq)
    }
}

#[derive(darling::FromMeta, Default, Clone, Copy)]
#[darling(rename_all = "snake_case")]
pub enum ColumnAttrIndexLimit {
    #[default]
    None,
    Limit,
    Page {
        per_page: u64,
    },
}
impl ColumnAttrIndexLimit {
    pub fn parameter(&self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Limit => Some("limit"),
            Self::Page { .. } => Some("page"),
        }
    }
}

#[derive(darling::FromMeta)]
pub struct ColumnAttrIndex {
    #[darling(
        rename = "name",
        and_then = "crate::utils::syn::TokenStreamAttr::transform"
    )]
    pub rs_name: Ident,
    #[darling(default)]
    pub filter: ColumnAttrIndexFilter,
    #[darling(default)]
    pub sort: bool,
    #[darling(default)]
    pub limit: ColumnAttrIndexLimit,
    #[darling(default)]
    pub controller: bool,
}

#[derive(darling::FromAttributes, Default)]
#[darling(attributes(db), forward_attrs(doc, allow), default)]
pub struct ColumnAttr {
    pub name: Option<String>,
    pub ty: Option<ColumnAttrTy>,
    pub response: ColumnAttrResponse,
    pub request: ColumnAttrRequest,
    #[darling(
        and_then = "crate::utils::syn::TokenStreamAttr::transform_option",
        rename = "real_ty"
    )]
    pub real_rs_ty: Option<Box<Type>>,
    pub unique: bool,
    #[darling(and_then = "crate::utils::syn::TokenStreamAttrOption::transform_option")]
    pub borrow: Option<Option<Box<Type>>>,
    #[darling(multiple)]
    pub index: Vec<ColumnAttrIndex>,
    #[darling(and_then = "crate::utils::syn::TokenStreamAttr::transform_option")]
    pub struct_name: Option<Ident>,
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

#[derive(darling::FromMeta)]
pub struct TableAttrModel {
    #[darling(default)]
    pub many: bool,
}

#[derive(darling::FromMeta)]
pub struct TableAttrController {
    #[darling(and_then = "crate::utils::syn::TokenStreamAttr::transform_option")]
    pub auth: Option<Box<Type>>,
}

// #[cfg_attr(debug_assertions, derive(PartialEq, Eq, Debug))]
#[derive(darling::FromAttributes)]
#[darling(attributes(db), forward_attrs(allow, doc))]
pub struct TableAttr {
    pub model: Option<TableAttrModel>,
    pub controller: Option<TableAttrController>,
    pub name: Option<String>,
    #[darling(and_then = "crate::utils::syn::TokenStreamAttr::transform_option")]
    pub index_name: Option<Ident>,

    pub attrs: Vec<Attribute>,
    // TODO: this was removed for simplicity, add it back
    // pub collection: Option<darling::util::SpannedValue<()>>,
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
        let item = input.parse::<Item>()?;
        Self::try_from(item)
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
            content: tables,
            semi: _,
        }) = item
        else {
            return Err(syn::Error::new(item.span(), DB_ITEM_MUST_BE_MOD));
        };

        if !rs_attrs.is_empty() {
            return Err(syn::Error::new(item_span, DB_MOD_MUST_NOT_HAVE_ATTRS));
        }
        if let Some(rs_unsafety) = rs_unsafety {
            return Err(syn::Error::new(
                rs_unsafety.span(),
                DB_MOD_MUST_NOT_BE_UNSAFE,
            ));
        }

        let Some((_, tables)) = tables else {
            return Err(syn::Error::new(item_span, DB_MOD_MUST_HAVE_CONTENT));
        };
        let tables = tables.into_iter().map(Table::try_from);
        let tables: Result<Vec<Table>, syn::Error> = tables.collect();
        let tables = tables?;

        Ok(Self {
            rs_name,
            tables,
            rs_vis,
        })
    }
}
