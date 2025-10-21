use super::stage1;

use crate::utils::{collections::TryCollectAll, multiplicity};

use syn::{Attribute, Ident, Type, Visibility, ext::IdentExt, spanned::Spanned};

const TABLE_MUST_HAVE_ID: &str = "table must have an ID";
const TABLE_MUST_NOT_HAVE_ID: &str = "table must not have an ID";
const TABLE_MUST_NOT_HAVE_MULTIPLE_IDS: &str = "table must not have multiple IDs";
const TABLE_MUST_IMPLEMENT_MODEL: &str = "table must implement model to implement controller";
const TABLE_MUST_HAVE_TWO_COLUMNS: &str = "table must have two columns";
const TABLE_DOES_NOT_EXIST: &str = "table does not exist";
const TABLE_MUST_NOT_IMPLEMENT_CONTROLLER: &str = "table must not implement controller";
// const MODEL_AND_MANY_MODEL_CONFLICT: &str = "model and many_model conflict";
const ID_MUST_BE_INT: &str = "id must be int";
const COLUMN_MUST_BE_STRING: &str = "column must be string";
const COLUMN_MUST_BE_TIME: &str = "column must be time";
const COLUMN_MUST_NOT_BE_OPTIONAL: &str = "column must not be optional";
const COLUMN_MUST_NOT_BE_UNIQUE: &str = "column must not be unique";
const COLUMN_MUST_BE_VEC: &str = "column must be Vec";
const COLUMN_MUST_SPECIFY_INTERMEDIATE_TABLE: &str = "column must specify intermediate table";

#[expect(non_camel_case_types)]
#[derive(Clone)]
pub enum AtomicTyInt {
    u8,
    i8,
    u16,
    i16,
    u32,
    i32,
    u64,
    i64,
}

#[expect(non_camel_case_types)]
#[derive(Clone)]
pub enum AtomicTyFloat {
    f32,
    f64,
}

#[derive(Clone)]
pub enum AtomicTyString {
    Varchar(stage1::StringLen),
    Char(stage1::StringLen),
    Text,
}

#[derive(Clone)]
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

