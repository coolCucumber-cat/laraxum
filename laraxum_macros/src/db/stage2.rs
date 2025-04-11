use syn::{Ident, Type, Visibility, spanned::Spanned};

use crate::utils::collections::TryCollectAll;

pub use super::stage1::ForeignTy;
use super::stage1::{self};

const TABLE_MUST_HAVE_ID: &str = "table must have an ID";
const TABLE_MUST_NOT_HAVE_MULTIPLE_IDS: &str = "table must not have multiple IDs";
const ID_MUST_BE_U64: &str = "id must be u64";
const COLUMN_MUST_BE_STRING: &str = "must be string";
const COLUMN_MUST_NOT_BE_OPTIONAL: &str = "must not be null";
const COLUMN_MUST_NOT_HAVE_CONFLICTING_TYPES: &str = "column must not have conflicting types";

#[derive(Copy, Clone)]
pub enum StringScalarTy {
    Varchar(stage1::StringLen),
    Char(stage1::StringLen),
    Text,
}

#[derive(Copy, Clone)]
pub enum TimeScalarTy {
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
#[derive(Copy, Clone)]
pub enum ScalarTy {
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
    String(StringScalarTy),
    Time(TimeScalarTy),
}

impl From<stage1::ScalarTy> for ScalarTy {
    fn from(stage1_scalar_ty: stage1::ScalarTy) -> Self {
        match stage1_scalar_ty {
            stage1::ScalarTy::bool => Self::bool,
            stage1::ScalarTy::u8 => Self::u8,
            stage1::ScalarTy::i8 => Self::i8,
            stage1::ScalarTy::u16 => Self::u16,
            stage1::ScalarTy::i16 => Self::i16,
            stage1::ScalarTy::u32 => Self::u32,
            stage1::ScalarTy::i32 => Self::i32,
            stage1::ScalarTy::u64 => Self::u64,
            stage1::ScalarTy::i64 => Self::i64,
            stage1::ScalarTy::f32 => Self::f32,
            stage1::ScalarTy::f64 => Self::f64,

            stage1::ScalarTy::String => Self::String(StringScalarTy::Varchar(255)),

            stage1::ScalarTy::ChronoDateTimeUtc => Self::Time(TimeScalarTy::ChronoDateTimeUtc),
            stage1::ScalarTy::ChronoDateTimeLocal => Self::Time(TimeScalarTy::ChronoDateTimeLocal),
            stage1::ScalarTy::ChronoNaiveDateTime => Self::Time(TimeScalarTy::ChronoNaiveDateTime),
            stage1::ScalarTy::ChronoNaiveDate => Self::Time(TimeScalarTy::ChronoNaiveDate),
            stage1::ScalarTy::ChronoNaiveTime => Self::Time(TimeScalarTy::ChronoNaiveTime),
            stage1::ScalarTy::ChronoTimeDelta => Self::Time(TimeScalarTy::ChronoTimeDelta),

            stage1::ScalarTy::TimeDateTime => Self::Time(TimeScalarTy::TimePrimitiveDateTime),
            stage1::ScalarTy::TimeOffsetDateTime => Self::Time(TimeScalarTy::TimeOffsetDateTime),
            stage1::ScalarTy::TimeDate => Self::Time(TimeScalarTy::TimeDate),
            stage1::ScalarTy::TimeTime => Self::Time(TimeScalarTy::TimeTime),
            stage1::ScalarTy::TimeDuration => Self::Time(TimeScalarTy::TimeDuration),
        }
    }
}

#[derive(Copy, Clone)]
pub struct RealTy {
    pub ty: ScalarTy,
    pub optional: bool,
}

impl From<stage1::RealTy> for RealTy {
    fn from(stage1_real_ty: stage1::RealTy) -> Self {
        Self {
            ty: ScalarTy::from(stage1_real_ty.ty),
            optional: stage1_real_ty.optional,
        }
    }
}

enum AutoTimeAttr {
    OnCreate,
    OnUpdate,
}

enum ColumnAttrNotForeign {
    None,
    Id,
    String(StringScalarTy),
    AutoTime(AutoTimeAttr),
}

enum ColumnAttr {
    Foreign,
    NotForeign(ColumnAttrNotForeign),
}

impl ColumnAttr {
    const DEFAULT: Self = Self::NotForeign(ColumnAttrNotForeign::None);
    fn set(&mut self, attr: Self, ident: &Ident) -> syn::Result<()> {
        if matches!(self, Self::NotForeign(ColumnAttrNotForeign::None)) {
            *self = attr;
            Ok(())
        } else {
            Err(syn::Error::new(
                ident.span(),
                COLUMN_MUST_NOT_HAVE_CONFLICTING_TYPES,
            ))
        }
    }
}

pub enum VirtualTy {
    Id,
    Real(RealTy),
    Foreign(ForeignTy),
    OnCreate(TimeScalarTy),
    OnUpdate(TimeScalarTy),
}

pub struct ColumnTy {
    /// the type for the column
    pub virtual_ty: VirtualTy,
    /// the parsed rust type for the column
    pub rs_ty: Type,
}

impl ColumnTy {
    fn try_from_attr_and_ty(attr: ColumnAttr, rs_ty: Type) -> syn::Result<Self> {
        // turn combination of attrs and types into valid type
        let virtual_ty = match attr {
            ColumnAttr::Foreign => {
                let foreign_ty = ForeignTy::try_from(&rs_ty)?;
                VirtualTy::Foreign(foreign_ty)
            }
            ColumnAttr::NotForeign(attr) => {
                use ColumnAttrNotForeign as CANF;
                let stage1_real_ty = stage1::RealTy::try_from(&rs_ty)?;
                match attr {
                    CANF::None => VirtualTy::Real(RealTy::from(stage1_real_ty)),
                    CANF::Id => {
                        let stage1::RealTy {
                            ty: stage1::ScalarTy::u64,
                            optional,
                        } = stage1_real_ty
                        else {
                            return Err(syn::Error::new(rs_ty.span(), ID_MUST_BE_U64));
                        };
                        if optional {
                            return Err(syn::Error::new(rs_ty.span(), COLUMN_MUST_NOT_BE_OPTIONAL));
                        };
                        VirtualTy::Id
                    }
                    CANF::String(ty) => {
                        let stage1::RealTy {
                            ty: stage1::ScalarTy::String,
                            optional,
                        } = stage1_real_ty
                        else {
                            return Err(syn::Error::new(rs_ty.span(), COLUMN_MUST_BE_STRING));
                        };
                        VirtualTy::Real(RealTy {
                            ty: ScalarTy::String(ty),
                            optional,
                        })
                    }
                    CANF::AutoTime(auto_time_attr) => {
                        let stage1::RealTy { ty, optional } = stage1_real_ty;
                        let ty = ScalarTy::from(ty);
                        let ScalarTy::Time(ty) = ty else {
                            return Err(syn::Error::new(rs_ty.span(), ""));
                        };
                        if optional {
                            return Err(syn::Error::new(rs_ty.span(), COLUMN_MUST_NOT_BE_OPTIONAL));
                        };
                        match auto_time_attr {
                            AutoTimeAttr::OnCreate => VirtualTy::OnCreate(ty),
                            AutoTimeAttr::OnUpdate => VirtualTy::OnUpdate(ty),
                        }
                    }
                }
            }
        };
        Ok(Self { virtual_ty, rs_ty })
    }
}

pub struct Column {
    /// the name for the column in the response
    pub response_ident: Ident,
    /// the name for the column in the request
    pub request_ident: Ident,
    /// the name for the column in the database
    pub sql_name: String,
    /// the type of the column
    pub ty: ColumnTy,
}

impl TryFrom<stage1::Column> for Column {
    type Error = syn::Error;
    fn try_from(stage1_column: stage1::Column) -> Result<Self, Self::Error> {
        let stage1::Column {
            ident: response_ident,
            ty: rs_ty,
            attrs: stage1_attrs,
        } = stage1_column;

        let request_ident = stage1_attrs
            .request_ident
            .unwrap_or_else(|| response_ident.clone());

        let sql_name = stage1_attrs
            .sql_name
            .unwrap_or_else(|| request_ident.to_string());

        let mut attr = ColumnAttr::DEFAULT;
        if stage1_attrs.id {
            attr.set(
                ColumnAttr::NotForeign(ColumnAttrNotForeign::Id),
                &response_ident,
            )?;
        };
        if stage1_attrs.foreign {
            attr.set(ColumnAttr::Foreign, &response_ident)?;
        };
        if stage1_attrs.on_create {
            attr.set(
                ColumnAttr::NotForeign(ColumnAttrNotForeign::AutoTime(AutoTimeAttr::OnCreate)),
                &response_ident,
            )?;
        };
        if stage1_attrs.on_update {
            attr.set(
                ColumnAttr::NotForeign(ColumnAttrNotForeign::AutoTime(AutoTimeAttr::OnUpdate)),
                &response_ident,
            )?;
        };
        if let Some(len) = stage1_attrs.varchar {
            attr.set(
                ColumnAttr::NotForeign(ColumnAttrNotForeign::String(StringScalarTy::Varchar(len))),
                &response_ident,
            )?;
        };
        if let Some(len) = stage1_attrs.char {
            attr.set(
                ColumnAttr::NotForeign(ColumnAttrNotForeign::String(StringScalarTy::Char(len))),
                &response_ident,
            )?;
        };
        if stage1_attrs.text {
            attr.set(
                ColumnAttr::NotForeign(ColumnAttrNotForeign::String(StringScalarTy::Text)),
                &response_ident,
            )?;
        };

        let ty = ColumnTy::try_from_attr_and_ty(attr, rs_ty)?;

        Ok(Self {
            response_ident,
            request_ident,
            sql_name,
            ty,
        })
    }
}

pub struct Table {
    /// the name for the table struct, for example `Customer`
    pub ident: Ident,
    /// the name for the sql table, for example `customers`
    pub name: String,
    /// the columns in the database
    pub columns: Vec<Column>,
    /// the name for the id of the table, for example `CustomerId`
    pub id_ident: String,
    /// automatically implement the controller as well as the model, using the db as the state
    pub auto_impl_controller: bool,
    /// vis
    pub vis: Visibility,
}

impl TryFrom<stage1::Table> for Table {
    type Error = syn::Error;
    fn try_from(stage1_table: stage1::Table) -> Result<Self, Self::Error> {
        let stage1::Table {
            ident,
            columns,
            attrs:
                stage1::TableAttrs {
                    auto_impl_controller,
                    sql_name: name,
                    attrs: _,
                },
            vis,
        } = stage1_table;

        let name = name.unwrap_or_else(|| ident.to_string());

        let mut id_ident = None;
        let columns = columns.into_iter().map(|stage1_column| {
            let column = Column::try_from(stage1_column)?;
            if matches!(column.ty.virtual_ty, VirtualTy::Id) {
                if id_ident.is_some() {
                    return Err(syn::Error::new(
                        column.response_ident.span(),
                        TABLE_MUST_NOT_HAVE_MULTIPLE_IDS,
                    ));
                }
                id_ident = Some(column.sql_name.clone());
            }
            Ok(column)
        });
        let columns: Result<Vec<Column>, syn::Error> = columns.try_collect_all_default();
        let columns = columns?;

        let id_ident = id_ident.ok_or_else(|| syn::Error::new(ident.span(), TABLE_MUST_HAVE_ID))?;

        Ok(Self {
            ident,
            name,
            columns,
            id_ident,
            auto_impl_controller,
            vis,
        })
    }
}

pub struct Db {
    /// the name for the database module, for example `db`
    pub ident: Ident,
    /// the name of the database
    pub name: String,
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

    pub fn find_table(&self, ident: &Ident) -> Option<&Table> {
        self.tables.iter().find(|table| &table.ident == ident)
    }
}
