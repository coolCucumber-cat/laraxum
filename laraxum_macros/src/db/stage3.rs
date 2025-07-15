use super::stage2;

use crate::utils::borrow::DerefEither;
use crate::utils::collections::TryCollectAll;

use std::borrow::Cow;

use syn::{Attribute, Ident, Type, Visibility};

const TABLE_MUST_HAVE_ID: &str = "table must have an ID";

fn name_extern((parent, child): (&str, &str)) -> String {
    fmt2::fmt! { { str } => {parent} "__" {child} }
}
fn name_extern_triple((grandparent, parent, child): (&str, &str, &str)) -> String {
    fmt2::fmt! { { str } => {grandparent} "__" {parent} "__" {child} }
}
fn name_intern((parent, child): (&str, &str)) -> String {
    fmt2::fmt! { { str } => {parent} "." {child} }
}
fn name_intern_extern(parent_child: (&str, &str)) -> (String, String) {
    (name_intern(parent_child), name_extern(parent_child))
}

pub use stage2::{
    AtomicTy, AtomicTyString, AtomicTyTime, AutoTimeEvent, Columns, DefaultValue, TyElement,
    TyElementAutoTime,
};

pub struct TyCompound<'a> {
    pub foreign_table_name: &'a str,
    pub foreign_table_id_name: &'a str,
    pub optional: bool,
}
impl TyCompound<'_> {
    pub const fn optional(&self) -> bool {
        self.optional
    }
    pub const fn unique(&self) -> bool {
        // TODO: unique
        false
    }
}

pub enum Ty<'a> {
    Element(TyElement),
    Compound(TyCompound<'a>),
}
impl Ty<'_> {
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
}

pub struct CreateColumn<'a> {
    pub name: &'a str,
    pub ty: Ty<'a>,
}

pub struct ResponseColumnGetterElement<'a> {
    pub name_intern: String,
    pub name_extern: String,
    pub optional: bool,
    pub rs_name: &'a Ident,
}

pub struct ResponseColumnGetterCompound<'a> {
    pub name_intern: String,
    pub foreign_table_id_name_intern: String,
    pub foreign_table_name_intern: String,
    pub foreign_table_name_extern: String,
    pub optional: bool,
    pub rs_name: &'a Ident,
    pub foreign_table_rs_name: &'a Ident,
    pub columns: Vec<ResponseColumnGetter<'a>>,
}

pub enum ResponseColumnGetterOne<'a> {
    Element(ResponseColumnGetterElement<'a>),
    Compound(ResponseColumnGetterCompound<'a>),
}
impl ResponseColumnGetterOne<'_> {
    pub fn name_intern(&self) -> &str {
        match self {
            Self::Element(element) => &element.name_intern,
            Self::Compound(compound) => &compound.name_intern,
        }
    }
    pub fn rs_name(&self) -> &Ident {
        match self {
            Self::Element(element) => element.rs_name,
            Self::Compound(compound) => compound.rs_name,
        }
    }
    pub fn optional(&self) -> bool {
        match self {
            Self::Element(element) => element.optional,
            Self::Compound(compound) => compound.optional,
        }
    }
}

pub struct ResponseColumnGetterCompounds<'a> {
    pub rs_name: &'a Ident,
    pub table_rs_name: &'a Ident,
    pub table_id_name_extern: String,
    pub foreign_table_rs_name: &'a Ident,
    pub many_foreign_table_rs_name: &'a Ident,
}

pub enum ResponseColumnGetter<'a> {
    One(ResponseColumnGetterOne<'a>),
    Compounds(ResponseColumnGetterCompounds<'a>),
}
#[derive(Clone, Copy)]
pub enum ResponseColumnGetterRef<'a> {
    One(&'a ResponseColumnGetterOne<'a>),
    Compounds(&'a ResponseColumnGetterCompounds<'a>),
}
impl ResponseColumnGetterRef<'_> {
    pub fn rs_name(&self) -> &Ident {
        match self {
            Self::One(one) => one.rs_name(),
            Self::Compounds(compounds) => compounds.rs_name,
        }
    }
}
impl<'a> From<&'a ResponseColumnGetter<'a>> for ResponseColumnGetterRef<'a> {
    fn from(value: &'a ResponseColumnGetter<'a>) -> Self {
        match value {
            ResponseColumnGetter::One(one) => Self::One(one),
            ResponseColumnGetter::Compounds(compounds) => Self::Compounds(compounds),
        }
    }
}

