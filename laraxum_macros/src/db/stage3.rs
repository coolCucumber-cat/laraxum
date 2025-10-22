use super::stage2;

pub use stage2::{
    AtomicTy, AtomicTyFloat, AtomicTyInt, AtomicTyString, AtomicTyTime, AutoTimeEvent, Columns,
    DefaultValue, TyElementAutoTime,
};

use crate::utils::{borrow::DerefEither, collections::TryCollectAll};

use std::borrow::Cow;

use syn::{Attribute, Ident, Type, Visibility};

const TABLE_MUST_HAVE_ID: &str = "table must have an ID";
const TABLE_ID_MUST_BE_INT: &str = "table ID must be int";
const COLUMN_MUST_NOT_BE_COMPOUNDS: &str = "column must not be many-to-many relationship";
// const COLUMN_MUST_HAVE_STRUCT_NAME: &str = "column must have struct name";

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

pub use stage2::TyElement;

pub struct TyCompound<'a> {
    pub foreign_table_name: &'a str,
    pub foreign_table_id_name: &'a str,
    pub ty: &'a AtomicTyInt,
    pub is_optional: bool,
    pub is_unique: bool,
}
impl TyCompound<'_> {
    pub const fn is_optional(&self) -> bool {
        self.is_optional
    }
    pub const fn is_unique(&self) -> bool {
        self.is_unique
    }
}

pub enum Ty<'a> {
    Element(TyElement),
    Compound(TyCompound<'a>),
}
impl Ty<'_> {
    pub const fn is_optional(&self) -> bool {
        match self {
            Self::Compound(compound) => compound.is_optional(),
            Self::Element(element) => element.is_optional(),
        }
    }
    pub const fn is_unique(&self) -> bool {
        match self {
            Self::Compound(compound) => compound.is_unique(),
            Self::Element(element) => element.is_unique(),
        }
    }
    pub const fn default_value(&self) -> Option<DefaultValue<'_>> {
        match self {
            Self::Compound(_) => None,
            Self::Element(element) => element.default_value(),
        }
    }
    pub const fn is_id(&self) -> bool {
        matches!(self, Self::Element(element) if element.is_id())
    }
}

pub struct CreateColumn<'a> {
    pub name: &'a str,
    pub ty: Ty<'a>,
}

pub struct ResponseColumnGetterElement<'a> {
    pub name_intern: String,
    pub name_extern: String,
    pub is_optional: bool,
    pub rs_name: &'a Ident,
    // pub rs_ty: &'a Type,
}

pub struct ResponseColumnGetterCompound<'a> {
    pub name_intern: String,
    pub foreign_table_id_name_intern: String,
    pub foreign_table_name_intern: String,
    pub foreign_table_name_extern: String,
    pub is_optional: bool,
    pub rs_name: &'a Ident,
    // pub rs_ty: &'a Type,
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
    pub const fn rs_name(&self) -> &Ident {
        match self {
            Self::Element(element) => element.rs_name,
            Self::Compound(compound) => compound.rs_name,
        }
    }
    pub const fn is_optional(&self) -> bool {
        match self {
            Self::Element(element) => element.is_optional,
            Self::Compound(compound) => compound.is_optional,
        }
    }
}

pub struct ResponseColumnGetterCompounds<'a> {
    pub rs_name: &'a Ident,
    pub index_rs_name: &'a Ident,
    pub table_id_name_extern: String,
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
    pub const fn rs_name(&self) -> &Ident {
        match self {
            Self::One(one) => one.rs_name(),
            Self::Compounds(compounds) => compounds.rs_name,
        }
    }
}
impl<'a> From<&'a ResponseColumnGetter<'a>> for ResponseColumnGetterRef<'a> {
    fn from(column: &'a ResponseColumnGetter<'a>) -> Self {
        match *column {
            ResponseColumnGetter::One(ref one) => Self::One(one),
            ResponseColumnGetter::Compounds(ref compounds) => Self::Compounds(compounds),
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

pub use stage2::Validate;

pub struct RequestColumnSetterOne<'a> {
    pub rs_name: &'a Ident,
    pub name: &'a str,
    pub is_optional: bool,
    pub validate: &'a Validate,
    pub is_mut: bool,
}

pub struct RequestColumnSetterCompounds<'a> {
    pub rs_name: &'a Ident,
    pub index_rs_name: &'a Ident,
    pub many_foreign_table_rs_name: &'a Ident,
}

pub struct RequestColumnField<'a> {
    pub rs_name: &'a Ident,
    pub rs_ty: DerefEither<Type, &'a Type, Box<Type>>,
    pub is_mut: bool,
    pub attr: &'a stage2::ColumnAttrRequest,
    pub rs_attrs: &'a [Attribute],
}