#[expect(non_camel_case_types)]
#[derive(Clone)]
pub enum AtomicTy {
    bool,
    Int(AtomicTyInt),
    Float(AtomicTyFloat),
    String(AtomicTyString),
    Time(AtomicTyTime),
}
impl From<stage1::AtomicTy> for AtomicTy {
    fn from(atomic_ty: stage1::AtomicTy) -> Self {
        match atomic_ty {
            stage1::AtomicTy::bool => Self::bool,

            stage1::AtomicTy::u8 => Self::Int(AtomicTyInt::u8),
            stage1::AtomicTy::i8 => Self::Int(AtomicTyInt::i8),
            stage1::AtomicTy::u16 => Self::Int(AtomicTyInt::u16),
            stage1::AtomicTy::i16 => Self::Int(AtomicTyInt::i16),
            stage1::AtomicTy::u32 => Self::Int(AtomicTyInt::u32),
            stage1::AtomicTy::i32 => Self::Int(AtomicTyInt::i32),
            stage1::AtomicTy::u64 => Self::Int(AtomicTyInt::u64),
            stage1::AtomicTy::i64 => Self::Int(AtomicTyInt::i64),

            stage1::AtomicTy::f32 => Self::Float(AtomicTyFloat::f32),
            stage1::AtomicTy::f64 => Self::Float(AtomicTyFloat::f64),

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

#[derive(Clone)]
pub struct TyElementValue {
    pub ty: AtomicTy,
    pub optional: bool,
    pub unique: bool,
}
impl TyElementValue {
    fn new(ty_element_value: stage1::TyElementValue, unique: bool) -> Self {
        Self {
            ty: AtomicTy::from(ty_element_value.ty),
            optional: ty_element_value.optional,
            unique,
        }
    }
}

#[derive(Clone)]
pub enum AutoTimeEvent {
    OnCreate,
    OnUpdate,
}

#[derive(Clone)]
pub struct TyElementAutoTime {
    pub ty: AtomicTyTime,
    pub event: AutoTimeEvent,
}

enum ColumnAttrTyElement {
    Id,
    None,
    String(AtomicTyString),
    AutoTime(AutoTimeEvent),
}

pub use stage1::ColumnAttrTyCompounds;

enum ColumnAttrTyCompound {
    One,
    Many(ColumnAttrTyCompounds),
}

enum ColumnAttrTy {
    Compound(ColumnAttrTyCompound),
    Element(ColumnAttrTyElement),
}
impl From<Option<stage1::ColumnAttrTy>> for ColumnAttrTy {
    fn from(attr_ty: Option<stage1::ColumnAttrTy>) -> Self {
        use ColumnAttrTyCompound as CATC;
        use ColumnAttrTyElement as CATE;
        use stage1::ColumnAttrTy as S1CAT;
        match attr_ty {
            Some(S1CAT::Compound(stage1::ColumnAttrTyCompound { many: None })) => {
                Self::Compound(CATC::One)
            }
            Some(S1CAT::Compound(stage1::ColumnAttrTyCompound { many: Some(many) })) => {
                Self::Compound(CATC::Many(many))
            }

            None => Self::Element(CATE::None),
            Some(S1CAT::Id) => Self::Element(CATE::Id),

            Some(S1CAT::Varchar(len)) => Self::Element(CATE::String(AtomicTyString::Varchar(len))),
            Some(S1CAT::Char(len)) => Self::Element(CATE::String(AtomicTyString::Char(len))),
            Some(S1CAT::Text) => Self::Element(CATE::String(AtomicTyString::Text)),

            Some(S1CAT::OnCreate) => Self::Element(CATE::AutoTime(AutoTimeEvent::OnCreate)),
            Some(S1CAT::OnUpdate) => Self::Element(CATE::AutoTime(AutoTimeEvent::OnUpdate)),
        }
    }
}

pub enum DefaultValue<'a> {
    AutoTime(&'a AtomicTyTime),
}

#[derive(Clone)]
pub enum TyElement {
    Id(AtomicTyInt),
    Value(TyElementValue),
    AutoTime(TyElementAutoTime),
}
impl TyElement {
    pub const fn optional(&self) -> bool {
        matches!(self, Self::Value(value) if value.optional)
    }
    pub const fn unique(&self) -> bool {
        match self {
            Self::Id(_) => true,
            Self::Value(value) => value.unique,
            Self::AutoTime(_) => false,
        }
    }
    pub const fn id(&self) -> Option<&AtomicTyInt> {
        match self {
            Self::Id(id) => Some(id),
            _ => None,
        }
    }
    pub const fn is_id(&self) -> bool {
        matches!(self, Self::Id(_))
    }
    pub const fn default_value(&self) -> Option<DefaultValue<'_>> {
        match self {
            Self::AutoTime(time_ty) => Some(DefaultValue::AutoTime(&time_ty.ty)),
            _ => None,
        }
    }
    pub const fn is_updatable(&self) -> bool {
        matches!(self, Self::Value(_))
    }
    pub const fn max_len(&self) -> Option<u16> {
        match self {
            Self::Value(TyElementValue {
                ty: AtomicTy::String(AtomicTyString::Varchar(len) | AtomicTyString::Char(len)),
                ..
            }) => Some(*len),
            _ => None,
        }
    }
}

pub enum TyCompoundMultiplicity {
    One { optional: bool, unique: bool },
    Many(ColumnAttrTyCompounds),
}
impl TyCompoundMultiplicity {
    pub const fn optional(&self) -> bool {
        matches!(*self, Self::One { optional, .. } if optional)
    }
    pub const fn unique(&self) -> bool {
        matches!(*self, Self::One { unique, .. } if unique)
    }
}

pub struct TyCompound {
    pub rs_ty_name: Ident,
    pub multiplicity: TyCompoundMultiplicity,
}
impl TyCompound {
    pub const fn optional(&self) -> bool {
        self.multiplicity.optional()
    }
    pub const fn unique(&self) -> bool {
        self.multiplicity.unique()
    }
}

pub enum Ty {
    Compound(TyCompound),
    Element(TyElement),
}
impl Ty {
    pub const fn optional(&self) -> bool {
        match self {
            Self::Compound(compound) => compound.optional(),
            Self::Element(element) => element.optional(),
        }
    }
    pub const fn unique(&self) -> bool {
        match self {
            Self::Compound(compound) => compound.unique(),
            Self::Element(element) => element.unique(),
        }
    }
    pub const fn id(&self) -> Option<&AtomicTyInt> {
        match self {
            Self::Compound(_) => None,
            Self::Element(element) => element.id(),
        }
    }
    pub const fn default_value(&self) -> Option<DefaultValue<'_>> {
        match self {
            Self::Compound(_) => None,
            Self::Element(element) => element.default_value(),
        }
    }
    pub const fn max_len(&self) -> Option<u16> {
        match self {
            Self::Compound(_) => None,
            Self::Element(element) => element.max_len(),
        }
    }
}

pub use stage1::ColumnAttrResponse;

pub use stage1::ColumnAttrRequest;

pub use stage1::Validate;

// pub struct ColumnAttrResponse {
//     pub name: Option<String>,
//     pub skip: bool,
// }
// pub enum ValidateRule {
//     MinLen(Expr),
//     MaxLen(usize),
//     Func(Expr),
//     Matches(PatRange),
//     NMatches(PatRange),
//     Eq(Expr),
//     NEq(Expr),
//     Gt(Expr),
//     Lt(Expr),
//     Gte(Expr),
//     Lte(Expr),
// }
//
// pub struct ColumnAttrRequest {
//     pub name: Option<String>,
//     pub validate: Vec<ValidateRule>,
//     // pub ty: Option<Type>,
// }

pub use stage1::ColumnAttrIndex;

pub use stage1::ColumnAttrIndexFilter;

pub use stage1::ColumnAttrIndexLimit;

pub struct Column {
    /// the name of the column in the database
    pub name: String,
    /// the name of the column in the rust struct
    pub rs_name: Ident,
    /// the type of the column
    pub ty: Ty,
    /// the type of the column in the rust struct
    pub rs_ty: Box<Type>,
    /// the response attribute of the column
    pub response: ColumnAttrResponse,
    /// the request attribute of the column
    pub request: ColumnAttrRequest,
    // /// validation rules
    // pub validate: Vec<ValidateRule>,
    pub is_mut: bool,
    /// borrowing behaviour
    pub borrow: Option<Option<Box<Type>>>,
    /// index
    pub index: Vec<ColumnAttrIndex>,
    /// struct name
    pub struct_name: Option<Ident>,

