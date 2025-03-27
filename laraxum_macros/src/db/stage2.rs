use syn::{Ident, Type, Visibility, spanned::Spanned};

use super::stage1::{self, ForeignTy, StringLen};

const TABLE_MUST_HAVE_ID: &str = "table must have an ID";
const TABLE_MUST_NOT_HAVE_MULTIPLE_IDS: &str = "table must not have multiple IDs";
const DUPLICATE_ATTR: &str = "duplicate attribute";
const ID_MUST_BE_U64: &str = "id must be u64";
const COLUMN_MUST_BE_STRING: &str = "must be string";
const COLUMN_MUST_NOT_BE_OPTIONAL: &str = "must not be null";

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ScalarTy {
    Varchar(StringLen),
    Char(StringLen),
    Text,
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

    TimePrimitiveDateTime,
    TimeOffsetDateTime,
    TimeDate,
    TimeTime,
    TimeDuration,

    ChronoDateTimeUtc,
    ChronoDateTimeLocal,
    ChronoNaiveDateTime,
    ChronoNaiveDate,
    ChronoNaiveTime,
    ChronoTimeDelta,
}

impl From<stage1::ScalarTy> for ScalarTy {
    fn from(stage1_scalar_ty: stage1::ScalarTy) -> Self {
        match stage1_scalar_ty {
            stage1::ScalarTy::String => Self::Varchar(255),
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
            stage1::ScalarTy::TimePrimitiveDateTime => Self::TimePrimitiveDateTime,
            stage1::ScalarTy::TimeOffsetDateTime => Self::TimeOffsetDateTime,
            stage1::ScalarTy::TimeDate => Self::TimeDate,
            stage1::ScalarTy::TimeTime => Self::TimeTime,
            stage1::ScalarTy::TimeDuration => Self::TimeDuration,
            stage1::ScalarTy::ChronoDateTimeUtc => Self::ChronoDateTimeUtc,
            stage1::ScalarTy::ChronoDateTimeLocal => Self::ChronoDateTimeLocal,
            stage1::ScalarTy::ChronoNaiveDateTime => Self::ChronoNaiveDateTime,
            stage1::ScalarTy::ChronoNaiveDate => Self::ChronoNaiveDate,
            stage1::ScalarTy::ChronoNaiveTime => Self::ChronoNaiveTime,
            stage1::ScalarTy::ChronoTimeDelta => Self::ChronoTimeDelta,
        }
    }
}

pub use ScalarTy as TimeTy;

#[derive(Clone, Copy)]
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

#[derive(Clone, PartialEq, Eq)]
enum ColumnAttr {
    Foreign,
    NotForeign(ColumnAttrNotForeign),
}

impl ColumnAttr {
    const DEFAULT: Self = Self::NotForeign(ColumnAttrNotForeign::None);
}

#[derive(Clone, PartialEq, Eq)]
enum AutoTimeEvent {
    OnCreate,
    OnUpdate,
}

#[derive(Clone, PartialEq, Eq)]
enum ColumnAttrNotForeign {
    None,
    Id,
    Varchar(StringLen),
    Char(StringLen),
    Text,
    AutoTime(AutoTimeEvent),
}

#[derive(Clone)]
pub enum VirtualTy {
    Id,
    Real(RealTy),
    Foreign(ForeignTy),
    OnCreate(TimeTy),
    OnUpdate(TimeTy),
}

#[derive(Clone)]
pub struct Column {
    /// the name for the column in the response
    pub response_ident: Ident,
    /// the name for the column in the request
    pub request_ident: Ident,
    /// the type for the column
    pub virtual_ty: VirtualTy,
    /// the parsed rust type for the column
    pub rs_ty: Type,
}