pub enum RequestColumnOne<'a> {
    Field {
        field: RequestColumnField<'a>,
        setter: RequestColumnSetterOne<'a>,
    },
    OnUpdate {
        name: &'a str,
        time_ty: stage2::AtomicTyTime,
    },
}
impl RequestColumnOne<'_> {
    pub const fn request_field(&self) -> Option<&RequestColumnField<'_>> {
        match self {
            Self::Field { field, setter: _ } => Some(field),
            _ => None,
        }
    }
    pub const fn request_setter(&self) -> Option<&RequestColumnSetterOne<'_>> {
        match self {
            Self::Field { setter, field: _ } => Some(setter),
            _ => None,
        }
    }
    pub const fn is_mut(&self) -> bool {
        match self {
            Self::Field { field, .. } => field.is_mut,
            Self::OnUpdate { .. } => true,
        }
    }
}

pub struct RequestColumnCompounds<'a> {
    pub field: RequestColumnField<'a>,
    pub setter: RequestColumnSetterCompounds<'a>,
}

pub use stage2::ColumnAttrIndex;

pub use stage2::ColumnAttrIndexFilter;

pub use stage2::ColumnAttrIndexLimit;

pub struct ColumnOne<'a> {
    pub create: CreateColumn<'a>,
    pub response: ResponseColumnOne<'a>,
    pub request: Option<RequestColumnOne<'a>>,
    pub index: &'a [ColumnAttrIndex],
    pub borrow: Option<Option<&'a Type>>,
    pub struct_name: Option<&'a Ident>,
}
impl ColumnOne<'_> {
    pub const fn name(&self) -> &str {
        self.create.name
    }
    pub fn name_intern(&self) -> &str {
        self.response.getter.name_intern()
    }
}
impl<'a> TryFrom<Column<'a>> for ColumnOne<'a> {
    type Error = syn::Error;
    fn try_from(column: Column<'a>) -> Result<Self, Self::Error> {
        match column {
            Column::One(one) => Ok(one),
            Column::Compounds(compounds) => Err(syn::Error::new(
                compounds.response.field.rs_name.span(),
                COLUMN_MUST_NOT_BE_COMPOUNDS,
            )),
        }
    }
}

pub struct ColumnCompounds<'a> {
    pub response: ResponseColumnCompounds<'a>,
    pub request: RequestColumnCompounds<'a>,
    pub struct_name: Option<&'a Ident>,
}

pub enum Column<'a> {
    One(ColumnOne<'a>),
    Compounds(ColumnCompounds<'a>),
}

#[derive(Clone, Copy)]
pub enum ColumnRef<'a> {
    One(&'a ColumnOne<'a>),
    Compounds(&'a ColumnCompounds<'a>),
}
impl<'a> ColumnRef<'a> {
    pub const fn create(self) -> Option<&'a CreateColumn<'a>> {
        match self {
            Self::One(one) => Some(&one.create),
            Self::Compounds(_) => None,
        }
    }
    pub const fn response_field(self) -> &'a ResponseColumnField<'a> {
        match self {
            Self::One(one) => &one.response.field,
            Self::Compounds(compounds) => &compounds.response.field,
        }
    }
    pub const fn response_getter(self) -> ResponseColumnGetterRef<'a> {
        match self {
            Self::One(one) => ResponseColumnGetterRef::One(&one.response.getter),
            Self::Compounds(compounds) => {
                ResponseColumnGetterRef::Compounds(&compounds.response.getter)
            }
        }
    }
    pub fn request_field(self) -> Option<&'a RequestColumnField<'a>> {
        match self {
            Self::One(one) => one
                .request
                .as_ref()
                .and_then(|request| request.request_field()),
            Self::Compounds(compounds) => Some(&compounds.request.field),
        }
    }
    pub const fn request_one(self) -> Option<&'a RequestColumnOne<'a>> {
        match self {
            Self::One(one) => one.request.as_ref(),
            Self::Compounds(_) => None,
        }
    }
    pub const fn request_compounds(self) -> Option<&'a RequestColumnCompounds<'a>> {
        match self {
            Self::One(_) => None,
            Self::Compounds(compounds) => Some(&compounds.request),
        }
    }
    pub fn request_one_setter(self) -> Option<&'a RequestColumnSetterOne<'a>> {
        match self {
            Self::One(one) => one
                .request
                .as_ref()
                .and_then(|request| request.request_setter()),
            Self::Compounds(_) => None,
        }
    }
    pub const fn request_compounds_settter(self) -> Option<&'a RequestColumnSetterCompounds<'a>> {
        match self {
            Self::One(_) => None,
            Self::Compounds(compounds) => Some(&compounds.request.setter),
        }
    }
    pub const fn struct_name(self) -> Option<&'a Ident> {
        match self {
            Self::One(one) => one.struct_name,
            Self::Compounds(compounds) => compounds.struct_name,
        }
    }
    // pub const fn is_mut(&self)->bool{
    //     match self {
    //         Self::One(one)=>one.is
    //     }
    // }
}
impl<'a> From<&'a Column<'a>> for ColumnRef<'a> {
    fn from(column: &'a Column<'a>) -> Self {
        match *column {
            Column::One(ref one) => ColumnRef::One(one),
            Column::Compounds(ref compounds) => ColumnRef::Compounds(compounds),
        }
    }
}

