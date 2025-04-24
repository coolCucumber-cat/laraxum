use super::stage1;
pub use super::stage1::TyCompound;

use crate::utils::collections::TryCollectAll;

use syn::{Ident, Type, Visibility, spanned::Spanned};

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

impl From<stage1::ValueTy> for TyElementValue {
    fn from(value_ty: stage1::ValueTy) -> Self {
        Self {
            ty: AtomicTy::from(value_ty.ty),
            optional: value_ty.optional,
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

enum ColumnTyAttrElement {
    None,
    Id,
    String(AtomicTyString),
    AutoTime(AutoTimeEvent),
}

enum ColumnTyAttr {
    Compound,
    Element(ColumnTyAttrElement),
}

impl From<Option<stage1::ColumnAttrTy>> for ColumnTyAttr {
    fn from(attr: Option<stage1::ColumnAttrTy>) -> Self {
        use stage1::ColumnAttrTy as CTA;
        match attr {
            Some(CTA::Foreign) => Self::Compound,
            Some(CTA::Id) => Self::Element(ColumnTyAttrElement::Id),
            Some(CTA::OnCreate) => {
                Self::Element(ColumnTyAttrElement::AutoTime(AutoTimeEvent::OnCreate))
            }
            Some(CTA::OnUpdate) => {
                Self::Element(ColumnTyAttrElement::AutoTime(AutoTimeEvent::OnUpdate))
            }
            Some(CTA::Varchar(len)) => {
                Self::Element(ColumnTyAttrElement::String(AtomicTyString::Varchar(len)))
            }
            Some(CTA::Char(len)) => {
                Self::Element(ColumnTyAttrElement::String(AtomicTyString::Char(len)))
            }
            Some(CTA::Text) => Self::Element(ColumnTyAttrElement::String(AtomicTyString::Text)),
            None => Self::Element(ColumnTyAttrElement::None),
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
        matches!(self, Self::Value(real_ty) if real_ty.optional)
    }
}

pub enum Ty {
    Compund(TyCompound),
    Element(TyElement),
}

impl Ty {
    pub fn optional(&self) -> bool {
        match self {
            Self::Compund(foreign) => foreign.optional,
            Self::Element(inner) => inner.optional(),
        }
    }
}

pub struct ColumnTy {
    /// the type for the column
    pub virtual_ty: Ty,
    /// the parsed rust type for the column
    pub rs_ty: Type,
}

pub struct Column {
    /// the name for the column in the database
    pub name: String,
    /// the name for the column in the response
    pub response_name: Ident,
    /// the name for the column in the request
    pub request_name: Ident,
    /// the type of the column
    pub ty: ColumnTy,
}

impl TryFrom<stage1::Column> for Column {
    type Error = syn::Error;
    fn try_from(stage1_column: stage1::Column) -> Result<Self, Self::Error> {
        let stage1::Column {
            ident: response_name,
            ty: rs_ty,
            attr,
        } = stage1_column;

        let request_name = attr.request_name.unwrap_or_else(|| response_name.clone());

        let name = attr.name.unwrap_or_else(|| request_name.to_string());

        let attr_ty = ColumnTyAttr::from(attr.ty);
        let virtual_ty = match attr_ty {
            ColumnTyAttr::Compound => {
                let foreign_ty = TyCompound::try_from(&rs_ty)?;
                Ty::Compund(foreign_ty)
            }
            ColumnTyAttr::Element(attr) => {
                use ColumnTyAttrElement as CAI;
                let stage1_real_ty = stage1::ValueTy::try_from(&rs_ty)?;
                match attr {
                    CAI::None => {
                        Ty::Element(TyElement::Value(TyElementValue::from(stage1_real_ty)))
                    }
                    CAI::Id => {
                        let stage1::ValueTy {
                            ty: stage1::AtomicTy::u64,
                            optional,
                        } = stage1_real_ty
                        else {
                            return Err(syn::Error::new(rs_ty.span(), ID_MUST_BE_U64));
                        };
                        if optional {
                            return Err(syn::Error::new(rs_ty.span(), COLUMN_MUST_NOT_BE_OPTIONAL));
                        };
                        Ty::Element(TyElement::Id)
                    }
                    CAI::String(ty) => {
                        let stage1::ValueTy {
                            ty: stage1::AtomicTy::String,
                            optional,
                        } = stage1_real_ty
                        else {
                            return Err(syn::Error::new(rs_ty.span(), COLUMN_MUST_BE_STRING));
                        };
                        Ty::Element(TyElement::Value(TyElementValue {
                            ty: AtomicTy::String(ty),
                            optional,
                        }))
                    }
                    CAI::AutoTime(event) => {
                        let stage1::ValueTy { ty, optional } = stage1_real_ty;
                        let ty = AtomicTy::from(ty);
                        let AtomicTy::Time(ty) = ty else {
                            return Err(syn::Error::new(rs_ty.span(), COLUMN_MUST_BE_TIME));
                        };
                        if optional {
                            return Err(syn::Error::new(rs_ty.span(), COLUMN_MUST_NOT_BE_OPTIONAL));
                        };
                        Ty::Element(TyElement::AutoTime(TyElementAutoTime { ty, event }))
                    }
                }
            }
        };

        Ok(Self {
            response_name,
            request_name,
            name,
            ty: ColumnTy { virtual_ty, rs_ty },
        })
    }
}

pub struct Table {
    /// the name for the sql table, for example `customers`
    pub name: String,
    /// the name for the table struct, for example `Customer`
    pub ty: Ident,
    /// the columns in the database
    pub columns: Vec<Column>,
    /// the name for the id of the table, for example `CustomerId`
    pub id_name: String,
    /// automatically implement the model
    pub model: bool,
    /// automatically implement the controller (model must be implemented), using the db as the state
    pub controller: bool,
    /// vis
    pub vis: Visibility,
}

impl TryFrom<stage1::Table> for Table {
    type Error = syn::Error;
    fn try_from(stage1_table: stage1::Table) -> Result<Self, Self::Error> {
        let stage1::Table {
            ident: ty,
            columns,
            attr:
                stage1::TableAttr {
                    controller,
                    model,
                    name,
                    attrs: _,
                },
            vis,
        } = stage1_table;

        if controller && !model {
            return Err(syn::Error::new(ty.span(), TABLE_MUST_IMPLEMENT_MODEL));
        }

        let name = name.unwrap_or_else(|| ty.to_string());

        let mut id_ident = None;
        let columns = columns.into_iter().map(|stage1_column| {
            let column = Column::try_from(stage1_column)?;
            if matches!(column.ty.virtual_ty, Ty::Element(TyElement::Id)) {
                if id_ident.is_some() {
                    return Err(syn::Error::new(
                        column.response_name.span(),
                        TABLE_MUST_NOT_HAVE_MULTIPLE_IDS,
                    ));
                }
                id_ident = Some(column.name.clone());
            }
            Ok(column)
        });
        let columns: Result<Vec<Column>, syn::Error> = columns.try_collect_all_default();
        let columns = columns?;

        let id_name = id_ident.ok_or_else(|| syn::Error::new(ty.span(), TABLE_MUST_HAVE_ID))?;

        Ok(Self {
            name,
            ty,
            columns,
            id_name,
            model,
            controller,
            vis,
        })
    }
}

pub struct Db {
    /// the name of the database
    pub name: String,
    /// the name for the database module, for example `db`
    pub ident: Ident,
    /// the tables in the database
    pub tables: Vec<Table>,
    /// vis
    pub vis: Visibility,
}

impl Db {
    pub fn try_from_db_and_attr(
        stage1_db: stage1::Db,
        stage1_db_attr: stage1::DbAttr,
    ) -> syn::Result<Self> {
        let stage1::DbAttr { name } = stage1_db_attr;
        let stage1::Db { ident, tables, vis } = stage1_db;

        let name = name.unwrap_or_else(|| ident.to_string());

        let tables = tables.into_iter().map(Table::try_from);
        let tables: Result<Vec<Table>, syn::Error> = tables.try_collect_all_default();
        let tables = tables?;

        Ok(Self {
            ident,
            name,
            tables,
            vis,
        })
    }
}

pub fn find_table<'a>(tables: &'a [Table], ident: &Ident) -> Option<&'a Table> {
    tables.iter().find(|table| &table.ty == ident)
}
