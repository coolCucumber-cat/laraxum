use super::stage1;
pub use super::stage1::{ColumnAttrRequest, ColumnAttrResponse, TyCompound};

use crate::utils::collections::TryCollectAll;

use syn::{Attribute, Ident, Type, Visibility, ext::IdentExt, spanned::Spanned};

const TABLE_MUST_HAVE_ID: &str = "table must have an ID";
const TABLE_MUST_NOT_HAVE_MULTIPLE_IDS: &str = "table must not have multiple IDs";
const TABLE_MUST_IMPLEMENT_MODEL: &str = "table must implement model to implement controller";
const ID_MUST_BE_U64: &str = "id must be u64";
const COLUMN_MUST_BE_STRING: &str = "column must be string";
const COLUMN_MUST_BE_TIME: &str = "column must be time";
const COLUMN_MUST_NOT_BE_OPTIONAL: &str = "column must not be optional";

pub enum AtomicTyString {
    Varchar(stage1::StringLen),
    Char(stage1::StringLen),
    Text,
}

pub enum AtomicTyTime {
    ChronoDateTimeUtc,
    ChronoDateTimeLocal,
    ChronoNaiveDateTime,
    ChronoNaiveDate,
    ChronoNaiveTime,
    ChronoTimeDelta,

    TimeOffsetDateTime,
    TimePrimitiveDateTime,
    TimeDate,
    TimeTime,
    TimeDuration,
}

#[allow(non_camel_case_types)]
pub enum AtomicTy {
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
    String(AtomicTyString),
    Time(AtomicTyTime),
}
impl From<stage1::AtomicTy> for AtomicTy {
    fn from(atomic_ty: stage1::AtomicTy) -> Self {
        match atomic_ty {
            stage1::AtomicTy::bool => Self::bool,
            stage1::AtomicTy::u8 => Self::u8,
            stage1::AtomicTy::i8 => Self::i8,
            stage1::AtomicTy::u16 => Self::u16,
            stage1::AtomicTy::i16 => Self::i16,
            stage1::AtomicTy::u32 => Self::u32,
            stage1::AtomicTy::i32 => Self::i32,
            stage1::AtomicTy::u64 => Self::u64,
            stage1::AtomicTy::i64 => Self::i64,
            stage1::AtomicTy::f32 => Self::f32,
            stage1::AtomicTy::f64 => Self::f64,

            stage1::AtomicTy::String => Self::String(AtomicTyString::Varchar(255)),

            stage1::AtomicTy::ChronoDateTimeUtc => Self::Time(AtomicTyTime::ChronoDateTimeUtc),
            stage1::AtomicTy::ChronoDateTimeLocal => Self::Time(AtomicTyTime::ChronoDateTimeLocal),
            stage1::AtomicTy::ChronoNaiveDateTime => Self::Time(AtomicTyTime::ChronoNaiveDateTime),
            stage1::AtomicTy::ChronoNaiveDate => Self::Time(AtomicTyTime::ChronoNaiveDate),
            stage1::AtomicTy::ChronoNaiveTime => Self::Time(AtomicTyTime::ChronoNaiveTime),
            stage1::AtomicTy::ChronoTimeDelta => Self::Time(AtomicTyTime::ChronoTimeDelta),

            stage1::AtomicTy::TimeDateTime => Self::Time(AtomicTyTime::TimePrimitiveDateTime),
            stage1::AtomicTy::TimeOffsetDateTime => Self::Time(AtomicTyTime::TimeOffsetDateTime),
            stage1::AtomicTy::TimeDate => Self::Time(AtomicTyTime::TimeDate),
            stage1::AtomicTy::TimeTime => Self::Time(AtomicTyTime::TimeTime),
            stage1::AtomicTy::TimeDuration => Self::Time(AtomicTyTime::TimeDuration),
        }
    }
}

pub struct TyElementValue {
    pub ty: AtomicTy,
    pub optional: bool,
}
impl From<stage1::TyElementValue> for TyElementValue {
    fn from(ty_element_value: stage1::TyElementValue) -> Self {
        Self {
            ty: AtomicTy::from(ty_element_value.ty),
            optional: ty_element_value.optional,
        }
    }
}

pub enum AutoTimeEvent {
    OnCreate,
    OnUpdate,
}

pub struct TyElementAutoTime {
    pub ty: AtomicTyTime,
    pub event: AutoTimeEvent,
}