impl<T, C> Columns<T, T, C> {
    pub fn map_try_collect_all_default<'a, F>(
        &'a self,
        mut f: F,
    ) -> Result<Columns<Column<'a>, ColumnOne<'a>, &'a C>, syn::Error>
    where
        F: FnMut(&'a T) -> Result<Column<'a>, syn::Error>,
    {
        match self {
            Self::CollectionOnly { columns } => {
                let columns = columns.iter().map(|c| f(c));
                let columns: Result<Vec<Column<'a>>, syn::Error> =
                    columns.try_collect_all_default();
                let columns = columns?;
                Ok(Columns::CollectionOnly { columns })
            }
            Self::Model {
                id,
                columns,
                controller,
            } => {
                let id = f(id)?;
                let id = ColumnOne::try_from(id)?;
                let columns = columns.iter().map(f);
                let columns: Result<Vec<Column<'a>>, syn::Error> =
                    columns.try_collect_all_default();
                let columns = columns?;
                Ok(Columns::Model {
                    id,
                    columns,
                    controller: controller.as_ref(),
                })
            }
            Self::ManyModel { a, b } => {
                let a = f(a)?;
                let a = ColumnOne::try_from(a)?;
                let b = f(b)?;
                let b = ColumnOne::try_from(b)?;
                Ok(Columns::ManyModel { a, b })
            }
        }
    }
}
impl<'a, C> Columns<Column<'a>, ColumnOne<'a>, C> {
    pub fn iter(&'a self) -> impl Iterator<Item = ColumnRef<'a>> + Clone {
        let (a, b, c) = match self {
            Self::CollectionOnly { columns } => (None, None, columns.iter().map(ColumnRef::from)),
            Self::Model { id, columns, .. } => (
                Some(ColumnRef::One(id)),
                None,
                columns.iter().map(ColumnRef::from),
            ),
            Self::ManyModel { a, b } => (
                Some(ColumnRef::One(a)),
                Some(ColumnRef::One(b)),
                [].iter().map(ColumnRef::from),
            ),
        };
        a.into_iter().chain(b).chain(c)
    }
}

pub struct Table<'a> {
    pub name_intern: String,
    pub name_extern: String,
    pub rs_name: &'a Ident,
    pub create_request_rs_name: Cow<'a, Ident>,
    pub update_request_rs_name: Cow<'a, Ident>,
    pub patch_request_rs_name: Cow<'a, Ident>,
    pub request_error_rs_name: Cow<'a, Ident>,
    pub index_rs_name: Option<&'a Ident>,
    pub db_rs_name: &'a Ident,
    pub rs_attrs: &'a [syn::Attribute],
    pub columns: Columns<Column<'a>, ColumnOne<'a>, &'a stage2::TableAttrController>,
}