pub struct ResponseColumnField<'a> {
    pub rs_name: &'a Ident,
    pub rs_ty: &'a Type,
    pub attr: &'a stage2::ColumnAttrResponse,
    pub rs_attrs: &'a [Attribute],
}

pub struct ResponseColumnOne<'a> {
    pub field: ResponseColumnField<'a>,
    pub getter: ResponseColumnGetterOne<'a>,
}

pub struct ResponseColumnCompounds<'a> {
    pub field: ResponseColumnField<'a>,
    pub getter: ResponseColumnGetterCompounds<'a>,
}

pub struct RequestColumnSetterOne<'a> {
    pub rs_name: &'a Ident,
    pub name: &'a str,
    pub optional: bool,
    pub validate: &'a [stage2::ValidateRule],
}

pub struct RequestColumnSetterCompounds<'a> {
    pub rs_name: &'a Ident,
    pub table_rs_name: &'a Ident,
    // pub table_id_rs_name: &'a Ident,
    pub foreign_table_rs_name: &'a Ident,
    pub many_foreign_table_rs_name: &'a Ident,
}

pub struct RequestColumnField<'a> {
    pub rs_name: &'a Ident,
    pub rs_ty: DerefEither<Type, &'a Type, Box<Type>>,
    pub attr: &'a stage2::ColumnAttrRequest,
    pub rs_attrs: &'a [Attribute],
}

pub enum RequestColumnOne<'a> {
    Some {
        field: RequestColumnField<'a>,
        setter: RequestColumnSetterOne<'a>,
    },
    AutoTime {
        name: &'a str,
        time_ty: stage2::AtomicTyTime,
    },
    None,
}

pub struct RequestColumnCompounds<'a> {
    pub field: RequestColumnField<'a>,
    pub setter: RequestColumnSetterCompounds<'a>,
}

pub struct ColumnOne<'a> {
    pub create: CreateColumn<'a>,
    pub response: ResponseColumnOne<'a>,
    pub request: RequestColumnOne<'a>,
}
impl ColumnOne<'_> {
    pub fn name(&self) -> &str {
        self.create.name
    }
    pub fn name_intern(&self) -> &str {
        self.response.getter.name_intern()
    }
}

pub struct ColumnCompounds<'a> {
    pub response: ResponseColumnCompounds<'a>,
    pub request: RequestColumnCompounds<'a>,
}

pub enum Column<'a> {
    One(ColumnOne<'a>),
    Compounds(ColumnCompounds<'a>),
}

impl Column<'_> {
    pub fn create(&self) -> Option<&CreateColumn<'_>> {
        match self {
            Self::One(one) => Some(&one.create),
            Self::Compounds(_) => None,
        }
    }
    pub fn response_field(&self) -> &ResponseColumnField<'_> {
        match self {
            Self::One(one) => &one.response.field,
            Self::Compounds(compounds) => &compounds.response.field,
        }
    }
    pub fn response_getter(&self) -> ResponseColumnGetterRef<'_> {
        match self {
            Self::One(one) => ResponseColumnGetterRef::One(&one.response.getter),
            Self::Compounds(compounds) => {
                ResponseColumnGetterRef::Compounds(&compounds.response.getter)
            }
        }
    }
    pub fn request_field(&self) -> Option<&RequestColumnField<'_>> {
        match self {
            Self::One(ColumnOne {
                request: RequestColumnOne::Some { field, .. },
                ..
            }) => Some(field),
            Self::Compounds(compounds) => Some(&compounds.request.field),
            _ => None,
        }
    }
    pub fn request_one(&self) -> Option<&RequestColumnOne<'_>> {
        match self {
            Self::One(one) => Some(&one.request),
            Self::Compounds(_) => None,
        }
    }
    pub fn request_compounds(&self) -> Option<&RequestColumnCompounds<'_>> {
        match self {
            Self::One(_) => None,
            Self::Compounds(compounds) => Some(&compounds.request),
        }
    }
}