enum ColumnAttrTyElement {
    None,
    Value(Type),
    Id,
    String(AtomicTyString),
    AutoTime(AutoTimeEvent),
}

enum ColumnAttrTyCompound {
    One,
    Many(Ident),
}

enum ColumnAttrTy {
    Compound(ColumnAttrTyCompound),
    Element(ColumnAttrTyElement),
}
impl From<Option<stage1::ColumnAttrTy>> for ColumnAttrTy {
    fn from(attr_ty: Option<stage1::ColumnAttrTy>) -> Self {
        use ColumnAttrTyCompound as CATC;
        use ColumnAttrTyElement as CATE;
        use stage1::{ColumnAttrTy as S1CAT, ColumnAttrTyCompound as S1CATF};
        match attr_ty {
            Some(S1CAT::Compound(S1CATF { many: None })) => Self::Compound(CATC::One),
            Some(S1CAT::Compound(S1CATF { many: Some(many) })) => Self::Compound(CATC::Many(many)),

            None => Self::Element(CATE::None),
            Some(S1CAT::Value(rs_ty)) => Self::Element(CATE::Value(rs_ty)),
            Some(S1CAT::Id) => Self::Element(CATE::Id),

            Some(S1CAT::Varchar(len)) => Self::Element(CATE::String(AtomicTyString::Varchar(len))),
            Some(S1CAT::Char(len)) => Self::Element(CATE::String(AtomicTyString::Char(len))),
            Some(S1CAT::Text) => Self::Element(CATE::String(AtomicTyString::Text)),

            Some(S1CAT::OnCreate) => Self::Element(CATE::AutoTime(AutoTimeEvent::OnCreate)),
            Some(S1CAT::OnUpdate) => Self::Element(CATE::AutoTime(AutoTimeEvent::OnUpdate)),
        }
    }
}

pub enum TyElement {
    Id,
    Value(TyElementValue),
    AutoTime(TyElementAutoTime),
}
impl TyElement {
    pub fn optional(&self) -> bool {
        matches!(self, Self::Value(value) if value.optional)
    }
}

pub enum Ty {
    Compund(TyCompound),
    Element(TyElement),
}
impl Ty {
    pub fn optional(&self) -> bool {
        match self {
            Self::Compund(compound) => compound.multiplicity.optional(),
            Self::Element(element) => element.optional(),
        }
    }
}

pub struct Column {
    /// the name of the column in the database
    pub name: String,
    /// the name of the column in the rust struct
    pub rs_name: Ident,
    /// the type of the column
    pub ty: Ty,
    /// the type of the column in the rust struct
    pub rs_ty: Type,
    /// the response attribute of the column
    pub attr_response: ColumnAttrResponse,
    /// the request attribute of the column
    pub attr_request: ColumnAttrRequest,

    pub rs_attrs: Vec<Attribute>,
}
impl TryFrom<stage1::Column> for Column {
    type Error = syn::Error;
    fn try_from(column: stage1::Column) -> Result<Self, Self::Error> {
        let stage1::Column {
            rs_name,
            rs_ty,
            attr,
        } = column;

        let name = attr.name.unwrap_or_else(|| rs_name.unraw().to_string());

        let attr_ty = ColumnAttrTy::from(attr.ty);
        let ty = match attr_ty {
            ColumnAttrTy::Compound(attr_ty_compound) => {
                let ty_compound = TyCompound::try_from(&rs_ty)?;
                Ty::Compund(ty_compound)
            }
            ColumnAttrTy::Element(attr_ty_element) => {
                use ColumnAttrTyElement as CATE;
                let rs_ty = match attr_ty_element {
                    CATE::Value(ref rs_ty) => rs_ty,
                    _ => &rs_ty,
                };
                let ty_element_value = stage1::TyElementValue::try_from(rs_ty)?;
                let ty_element_value = TyElementValue::from(ty_element_value);
                match attr_ty_element {
                    CATE::None | CATE::Value(_) => Ty::Element(TyElement::Value(ty_element_value)),
                    CATE::Id => {
                        let TyElementValue { ty, optional } = ty_element_value;
                        let AtomicTy::u64 = ty else {
                            return Err(syn::Error::new(rs_ty.span(), ID_MUST_BE_U64));
                        };
                        if optional {
                            return Err(syn::Error::new(rs_ty.span(), COLUMN_MUST_NOT_BE_OPTIONAL));
                        }
                        Ty::Element(TyElement::Id)
                    }
                    CATE::String(atomic_ty_string) => {
                        let TyElementValue { ty, optional } = ty_element_value;
                        let AtomicTy::String(_) = ty else {
                            return Err(syn::Error::new(rs_ty.span(), COLUMN_MUST_BE_STRING));
                        };

                        Ty::Element(TyElement::Value(TyElementValue {
                            ty: AtomicTy::String(atomic_ty_string),
                            optional,
                        }))
                    }
                    CATE::AutoTime(auto_time_event) => {
                        let TyElementValue { ty, optional } = ty_element_value;
                        let AtomicTy::Time(ty) = ty else {
                            return Err(syn::Error::new(rs_ty.span(), COLUMN_MUST_BE_TIME));
                        };
                        if optional {
                            return Err(syn::Error::new(rs_ty.span(), COLUMN_MUST_NOT_BE_OPTIONAL));
                        };

                        Ty::Element(TyElement::AutoTime(TyElementAutoTime {
                            ty,
                            event: auto_time_event,
                        }))
                    }
                }
            }
        };

        Ok(Self {
            name,
            rs_name,
            ty,
            rs_ty,
            attr_response: attr.attr_response,
            attr_request: attr.attr_request,
            rs_attrs: attr.attrs,
        })
    }
}

