use super::stage1;

use crate::utils::{collections::TryCollectAll, multiplicity};

use syn::{Attribute, Expr, Ident, Pat, Type, Visibility, ext::IdentExt, spanned::Spanned};

const TABLE_MUST_HAVE_ID: &str = "table must have an ID";
const TABLE_MUST_NOT_HAVE_MULTIPLE_IDS: &str = "table must not have multiple IDs";
const TABLE_MUST_IMPLEMENT_MODEL: &str = "table must implement model to implement controller";
const TABLE_MUST_HAVE_TWO_COLUMNS: &str = "table must have two columns";
const TABLE_DOES_NOT_EXIST: &str = "table does not exist";
const MODEL_AND_MANY_MODEL_CONFLICT: &str = "model and many_model conflict";
const ID_MUST_BE_U64: &str = "id must be u64";
const COLUMN_MUST_BE_STRING: &str = "column must be string";
const COLUMN_MUST_BE_TIME: &str = "column must be time";
const COLUMN_MUST_NOT_BE_OPTIONAL: &str = "column must not be optional";
const COLUMN_MUST_BE_VEC: &str = "column must be Vec";
const COLUMN_MUST_SPECIFY_INTERMEDIATE_TABLE: &str = "column must specify intermediate table";

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

#[allow(non_camel_case_types)]
#[derive(Clone)]
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

#[derive(Clone)]
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
        use stage1::ColumnAttrTy as S1CAT;
        match attr_ty {
            Some(S1CAT::Compound(stage1::ColumnAttrTyCompound { many: None })) => {
                Self::Compound(CATC::One)
            }
            Some(S1CAT::Compound(stage1::ColumnAttrTyCompound { many: Some(many) })) => {
                Self::Compound(CATC::Many(many.0))
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
    Id,
    Value(TyElementValue),
    AutoTime(TyElementAutoTime),
}
impl TyElement {
    pub const fn optional(&self) -> bool {
        matches!(self, Self::Value(value) if value.optional)
    }
    pub const fn unique(&self) -> bool {
        match self {
            Self::Id => true,
            // TODO: unique
            Self::Value(_value) => false,
            Self::AutoTime(_) => false,
        }
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
    One { optional: bool },
    Many(Ident),
}
impl TyCompoundMultiplicity {
    pub const fn optional(&self) -> bool {
        matches!(*self, Self::One { optional } if optional)
    }
}

pub struct TyCompound {
    pub ty: Ident,
    pub multiplicity: TyCompoundMultiplicity,
}
impl TyCompound {
    pub const fn optional(&self) -> bool {
        self.multiplicity.optional()
    }
    pub const fn unique(&self) -> bool {
        // TODO: unique
        false
    }
    pub const fn max_len(&self) -> Option<u16> {
        None
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
    pub const fn default_value(&self) -> Option<DefaultValue<'_>> {
        match self {
            Self::Element(element) => element.default_value(),
            _ => None,
        }
    }
    pub const fn max_len(&self) -> Option<u16> {
        match self {
            Self::Compound(compound) => compound.max_len(),
            Self::Element(element) => element.max_len(),
        }
    }
}

pub struct ColumnAttrResponse {
    pub name: Option<String>,
    pub skip: bool,
}

pub enum ValidateRule {
    MinLen(Expr),
    MaxLen(usize),
    Func(Expr),
    Matches(Pat),
    NMatches(Pat),
    Eq(Expr),
    NEq(Expr),
    Gt(Expr),
    Lt(Expr),
    Gte(Expr),
    Lte(Expr),
}

pub struct ColumnAttrRequest {
    pub name: Option<String>,
    // pub ty: Option<Type>,
    // pub validate: Vec<ValidateRule>,
}

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
    pub attr_response: ColumnAttrResponse,
    /// the request attribute of the column
    pub attr_request: ColumnAttrRequest,
    pub validate: Vec<ValidateRule>,

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

        // the real type that we actually want to parse, while keeping the type in the field the same
        let real_rs_ty = attr
            .real_ty
            .as_deref()
            .map_or(&*rs_ty, |real_rs_ty| &real_rs_ty.0);

        let attr_ty = ColumnAttrTy::from(attr.ty);
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
                    (CATC::One, M::One) => TCM::One { optional: false },
                    (CATC::One, M::OneOrZero) => TCM::One { optional: true },
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
                    ty,
                    multiplicity: ty_compound_multiplicity,
                })
            }
            ColumnAttrTy::Element(attr_ty_element) => {
                use ColumnAttrTyElement as CATE;
                let ty_element_value = stage1::TyElementValue::try_from(real_rs_ty)?;
                let ty_element_value = TyElementValue::from(ty_element_value);
                match attr_ty_element {
                    CATE::None => Ty::Element(TyElement::Value(ty_element_value)),
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

        let validate = attr
            .attr_request
            .validate
            .0
            .into_iter()
            .map(|validate_rule| {
                use stage1::ValidateRule as S1VR;
                match validate_rule {
                    S1VR::MinLen(min_len) => ValidateRule::MinLen(min_len.0),
                    S1VR::Func(func) => ValidateRule::Func(func.0),
                    S1VR::Matches(matches) => ValidateRule::Matches(matches.0.0),
                    S1VR::NMatches(n_matches) => ValidateRule::NMatches(n_matches.0.0),
                    S1VR::Eq(eq) => ValidateRule::Eq(eq.0),
                    S1VR::NEq(n_eq) => ValidateRule::NEq(n_eq.0),
                    S1VR::Gt(gt) => ValidateRule::Gt(gt.0),
                    S1VR::Lt(lt) => ValidateRule::Lt(lt.0),
                    S1VR::Gte(gte) => ValidateRule::Gte(gte.0),
                    S1VR::Lte(lte) => ValidateRule::Lte(lte.0),
                }
            });
        let max_len_validate_rule = ty
            .max_len()
            .map(|max_len| ValidateRule::MaxLen(max_len.into()));
        let validate = validate.chain(max_len_validate_rule);
        let validate = validate.collect();

        let attr_response = ColumnAttrResponse {
            name: attr.attr_response.name,
            skip: attr.attr_response.skip,
        };
        let attr_request = ColumnAttrRequest {
            name: attr.attr_request.name,
        };

        Ok(Self {
            name,
            rs_name,
            ty,
            rs_ty,
            attr_response,
            attr_request,
            rs_attrs: attr.attrs,
            validate,
        })
    }
}