impl<'a> Table<'a> {
    #[expect(clippy::too_many_lines)]
    fn try_new(table: &'a stage2::Table, db: &'a stage2::Db) -> syn::Result<Self> {
        fn rs_ty_compound_request(
            rs_ty: &Type,
            is_optional: bool,
        ) -> DerefEither<Type, &Type, Box<Type>> {
            if is_optional {
                DerefEither::Right(Box::new(syn::parse_quote!(Option<#rs_ty>)))
            } else {
                DerefEither::Left(rs_ty)
            }
        }
        fn rs_ty_compounds_request(
            many_foreign_table_rs_name: &Ident,
            index_rs_name: &Ident,
        ) -> Type {
            syn::parse_quote!(
                Vec<
                    <
                        #many_foreign_table_rs_name as ::laraxum::ManyModel<#index_rs_name>
                    >::ManyRequest
                >
            )
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
                let name = &*name;
                let (column_name_intern, column_name_extern) =
                    name_intern_extern((table_name_extern, name));

                let response_getter_column = match *ty {
                    stage2::Ty::Element(ref ty_element) => {
                        let element = ResponseColumnGetterElement {
                            name_intern: column_name_intern,
                            name_extern: column_name_extern,
                            rs_name,
                            is_optional: ty_element.is_optional(),
                        };
                        ResponseColumnGetter::One(ResponseColumnGetterOne::Element(element))
                    }
                    stage2::Ty::Compound(stage2::TyCompound {
                        rs_ty_name: ref foreign_table_rs_name,
                        multiplicity:
                            stage2::TyCompoundMultiplicity::One {
                                is_optional,
                                is_unique: _,
                            },
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
                            is_optional,
                            columns,
                        };
                        ResponseColumnGetter::One(ResponseColumnGetterOne::Compound(compound))
                    }
                    stage2::Ty::Compound(stage2::TyCompound {
                        rs_ty_name: _,
                        multiplicity:
                            stage2::TyCompoundMultiplicity::Many(stage2::ColumnAttrTyCompounds {
                                model_rs_name: ref many_foreign_table_rs_name,
                                index_rs_ty: ref index,
                            }),
                    }) => {
                        let table_rs_name = &table.rs_name;
                        let index_rs_name = index.as_ref().unwrap_or(table_rs_name);
                        let table_id = table.columns.model().ok_or_else(|| {
                            syn::Error::new(table_rs_name.span(), TABLE_MUST_HAVE_ID)
                        })?;
                        let table_id_name_extern = name_extern((table_name_extern, &table_id.name));
                        ResponseColumnGetter::Compounds(ResponseColumnGetterCompounds {
                            rs_name,
                            index_rs_name,
                            table_id_name_extern,
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
                    ref name,
                    ref rs_name,
                    ref ty,
                    ref rs_ty,
                    ref response,
                    ref request,
                    is_mut,
                    ref borrow,
                    ref index,
                    ref struct_name,
                    ref rs_attrs,
                } = *column;
                let (column_name_intern, column_name_extern) =
                    name_intern_extern((&*table_name_extern, name));
                let index = &**index;
                let borrow = borrow.as_ref().map(Option::as_deref);
                let struct_name = struct_name.as_ref();

                let column0 = match *ty {
                    stage2::Ty::Element(ref ty_element) => Column::One(ColumnOne {
                        create: CreateColumn {
                            name,
                            ty: Ty::Element(ty_element.clone()),
                        },
                        response: ResponseColumnOne {
                            getter: ResponseColumnGetterOne::Element(ResponseColumnGetterElement {
                                name_intern: column_name_intern,
                                name_extern: column_name_extern,
                                is_optional: ty_element.is_optional(),
                                rs_name,
                            }),
                            field: ResponseColumnField {
                                rs_name,
                                rs_ty,
                                attr: response,
                                rs_attrs,
                            },
                        },
                        request: match ty_element {
                            TyElement::Value(_) => Some(RequestColumnOne::Field {
                                field: RequestColumnField {
                                    rs_name,
                                    rs_ty: DerefEither::Left(rs_ty),
                                    attr: request,
                                    rs_attrs,
                                    is_mut,
                                },
                                setter: RequestColumnSetterOne {
                                    rs_name,
                                    name,
                                    is_optional: ty_element.is_optional(),
                                    validate: &request.validate,
                                    is_mut,
                                },
                            }),
                            TyElement::AutoTime(TyElementAutoTime {
                                event: AutoTimeEvent::OnUpdate,
                                ty,
                            }) => Some(RequestColumnOne::OnUpdate {
                                name,
                                time_ty: ty.clone(),
                            }),
                            TyElement::AutoTime(_) | TyElement::Id(_) => None,
                        },
                        index,
                        borrow,
                        struct_name,
                    }),
                    stage2::Ty::Compound(stage2::TyCompound {
                        rs_ty_name: ref foreign_table_rs_name,
                        multiplicity:
                            stage2::TyCompoundMultiplicity::One {
                                is_optional,
                                is_unique,
                            },
                    }) => {
                        let foreign_table = stage2::find_table(&db.tables, foreign_table_rs_name)?;
                        let foreign_table_id = foreign_table.columns.model().ok_or_else(|| {
                            syn::Error::new(foreign_table.rs_name.span(), TABLE_MUST_HAVE_ID)
                        })?;
                        let foreign_table_id_ty = foreign_table_id.ty.id().ok_or_else(|| {
                            syn::Error::new(foreign_table.rs_name.span(), TABLE_ID_MUST_BE_INT)
                        })?;
                        let foreign_table_id_rs_ty = &*foreign_table_id.rs_ty;

                        let foreign_table_name_intern =
                            name_intern((&db.name, &foreign_table.name));
                        let foreign_table_name_extern =
                            name_extern_triple((&table_name_extern, &foreign_table.name, name));
                        let foreign_table_id_name_intern =
                            name_intern((&*foreign_table_name_extern, &foreign_table_id.name));

                        let columns = traverse(&foreign_table_name_extern, foreign_table, db);
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
                            is_optional,
                            columns,
                        };

                        Column::One(ColumnOne {
                            create: CreateColumn {
                                name,
                                ty: Ty::Compound(TyCompound {
                                    foreign_table_name: &foreign_table.name,
                                    foreign_table_id_name: &foreign_table_id.name,
                                    ty: foreign_table_id_ty,
                                    is_optional,
                                    is_unique,
                                }),
                            },
                            response: ResponseColumnOne {
                                field: ResponseColumnField {
                                    rs_name,
                                    rs_ty,
                                    attr: response,
                                    rs_attrs,
                                },
                                getter: ResponseColumnGetterOne::Compound(compound),
                            },
                            request: Some(RequestColumnOne::Field {
                                field: RequestColumnField {
                                    rs_name,
                                    rs_ty: rs_ty_compound_request(
                                        foreign_table_id_rs_ty,
                                        is_optional,
                                    ),
                                    attr: request,
                                    rs_attrs,
                                    is_mut,
                                },
                                setter: RequestColumnSetterOne {
                                    rs_name,
                                    name,
                                    is_optional,
                                    validate: &request.validate,
                                    is_mut,
                                },
                            }),
                            index,
                            borrow,
                            struct_name,
                        })
                    }
                    stage2::Ty::Compound(stage2::TyCompound {
                        rs_ty_name: _,
                        multiplicity:
                            stage2::TyCompoundMultiplicity::Many(stage2::ColumnAttrTyCompounds {
                                model_rs_name: ref many_foreign_table_rs_name,
                                index_rs_ty: ref index,
                            }),
                    }) => {
                        let table_rs_name = &table.rs_name;
                        let index_rs_name = index.as_ref().unwrap_or(table_rs_name);
                        let table_id = table.columns.model().ok_or_else(|| {
                            syn::Error::new(table_rs_name.span(), TABLE_MUST_HAVE_ID)
                        })?;
                        let table_id_name_extern =
                            name_extern((&table_name_extern, &table_id.name));
                        let compounds_getter = ResponseColumnGetterCompounds {
                            rs_name,
                            index_rs_name,
                            table_id_name_extern,
                            many_foreign_table_rs_name,
                        };
                        let compounds_setter = RequestColumnSetterCompounds {
                            rs_name,
                            index_rs_name,
                            many_foreign_table_rs_name,
                        };

                        Column::Compounds(ColumnCompounds {
                            response: ResponseColumnCompounds {
                                field: ResponseColumnField {
                                    rs_name,
                                    rs_ty,
                                    attr: response,
                                    rs_attrs,
                                },
                                getter: compounds_getter,
                            },
                            request: RequestColumnCompounds {
                                field: RequestColumnField {
                                    rs_name,
                                    rs_ty: DerefEither::Right(Box::new(rs_ty_compounds_request(
                                        many_foreign_table_rs_name,
                                        index_rs_name,
                                    ))),
                                    attr: request,
                                    rs_attrs,
                                    is_mut,
                                },
                                setter: compounds_setter,
                            },
                            struct_name,
                        })
                    }
                };
                Ok(column0)
            },
        );
        let columns = columns?;

        let create_request_rs_name = quote::format_ident!("{}CreateRequest", table.rs_name);
        let update_request_rs_name = quote::format_ident!("{}UpdateRequest", table.rs_name);
        let patch_request_rs_name = quote::format_ident!("{}PatchRequest", table.rs_name);
        let request_error_rs_name = quote::format_ident!("{}RequestError", table.rs_name);
        Ok(Self {
            name_intern: table_name_intern,
            name_extern: table_name_extern,
            rs_name: &table.rs_name,
            create_request_rs_name: Cow::Owned(create_request_rs_name),
            update_request_rs_name: Cow::Owned(update_request_rs_name),
            patch_request_rs_name: Cow::Owned(patch_request_rs_name),
            request_error_rs_name: Cow::Owned(request_error_rs_name),
            index_rs_name: table.index_rs_name.as_ref(),
            db_rs_name: &db.rs_name,
            rs_attrs: &*table.rs_attrs,
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