pub struct Table<'a> {
    pub name_intern: String,
    pub name_extern: String,
    pub rs_name: &'a Ident,
    pub request_rs_name: Cow<'a, Ident>,
    pub request_error_rs_name: Cow<'a, Ident>,
    pub db_rs_name: &'a Ident,
    pub rs_attrs: &'a [syn::Attribute],
    pub columns: Columns<Column<'a>>,
}

impl<'a> Table<'a> {
    fn try_new(table: &'a stage2::Table, db: &'a stage2::Db) -> syn::Result<Self> {
        fn rs_ty_compound_request(optional: bool) -> Type {
            if optional {
                syn::parse_quote!(Option<u64>)
            } else {
                syn::parse_quote!(u64)
            }
        }
        fn rs_ty_compounds_request() -> Type {
            syn::parse_quote!(Vec<u64>)
        }

        fn traverse<'table: 'iter, 'iter>(
            table_name_extern: &str,
            table: &'table stage2::Table,
            db: &'iter stage2::Db,
        ) -> impl Iterator<Item = syn::Result<ResponseColumnGetter<'iter>>> {
            table.columns.iter().map(move |column| {
                let stage2::Column {
                    name, rs_name, ty, ..
                } = column;
                let name = &**name;
                let (column_name_intern, column_name_extern) =
                    name_intern_extern((table_name_extern, name));

                let response_getter_column = match ty {
                    stage2::Ty::Element(ty_element) => {
                        let element = ResponseColumnGetterElement {
                            name_intern: column_name_intern,
                            name_extern: column_name_extern,
                            rs_name,
                            optional: ty_element.optional(),
                        };
                        ResponseColumnGetter::One(ResponseColumnGetterOne::Element(element))
                    }
                    stage2::Ty::Compound(stage2::TyCompound {
                        ty: foreign_table_rs_name,
                        multiplicity: stage2::TyCompoundMultiplicity::One { optional },
                    }) => {
                        let foreign_table = stage2::find_table(&db.tables, foreign_table_rs_name)?;
                        let foreign_table_id = foreign_table.columns.model().ok_or_else(|| {
                            syn::Error::new(foreign_table.rs_name.span(), TABLE_MUST_HAVE_ID)
                        })?;

                        let foreign_table_name_intern =
                            name_intern((&db.name, &foreign_table.name));
                        let foreign_table_name_extern =
                            name_extern_triple((table_name_extern, &foreign_table.name, name));
                        let foreign_table_id_name_intern =
                            name_intern((&*foreign_table_name_extern, &foreign_table_id.name));

                        let columns = traverse(&foreign_table_name_extern, foreign_table, db);
                        // let columns = traverse(table_name_extern, table, db);
                        let columns: Result<Vec<ResponseColumnGetter>, syn::Error> =
                            columns.try_collect_all_default();
                        let columns = columns?;

                        let compound = ResponseColumnGetterCompound {
                            name_intern: column_name_intern,
                            foreign_table_id_name_intern,
                            foreign_table_name_intern,
                            foreign_table_name_extern,
                            rs_name,
                            foreign_table_rs_name: &foreign_table.rs_name,
                            optional: *optional,
                            columns,
                        };
                        ResponseColumnGetter::One(ResponseColumnGetterOne::Compound(compound))
                    }
                    stage2::Ty::Compound(stage2::TyCompound {
                        ty: foreign_table_rs_name,
                        multiplicity:
                            stage2::TyCompoundMultiplicity::Many(many_foreign_table_rs_name),
                    }) => {
                        let table_rs_name = &table.rs_name;
                        let table_id = table.columns.model().ok_or_else(|| {
                            syn::Error::new(table_rs_name.span(), TABLE_MUST_HAVE_ID)
                        })?;
                        let table_id_name_extern = name_extern((table_name_extern, &table_id.name));
                        ResponseColumnGetter::Compounds(ResponseColumnGetterCompounds {
                            rs_name,
                            table_rs_name,
                            table_id_name_extern,
                            foreign_table_rs_name,
                            many_foreign_table_rs_name,
                        })
                    }
                };
                Ok(response_getter_column)
            })
        }

        let (table_name_intern, table_name_extern) = name_intern_extern((&*db.name, &*table.name));
        let columns = table.columns.map_try_collect_all_default(
            |column: &stage2::Column| -> Result<Column, syn::Error> {
                let stage2::Column {
                    name,
                    rs_name,
                    ty,
                    rs_ty,
                    attr_response,
                    attr_request,
                    rs_attrs,
                    validate,
                } = column;
                let (column_name_intern, column_name_extern) =
                    name_intern_extern((&*table_name_extern, name));

                let column0 = match ty {
                    stage2::Ty::Element(ty_element) => Column::One(ColumnOne {
                        create: CreateColumn {
                            name,
                            ty: Ty::Element(ty_element.clone()),
                        },
                        response: ResponseColumnOne {
                            getter: ResponseColumnGetterOne::Element(ResponseColumnGetterElement {
                                name_intern: column_name_intern,
                                name_extern: column_name_extern,
                                optional: ty_element.optional(),
                                rs_name,
                            }),
                            field: ResponseColumnField {
                                rs_name,
                                rs_ty,
                                attr: attr_response,
                                rs_attrs,
                            },
                        },
                        request: match ty_element {
                            TyElement::Value(_) => RequestColumnOne::Some {
                                field: RequestColumnField {
                                    rs_name,
                                    rs_ty: DerefEither::Left(rs_ty),
                                    attr: attr_request,
                                    rs_attrs,
                                },
                                setter: RequestColumnSetterOne {
                                    rs_name,
                                    name,
                                    optional: ty_element.optional(),
                                    validate,
                                },
                            },
                            TyElement::AutoTime(TyElementAutoTime {
                                event: AutoTimeEvent::OnUpdate,
                                ty,
                            }) => RequestColumnOne::AutoTime {
                                name,
                                time_ty: ty.clone(),
                            },
                            TyElement::AutoTime(_) | TyElement::Id => RequestColumnOne::None,
                        },
                    }),
                    stage2::Ty::Compound(stage2::TyCompound {
                        ty: foreign_table_rs_name,
                        multiplicity: stage2::TyCompoundMultiplicity::One { optional },
                    }) => {
                        let foreign_table = stage2::find_table(&db.tables, foreign_table_rs_name)?;
                        let foreign_table_id = foreign_table.columns.model().ok_or_else(|| {
                            syn::Error::new(foreign_table.rs_name.span(), TABLE_MUST_HAVE_ID)
                        })?;

                        let foreign_table_name_intern =
                            name_intern((&db.name, &foreign_table.name));
                        let foreign_table_name_extern =
                            name_extern_triple((&table_name_extern, &foreign_table.name, name));
                        let foreign_table_id_name_intern =
                            name_intern((&*foreign_table_name_extern, &foreign_table_id.name));

                        let columns = traverse(&foreign_table_name_extern, foreign_table, db);
                        // let columns = traverse(table_name_extern, table, db);
                        let columns: Result<Vec<ResponseColumnGetter>, syn::Error> =
                            columns.try_collect_all_default();
                        let columns = columns?;

                        let compound = ResponseColumnGetterCompound {
                            name_intern: column_name_intern,
                            foreign_table_id_name_intern,
                            foreign_table_name_intern,
                            foreign_table_name_extern,
                            rs_name,
                            foreign_table_rs_name: &foreign_table.rs_name,
                            optional: *optional,
                            columns,
                        };

                        Column::One(ColumnOne {
                            create: CreateColumn {
                                name,
                                ty: Ty::Compound(TyCompound {
                                    foreign_table_name: &foreign_table.name,
                                    foreign_table_id_name: &foreign_table_id.name,
                                    optional: *optional,
                                }),
                            },
                            response: ResponseColumnOne {
                                field: ResponseColumnField {
                                    rs_name,
                                    rs_ty,
                                    attr: attr_response,
                                    rs_attrs,
                                },
                                getter: ResponseColumnGetterOne::Compound(compound),
                            },
                            request: RequestColumnOne::Some {
                                field: RequestColumnField {
                                    rs_name,
                                    rs_ty: DerefEither::Right(Box::new(rs_ty_compound_request(
                                        *optional,
                                    ))),
                                    attr: attr_request,
                                    rs_attrs,
                                },
                                setter: RequestColumnSetterOne {
                                    rs_name,
                                    name,
                                    optional: *optional,
                                    validate,
                                },
                            },
                        })
                    }
                    stage2::Ty::Compound(stage2::TyCompound {
                        ty: foreign_table_rs_name,
                        multiplicity:
                            stage2::TyCompoundMultiplicity::Many(many_foreign_table_rs_name),
                    }) => {
                        let table_rs_name = &table.rs_name;
                        let table_id = table.columns.model().ok_or_else(|| {
                            syn::Error::new(table_rs_name.span(), TABLE_MUST_HAVE_ID)
                        })?;
                        let table_id_name_extern =
                            name_extern((&table_name_extern, &table_id.name));
                        let compounds_getter = ResponseColumnGetterCompounds {
                            rs_name,
                            table_rs_name,
                            table_id_name_extern,
                            foreign_table_rs_name,
                            many_foreign_table_rs_name,
                        };
                        let compounds_setter = RequestColumnSetterCompounds {
                            rs_name,
                            table_rs_name,
                            // table_id_rs_name: &table_id.rs_name,
                            foreign_table_rs_name,
                            many_foreign_table_rs_name,
                        };

                        Column::Compounds(ColumnCompounds {
                            response: ResponseColumnCompounds {
                                field: ResponseColumnField {
                                    rs_name,
                                    rs_ty,
                                    attr: attr_response,
                                    rs_attrs,
                                },
                                getter: compounds_getter,
                            },
                            request: RequestColumnCompounds {
                                field: RequestColumnField {
                                    rs_name,
                                    rs_ty: DerefEither::Right(Box::new(rs_ty_compounds_request())),
                                    attr: attr_request,
                                    rs_attrs,
                                },
                                setter: compounds_setter,
                            },
                        })
                    }
                };
                Ok(column0)
            },
        );
        // let columns: Result<Vec<Column>, syn::Error> = columns.try_collect_all_default();
        let columns = columns?;

        let table_request_rs_name = quote::format_ident!("{}Request", table.rs_name);
        let table_request_error_rs_name = quote::format_ident!("{}RequestError", table.rs_name);
        let table_rs_attrs = &*table.rs_attrs;
        Ok(Self {
            name_intern: table_name_intern,
            name_extern: table_name_extern,
            rs_name: &table.rs_name,
            request_rs_name: Cow::Owned(table_request_rs_name),
            request_error_rs_name: Cow::Owned(table_request_error_rs_name),
            db_rs_name: &db.rs_name,
            rs_attrs: table_rs_attrs,
            columns,
        })
    }
}

pub struct Db<'a> {
    /// the name of the database
    pub name: &'a str,
    /// the name for the database module, for example `db`
    pub rs_name: &'a Ident,
    /// the tables in the database
    pub tables: Vec<Table<'a>>,
    /// visibility
    pub rs_vis: &'a Visibility,
}
// pub struct Db<'a> {
//     /// the name of the database
//     pub name: String,
//     /// the name for the database module, for example `db`
//     pub rs_name: Ident,
//     /// the tables in the database
//     pub tables: Vec<Table<'a>>,
//     /// visibility
//     pub rs_vis: Visibility,
// }

impl<'a> TryFrom<&'a stage2::Db> for Db<'a> {
    type Error = syn::Error;
    fn try_from(db: &'a stage2::Db) -> Result<Self, Self::Error> {
        let tables: Result<Vec<Table>, syn::Error> = db
            .tables
            .iter()
            .map(|table| Table::try_new(table, db))
            .try_collect_all_default();
        let tables = tables?;

        Ok(Self {
            name: &db.name,
            rs_name: &db.rs_name,
            tables,
            rs_vis: &db.rs_vis,
        })
    }
}