pub enum Columns<C> {
    CollectionOnly {
        columns: Vec<C>,
    },
    Model {
        id: C,
        columns: Vec<C>,
        controller: bool,
    },
    ManyModel {
        a: C,
        b: C,
    },
}
impl<C> Columns<C> {
    pub fn iter(&self) -> impl Iterator<Item = &C> + Clone {
        let (a, b, c) = match self {
            Self::CollectionOnly { columns } => (None, None, &**columns),
            Self::Model { id, columns, .. } => (Some(id), None, &**columns),
            Self::ManyModel { a, b } => (Some(a), Some(b), &[][..]),
        };
        a.into_iter().chain(b).chain(c)
    }
    pub fn map_try_collect_all_default<'a, F, C2, E>(&'a self, mut f: F) -> Result<Columns<C2>, E>
    where
        C: 'a,
        C2: 'a,
        F: FnMut(&'a C) -> Result<C2, E>,
        E: crate::utils::collections::Push<E>,
    {
        match self {
            Self::CollectionOnly { columns } => {
                let columns = columns.iter().map(f);
                let columns: Result<Vec<C2>, E> = columns.try_collect_all_default();
                let columns = columns?;
                Ok(Columns::CollectionOnly { columns })
            }
            Self::Model {
                id,
                columns,
                controller,
            } => {
                let id = f(id)?;
                let columns = columns.iter().map(f);
                let columns: Result<Vec<C2>, E> = columns.try_collect_all_default();
                let columns = columns?;
                Ok(Columns::Model {
                    id,
                    columns,
                    controller: *controller,
                })
            }
            Self::ManyModel { a, b } => {
                let a = f(a)?;
                let b = f(b)?;
                Ok(Columns::ManyModel { a, b })
            }
        }
    }
    pub fn model(&self) -> Option<&C> {
        match self {
            Self::Model { id, .. } => Some(id),
            _ => None,
        }
    }
    pub fn many_model(&self) -> Option<(&C, &C)> {
        match self {
            Self::ManyModel { a, b } => Some((a, b)),
            _ => None,
        }
    }
    pub fn is_collection(&self) -> bool {
        matches!(*self, Self::CollectionOnly { .. } | Self::Model { .. })
    }
    pub fn is_controller(&self) -> bool {
        matches!(*self, Self::Model { controller, .. } if controller)
    }
}

pub struct Table {
    /// the name for the sql table, for example `customers`
    pub name: String,
    /// the name for the table struct, for example `Customer`
    pub rs_name: Ident,
    /// the columns in the database
    pub columns: Columns<Column>,
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
                    model: model_attr,
                    controller: controller_attr,
                    many_model: many_model_attr,
                    name,
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
                if let Ty::Element(TyElement::Id) = column.ty {
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

        // TODO: allow model to have options or default options if only controller was specified (controller auto implements model too)
        // let model = model.or_else(|| controller.map(|_x| Default::default()));
        let model = model_attr.or(controller_attr);
        let model = match (model, id) {
            (Some(model), Some(id)) => Some((id, model)),
            (None, None) => None,
            (Some(model), None) => return Err(syn::Error::new(model.span(), TABLE_MUST_HAVE_ID)),
            (None, Some(ref id)) => {
                return Err(syn::Error::new(
                    id.rs_name.span(),
                    TABLE_MUST_IMPLEMENT_MODEL,
                ));
            }
        };

        let columns = match (model, many_model_attr) {
            (Some((_, model_attr)), Some(_)) => {
                return Err(syn::Error::new(
                    model_attr.span(),
                    MODEL_AND_MANY_MODEL_CONFLICT,
                ));
            }
            (Some((id, _)), None) => Columns::Model {
                id,
                columns,
                controller: controller_attr.is_some(),
            },
            (None, Some(_)) => {
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
            (None, None) => Columns::CollectionOnly { columns },
        };

        Ok(Self {
            name,
            rs_name,
            columns,
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

pub fn find_table<'a>(tables: &'a [Table], ident: &Ident) -> syn::Result<&'a Table> {
    tables
        .iter()
        .find(|table| &table.rs_name == ident)
        .ok_or_else(|| syn::Error::new(ident.span(), TABLE_DOES_NOT_EXIST))
}