pub struct Table {
    /// the name for the sql table, for example `customers`
    pub name: String,
    /// the name for the table struct, for example `Customer`
    pub rs_name: Ident,
    /// the columns in the database
    pub columns: Vec<Column>,
    /// the name for the id of the table, for example `CustomerId`
    pub id_name: String,
    /// automatically implement the model
    pub model: bool,
    /// automatically implement the controller (model must be implemented), using the db as the state
    pub controller: bool,
    /// visibility
    pub rs_vis: Visibility,
    /// attributes
    pub rs_attrs: Vec<Attribute>,
}
impl TryFrom<stage1::Table> for Table {
    type Error = syn::Error;
    fn try_from(table: stage1::Table) -> Result<Self, Self::Error> {
        let stage1::Table {
            rs_name,
            columns,
            attr:
                stage1::TableAttr {
                    controller,
                    model,
                    name,
                    attrs: rs_attrs,
                },
            rs_vis,
        } = table;

        if controller && !model {
            return Err(syn::Error::new(rs_name.span(), TABLE_MUST_IMPLEMENT_MODEL));
        }

        let name = name.unwrap_or_else(|| rs_name.unraw().to_string());

        let mut id_name = None;
        let columns = columns.into_iter().map(|column| {
            let column = Column::try_from(column)?;
            if let Ty::Element(TyElement::Id) = column.ty {
                if id_name.is_some() {
                    return Err(syn::Error::new(
                        column.rs_name.span(),
                        TABLE_MUST_NOT_HAVE_MULTIPLE_IDS,
                    ));
                }
                id_name = Some(column.name.clone());
            }
            Ok(column)
        });
        let columns: Result<Vec<Column>, syn::Error> = columns.try_collect_all_default();
        let columns = columns?;
        let id_name = id_name.ok_or_else(|| syn::Error::new(rs_name.span(), TABLE_MUST_HAVE_ID))?;

        Ok(Self {
            name,
            rs_name,
            columns,
            id_name,
            model,
            controller,
            rs_vis,
            rs_attrs,
        })
    }
}

pub struct Db {
    /// the name of the database
    pub name: String,
    /// the name for the database module, for example `db`
    pub rs_name: Ident,
    /// the tables in the database
    pub tables: Vec<Table>,
    /// visibility
    pub rs_vis: Visibility,
}
impl Db {
    pub fn try_new(db: stage1::Db, attr: stage1::DbAttr) -> syn::Result<Self> {
        let stage1::DbAttr { name } = attr;
        let stage1::Db {
            rs_name,
            tables,
            rs_vis,
        } = db;

        let name = name.unwrap_or_else(|| rs_name.unraw().to_string());

        let tables = tables.into_iter().map(Table::try_from);
        let tables: Result<Vec<Table>, syn::Error> = tables.try_collect_all_default();
        let tables = tables?;

        Ok(Self {
            rs_name,
            name,
            tables,
            rs_vis,
        })
    }
}

pub fn find_table<'a>(tables: &'a [Table], ident: &Ident) -> Option<&'a Table> {
    tables.iter().find(|table| &table.rs_name == ident)
}