impl TryFrom<stage1::Column> for Column {
    type Error = syn::Error;
    fn try_from(stage1_column: stage1::Column) -> Result<Self, Self::Error> {
        let stage1::Column {
            ident: response_ident,
            ty: rs_ty,
            attrs: stage1_attrs,
        } = stage1_column;

        let mut request_ident = None;
        let mut attr = ColumnAttr::DEFAULT;
        // turn list of attrs into singular, unique values
        for stage1_attr in stage1_attrs {
            match stage1_attr {
                stage1::ColumnAttr::Id if attr == ColumnAttr::DEFAULT => {
                    attr = ColumnAttr::NotForeign(ColumnAttrNotForeign::Id)
                }
                stage1::ColumnAttr::OnCreate if attr == ColumnAttr::DEFAULT => {
                    attr = ColumnAttr::NotForeign(ColumnAttrNotForeign::AutoTime(
                        AutoTimeEvent::OnCreate,
                    ))
                }
                stage1::ColumnAttr::OnUpdate if attr == ColumnAttr::DEFAULT => {
                    attr = ColumnAttr::NotForeign(ColumnAttrNotForeign::AutoTime(
                        AutoTimeEvent::OnUpdate,
                    ))
                }
                stage1::ColumnAttr::Foreign if attr == ColumnAttr::DEFAULT => {
                    attr = ColumnAttr::Foreign
                }
                stage1::ColumnAttr::Varchar(len) if attr == ColumnAttr::DEFAULT => {
                    attr = ColumnAttr::NotForeign(ColumnAttrNotForeign::Varchar(len))
                }
                stage1::ColumnAttr::Char(len) if attr == ColumnAttr::DEFAULT => {
                    attr = ColumnAttr::NotForeign(ColumnAttrNotForeign::Char(len))
                }
                stage1::ColumnAttr::Text if attr == ColumnAttr::DEFAULT => {
                    attr = ColumnAttr::NotForeign(ColumnAttrNotForeign::Text)
                }
                stage1::ColumnAttr::Name(name) if request_ident.is_none() => {
                    request_ident = Some(name);
                }
                stage1::ColumnAttr::Response(_expr) => {}
                _ => return Err(syn::Error::new(response_ident.span(), DUPLICATE_ATTR)),
            }
        }

        let request_ident = request_ident.unwrap_or_else(|| response_ident.clone());
        // turn combination of attrs and types into valid type
        let virtual_ty = match attr {
            ColumnAttr::Foreign => {
                let foreign_ty = ForeignTy::try_from(rs_ty.clone())?;
                VirtualTy::Foreign(foreign_ty)
            }
            ColumnAttr::NotForeign(attr) => {
                use ColumnAttrNotForeign as CANF;
                let stage1_real_ty = stage1::RealTy::try_from(rs_ty.clone())?;
                match (attr, stage1_real_ty) {
                    (CANF::None, stage1_real_ty) => VirtualTy::Real(RealTy::from(stage1_real_ty)),
                    (
                        CANF::Id,
                        stage1::RealTy {
                            ty: stage1::ScalarTy::u64,
                            optional: false,
                        },
                    ) => VirtualTy::Id,
                    (
                        CANF::Id,
                        stage1::RealTy {
                            ty: stage1::ScalarTy::u64,
                            optional: true,
                        },
                    ) => return Err(syn::Error::new(rs_ty.span(), COLUMN_MUST_NOT_BE_OPTIONAL)),
                    (CANF::Id, _) => return Err(syn::Error::new(rs_ty.span(), ID_MUST_BE_U64)),
                    (
                        CANF::AutoTime(event),
                        stage1::RealTy {
                            ty: stage1_scalar_ty,
                            optional: false,
                        },
                    ) => {
                        let scalar_ty = ScalarTy::from(stage1_scalar_ty);
                        match event {
                            AutoTimeEvent::OnCreate => VirtualTy::OnCreate(scalar_ty),
                            AutoTimeEvent::OnUpdate => VirtualTy::OnUpdate(scalar_ty),
                        }
                    }
                    (CANF::AutoTime(_), _) => {
                        return Err(syn::Error::new(rs_ty.span(), COLUMN_MUST_NOT_BE_OPTIONAL));
                    }
                    (
                        CANF::Varchar(len),
                        stage1::RealTy {
                            ty: stage1::ScalarTy::String,
                            optional,
                        },
                    ) => VirtualTy::Real(RealTy {
                        ty: ScalarTy::Varchar(len),
                        optional,
                    }),
                    (
                        CANF::Char(len),
                        stage1::RealTy {
                            ty: stage1::ScalarTy::String,
                            optional,
                        },
                    ) => VirtualTy::Real(RealTy {
                        ty: ScalarTy::Char(len),
                        optional,
                    }),
                    (
                        CANF::Text,
                        stage1::RealTy {
                            ty: stage1::ScalarTy::String,
                            optional,
                        },
                    ) => VirtualTy::Real(RealTy {
                        ty: ScalarTy::Text,
                        optional,
                    }),
                    (CANF::Varchar(_) | CANF::Char(_) | CANF::Text, _) => {
                        return Err(syn::Error::new(rs_ty.span(), COLUMN_MUST_BE_STRING));
                    }
                }
            }
        };

        Ok(Self {
            response_ident,
            request_ident,
            virtual_ty,
            rs_ty,
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
    pub id_ident: Ident,
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
            attrs: stage1_attrs,
            vis,
        } = stage1_table;

        let mut name = None;
        let mut auto_impl_controller = false;
        for stage1_attr in stage1_attrs {
            match stage1_attr {
                stage1::TableAttr::Auto => {
                    if auto_impl_controller {
                        return Err(syn::Error::new(ident.span(), DUPLICATE_ATTR));
                    }
                    auto_impl_controller = true;
                }
                stage1::TableAttr::Name(n) => {
                    if name.is_some() {
                        return Err(syn::Error::new(ident.span(), DUPLICATE_ATTR));
                    }
                    name = Some(n);
                }
            }
        }
        let name = name.unwrap_or_else(|| ident.to_string());

        let mut id_ident = None;
        let columns = columns.into_iter().map(|stage1_column| {
            let column = Column::try_from(stage1_column)?;
            if matches!(column.virtual_ty, VirtualTy::Id) {
                if id_ident.is_some() {
                    return Err(syn::Error::new(
                        column.response_ident.span(),
                        TABLE_MUST_NOT_HAVE_MULTIPLE_IDS,
                    ));
                }
                id_ident = Some(column.response_ident.clone());
            }
            Ok(column)
        });
        let columns: Result<Vec<Column>, syn::Error> = columns.collect();
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
        let tables: Result<Vec<Table>, syn::Error> = tables.collect();
        let tables = tables?;

        Ok(Self {
            ident,
            name,
            tables,
            vis,
        })
    }
}