    pub rs_attrs: Vec<Attribute>,
}
impl TryFrom<stage1::Column> for Column {
    type Error = syn::Error;
    fn try_from(column: stage1::Column) -> Result<Self, Self::Error> {
        let stage1::Column {
            rs_name,
            rs_ty,
            attr:
                stage1::ColumnAttr {
                    name,
                    ty: attr_ty,
                    response,
                    mut request,
                    real_rs_ty,
                    unique,
                    is_mut,
                    borrow,
                    index,
                    struct_name,
                    attrs: rs_attrs,
                },
        } = column;

        let name = name.unwrap_or_else(|| rs_name.unraw().to_string());

        // the real type that we actually want to parse, while keeping the type in the field the same
        let real_rs_ty = real_rs_ty.as_deref();
        let real_rs_ty = real_rs_ty.unwrap_or(&*rs_ty);

        let attr_ty = ColumnAttrTy::from(attr_ty);
        let ty = match attr_ty {
            ColumnAttrTy::Compound(attr_ty_compound) => {
                use ColumnAttrTyCompound as CATC;
                use TyCompoundMultiplicity as TCM;
                use multiplicity::Multiplicity as M;
                let stage1::TyCompound {
                    ty,
                    multiplicity: ty_compound_multiplicity,
                } = stage1::TyCompound::try_from(real_rs_ty)?;
                let ty_compound_multiplicity = match (attr_ty_compound, ty_compound_multiplicity) {
                    (CATC::One, M::One) => TCM::One {
                        optional: false,
                        unique,
                    },
                    (CATC::One, M::OneOrZero) => TCM::One {
                        optional: true,
                        unique,
                    },
                    (CATC::One, M::Many) => {
                        return Err(syn::Error::new(
                            real_rs_ty.span(),
                            COLUMN_MUST_SPECIFY_INTERMEDIATE_TABLE,
                        ));
                    }
                    (CATC::Many(many), M::Many) => TCM::Many(many),
                    (CATC::Many(_), _) => {
                        return Err(syn::Error::new(real_rs_ty.span(), COLUMN_MUST_BE_VEC));
                    }
                };
                Ty::Compound(TyCompound {
                    rs_ty_name: ty,
                    multiplicity: ty_compound_multiplicity,
                })
            }
            ColumnAttrTy::Element(attr_ty_element) => {
                use ColumnAttrTyElement as CATE;
                let ty_element_value = stage1::TyElementValue::try_from(real_rs_ty)?;
                let ty_element_value = TyElementValue::new(ty_element_value, unique);
                match attr_ty_element {
                    CATE::None => Ty::Element(TyElement::Value(ty_element_value)),
                    CATE::Id => {
                        let TyElementValue {
                            ty,
                            optional,
                            unique,
                        } = ty_element_value;
                        if unique {
                            return Err(syn::Error::new(rs_ty.span(), COLUMN_MUST_NOT_BE_UNIQUE));
                        }
                        let AtomicTy::Int(ty) = ty else {
                            return Err(syn::Error::new(rs_ty.span(), ID_MUST_BE_INT));
                        };
                        if optional {
                            return Err(syn::Error::new(rs_ty.span(), COLUMN_MUST_NOT_BE_OPTIONAL));
                        }
                        Ty::Element(TyElement::Id(ty))
                    }
                    CATE::String(atomic_ty_string) => {
                        let TyElementValue {
                            ty,
                            optional,
                            unique,
                        } = ty_element_value;
                        let AtomicTy::String(_) = ty else {
                            return Err(syn::Error::new(rs_ty.span(), COLUMN_MUST_BE_STRING));
                        };

                        Ty::Element(TyElement::Value(TyElementValue {
                            ty: AtomicTy::String(atomic_ty_string),
                            optional,
                            unique,
                        }))
                    }
                    CATE::AutoTime(auto_time_event) => {
                        let TyElementValue {
                            ty,
                            optional,
                            unique,
                        } = ty_element_value;
                        if unique {
                            return Err(syn::Error::new(rs_ty.span(), COLUMN_MUST_NOT_BE_UNIQUE));
                        }
                        let AtomicTy::Time(ty) = ty else {
                            return Err(syn::Error::new(rs_ty.span(), COLUMN_MUST_BE_TIME));
                        };
                        if optional {
                            return Err(syn::Error::new(rs_ty.span(), COLUMN_MUST_NOT_BE_OPTIONAL));
                        }

                        Ty::Element(TyElement::AutoTime(TyElementAutoTime {
                            ty,
                            event: auto_time_event,
                        }))
                    }
                }
            }
        };

        let max_len_validate_rule = ty.max_len().map(usize::from);
        request.validate.max_len = max_len_validate_rule;

        Ok(Self {
            name,
            rs_name,
            ty,
            rs_ty,
            response,
            request,
            is_mut,
            borrow,
            index,
            struct_name,
            rs_attrs,
        })
    }
}

pub use stage1::TableAttrController;

pub enum Columns<T0, T1, C> {
    CollectionOnly {
        columns: Vec<T0>,
    },
    Model {
        id: T1,
        columns: Vec<T0>,
        controller: Option<C>,
    },
    ManyModel {
        a: T1,
        b: T1,
    },
}
impl<T, C> Columns<T, T, C> {
    pub fn iter(&self) -> impl Iterator<Item = &T> + Clone {
        let (a, b, c) = match self {
            Self::CollectionOnly { columns } => (None, None, &**columns),
            Self::Model { id, columns, .. } => (Some(id), None, &**columns),
            Self::ManyModel { a, b } => (Some(a), Some(b), &[][..]),
        };
        a.into_iter().chain(b).chain(c)
    }
}
impl<T0, T1, C> Columns<T0, T1, C> {
    pub const fn model(&self) -> Option<&T1> {
        match self {
            Self::Model { id, .. } => Some(id),
            _ => None,
        }
    }
    pub const fn many_model(&self) -> Option<(&T1, &T1)> {
        match self {
            Self::ManyModel { a, b } => Some((a, b)),
            _ => None,
        }
    }
    pub const fn is_collection(&self) -> bool {
        matches!(self, Self::CollectionOnly { .. } | Self::Model { .. })
    }
    pub const fn controller(&self) -> Option<&C> {
        match self {
            Self::Model { controller, .. } => controller.as_ref(),
            _ => None,
        }
    }
}

pub struct Table {
    /// the name for the sql table, for example `customers`
    pub name: String,
    /// the name for the table struct, for example `Customer`
    pub rs_name: Ident,
    /// the columns in the database
    pub columns: Columns<Column, Column, TableAttrController>,
    pub index_rs_name: Option<Ident>,
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
                    model,
                    controller,
                    name,
                    index_name: index_rs_name,
                    attrs: rs_attrs,
                },
            rs_vis,
        } = table;

        let name = name.unwrap_or_else(|| rs_name.unraw().to_string());

        let mut id = None;
        let columns = columns
            .into_iter()
            .map(|column| {
                let column = Column::try_from(column)?;
                if matches!(column.ty, Ty::Element(TyElement::Id(_))) {
                    if id.is_some() {
                        return Err(syn::Error::new(
                            column.rs_name.span(),
                            TABLE_MUST_NOT_HAVE_MULTIPLE_IDS,
                        ));
                    }
                    id = Some(column);
                    Ok(None)
                } else {
                    Ok(Some(column))
                }
            })
            .filter_map(Result::transpose);
        let columns: Result<Vec<Column>, syn::Error> = columns.try_collect_all_default();
        let columns = columns?;

        let model = model.map(|model| model.many);
        let columns = match model {
            Some(false) => {
                let Some(id) = id else {
                    return Err(syn::Error::new(rs_name.span(), TABLE_MUST_HAVE_ID));
                };
                let controller = controller.map(TableAttrController::from);
                Columns::Model {
                    id,
                    columns,
                    controller,
                }
            }
            _ if controller.is_some() => {
                return Err(syn::Error::new(
                    rs_name.span(),
                    TABLE_MUST_NOT_IMPLEMENT_CONTROLLER,
                ));
            }
            None => {
                if id.is_some() {
                    return Err(syn::Error::new(rs_name.span(), TABLE_MUST_IMPLEMENT_MODEL));
                }
                Columns::CollectionOnly { columns }
            }
            _ if id.is_some() => {
                return Err(syn::Error::new(rs_name.span(), TABLE_MUST_NOT_HAVE_ID));
            }

            Some(true) => {
                let mut columns = columns.into_iter();
                let span = rs_name.span();
                let f_err = || syn::Error::new(span, TABLE_MUST_HAVE_TWO_COLUMNS);
                let many_model = Columns::ManyModel {
                    a: columns.next().ok_or_else(f_err)?,
                    b: columns.next().ok_or_else(f_err)?,
                };
                columns.next().map_or_else(
                    || Ok(()),
                    |column| {
                        Err(syn::Error::new(
                            column.rs_name.span(),
                            TABLE_MUST_HAVE_TWO_COLUMNS,
                        ))
                    },
                )?;
                many_model
            }
        };

        Ok(Self {
            name,
            rs_name,
            columns,
            index_rs_name,
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
            name,
            rs_name,
            tables,
            rs_vis,
        })
    }
}

pub fn find_table<'a>(tables: &'a [Table], ident: &Ident) -> syn::Result<&'a Table> {
    tables
        .iter()
        .find(|table| &table.rs_name == ident)
        .ok_or_else(|| syn::Error::new(ident.span(), TABLE_DOES_NOT_EXIST))
}
