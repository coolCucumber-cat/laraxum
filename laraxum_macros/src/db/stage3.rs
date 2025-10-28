use super::stage2;

pub use stage2::{
    AtomicTy, AtomicTyFloat, AtomicTyInt, AtomicTyString, AtomicTyTime, AutoTimeEvent, Columns,
    DefaultValue, TyElementAutoTime,
};

use crate::utils::{borrow::CowBoxDeref, collections::TryCollectAll};

use std::borrow::Cow;

use syn::{Attribute, Ident, Type, Visibility};

const TABLE_MUST_HAVE_ID: &str = "table must have an ID";
const TABLE_ID_MUST_BE_INT: &str = "table ID must be int";
const COLUMN_MUST_NOT_BE_COLLECTION: &str = "column must not be many-to-many relationship";
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

pub enum TyMolecule<'a> {
    Element(TyElement),
    Compound(TyCompound<'a>),
}
impl TyMolecule<'_> {
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
    pub ty: TyMolecule<'a>,
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

pub enum ResponseColumnGetterMolecule<'a> {
    Element(ResponseColumnGetterElement<'a>),
    Compound(ResponseColumnGetterCompound<'a>),
}
impl ResponseColumnGetterMolecule<'_> {
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

pub struct ResponseColumnGetterCollection<'a> {
    pub rs_name: &'a Ident,
    pub aggregate_rs_name: &'a Ident,
    pub table_id_name_extern: String,
    pub many_foreign_table_rs_name: &'a Ident,
}

pub enum ResponseColumnGetter<'a> {
    Molecule(ResponseColumnGetterMolecule<'a>),
    Collection(ResponseColumnGetterCollection<'a>),
}

#[derive(Clone, Copy)]
pub enum ResponseColumnGetterRef<'a> {
    Molecule(&'a ResponseColumnGetterMolecule<'a>),
    Collection(&'a ResponseColumnGetterCollection<'a>),
}
impl ResponseColumnGetterRef<'_> {
    pub const fn rs_name(&self) -> &Ident {
        match self {
            Self::Molecule(molecule) => molecule.rs_name(),
            Self::Collection(collection) => collection.rs_name,
        }
    }
}
impl<'a> From<&'a ResponseColumnGetter<'a>> for ResponseColumnGetterRef<'a> {
    fn from(column: &'a ResponseColumnGetter<'a>) -> Self {
        match *column {
            ResponseColumnGetter::Molecule(ref molecule) => Self::Molecule(molecule),
            ResponseColumnGetter::Collection(ref collection) => Self::Collection(collection),
        }
    }
}

pub struct ResponseColumnField<'a> {
    pub rs_name: &'a Ident,
    pub rs_ty: &'a Type,
    pub attr: &'a stage2::ColumnAttrResponse,
    pub rs_attrs: &'a [Attribute],
}

pub struct ResponseColumnMolecule<'a> {
    pub field: ResponseColumnField<'a>,
    pub getter: ResponseColumnGetterMolecule<'a>,
}

pub struct ResponseColumnCollection<'a> {
    pub field: ResponseColumnField<'a>,
    pub getter: ResponseColumnGetterCollection<'a>,
}

pub use stage2::Validate;

pub struct RequestColumnSetterMolecule<'a> {
    pub rs_name: &'a Ident,
    pub name: &'a str,
    pub is_optional: bool,
    pub is_mut: bool,
    pub validate: &'a Validate,
}

pub struct RequestColumnSetterCollection<'a> {
    pub rs_name: &'a Ident,
    pub aggregate_rs_name: &'a Ident,
    pub many_foreign_table_rs_name: &'a Ident,
}

pub struct RequestColumnField<'a> {
    pub rs_name: &'a Ident,
    pub rs_ty: CowBoxDeref<'a, Type>,
    pub attr: &'a stage2::ColumnAttrRequest,
    pub rs_attrs: &'a [Attribute],
}

pub struct RequestColumnMoleculeMutable<'a> {
    pub field: RequestColumnField<'a>,
    pub setter: RequestColumnSetterMolecule<'a>,
    // pub is_mut: bool,
}

pub struct RequestColumnMoleculeOnUpdate<'a> {
    pub name: &'a str,
    pub time_ty: stage2::AtomicTyTime,
}

pub enum RequestColumnMolecule<'a> {
    Mutable(RequestColumnMoleculeMutable<'a>),
    OnUpdate(RequestColumnMoleculeOnUpdate<'a>),
}
impl RequestColumnMolecule<'_> {
    pub const fn mutable(&self) -> Option<&RequestColumnMoleculeMutable<'_>> {
        match self {
            Self::Mutable(mutable) => Some(mutable),
            Self::OnUpdate(_) => None,
        }
    }
    pub const fn field(&self) -> Option<&RequestColumnField<'_>> {
        match self {
            Self::Mutable(mutable) => Some(&mutable.field),
            Self::OnUpdate(_) => None,
        }
    }
    pub const fn setter(&self) -> Option<&RequestColumnSetterMolecule<'_>> {
        match self {
            Self::Mutable(mutable) => Some(&mutable.setter),
            Self::OnUpdate(_) => None,
        }
    }
    pub const fn is_mut(&self) -> bool {
        match self {
            Self::Mutable(mutable) => mutable.setter.is_mut,
            // Self::Mutable(mutable) => mutable.is_mut,
            Self::OnUpdate(_) => true,
        }
    }
}

pub struct RequestColumnCollection<'a> {
    pub field: RequestColumnField<'a>,
    pub setter: RequestColumnSetterCollection<'a>,
}

pub use stage2::ColumnAttrAggregate;
pub use stage2::ColumnAttrAggregateFilter;
pub use stage2::ColumnAttrAggregateLimit;

pub struct ColumnMolecule<'a> {
    pub create: CreateColumn<'a>,
    pub response: ResponseColumnMolecule<'a>,
    pub request: Option<RequestColumnMolecule<'a>>,
    pub aggregates: &'a [ColumnAttrAggregate],
    pub borrow: Option<Option<&'a Type>>,
    pub struct_name: Option<&'a Ident>,
}
impl ColumnMolecule<'_> {
    pub const fn name(&self) -> &str {
        self.create.name
    }
    pub fn name_intern(&self) -> &str {
        self.response.getter.name_intern()
    }
}

pub struct ColumnCollection<'a> {
    pub response: ResponseColumnCollection<'a>,
    pub request: RequestColumnCollection<'a>,
    pub struct_name: Option<&'a Ident>,
}

pub enum Column<'a> {
    Molecule(ColumnMolecule<'a>),
    Collection(ColumnCollection<'a>),
}
impl<'a> TryFrom<Column<'a>> for ColumnMolecule<'a> {
    type Error = syn::Error;
    fn try_from(column: Column<'a>) -> Result<Self, Self::Error> {
        match column {
            Column::Molecule(molecule) => Ok(molecule),
            Column::Collection(collection) => Err(syn::Error::new(
                collection.response.field.rs_name.span(),
                COLUMN_MUST_NOT_BE_COLLECTION,
            )),
        }
    }
}

#[derive(Clone, Copy)]
pub enum ColumnRef<'a> {
    Molecule(&'a ColumnMolecule<'a>),
    Collection(&'a ColumnCollection<'a>),
}
impl<'a> ColumnRef<'a> {
    pub const fn create(self) -> Option<&'a CreateColumn<'a>> {
        match self {
            Self::Molecule(molecule) => Some(&molecule.create),
            Self::Collection(_) => None,
        }
    }
    pub const fn response_field(self) -> &'a ResponseColumnField<'a> {
        match self {
            Self::Molecule(molecule) => &molecule.response.field,
            Self::Collection(collection) => &collection.response.field,
        }
    }
    pub const fn response_getter(self) -> ResponseColumnGetterRef<'a> {
        match self {
            Self::Molecule(molecule) => {
                ResponseColumnGetterRef::Molecule(&molecule.response.getter)
            }
            Self::Collection(collection) => {
                ResponseColumnGetterRef::Collection(&collection.response.getter)
            }
        }
    }
    pub fn request_field(self) -> Option<&'a RequestColumnField<'a>> {
        match self {
            Self::Molecule(molecule) => molecule
                .request
                .as_ref()
                .and_then(|request| request.field()),
            Self::Collection(collection) => Some(&collection.request.field),
        }
    }
    pub const fn request_molecule(self) -> Option<&'a RequestColumnMolecule<'a>> {
        match self {
            Self::Molecule(molecule) => molecule.request.as_ref(),
            Self::Collection(_) => None,
        }
    }
    pub const fn request_collection(self) -> Option<&'a RequestColumnCollection<'a>> {
        match self {
            Self::Molecule(_) => None,
            Self::Collection(collection) => Some(&collection.request),
        }
    }
    pub fn request_setter_molecule(self) -> Option<&'a RequestColumnSetterMolecule<'a>> {
        match self {
            Self::Molecule(molecule) => molecule
                .request
                .as_ref()
                .and_then(|request| request.setter()),
            Self::Collection(_) => None,
        }
    }
    pub const fn request_setter_collection(self) -> Option<&'a RequestColumnSetterCollection<'a>> {
        match self {
            Self::Molecule(_) => None,
            Self::Collection(collection) => Some(&collection.request.setter),
        }
    }
    pub const fn struct_name(self) -> Option<&'a Ident> {
        match self {
            Self::Molecule(molecule) => molecule.struct_name,
            Self::Collection(collection) => collection.struct_name,
        }
    }
    pub fn is_mut(self) -> bool {
        match self {
            Self::Molecule(molecule) => molecule
                .request
                .as_ref()
                .is_some_and(|request| request.is_mut()),
            Self::Collection(_) => true,
        }
    }
}
impl<'a> From<&'a Column<'a>> for ColumnRef<'a> {
    fn from(column: &'a Column<'a>) -> Self {
        match column {
            Column::Molecule(molecule) => ColumnRef::Molecule(molecule),
            Column::Collection(collection) => ColumnRef::Collection(collection),
        }
    }
}

impl<T, C> Columns<T, T, C> {
    pub fn map_try_collect_all_default<'a, F>(
        &'a self,
        mut f: F,
    ) -> Result<Columns<Column<'a>, ColumnMolecule<'a>, &'a C>, syn::Error>
    where
        F: FnMut(&'a T) -> Result<Column<'a>, syn::Error>,
    {
        match self {
            Self::CollectionOnly { columns } => {
                let columns = columns.iter().map(|c| f(c));
                let columns: Result<Vec<Column<'a>>, syn::Error> = columns.try_collect_all();
                let columns = columns?;
                Ok(Columns::CollectionOnly { columns })
            }
            Self::Model {
                id,
                columns,
                controller,
            } => {
                let id = f(id)?;
                let id = ColumnMolecule::try_from(id)?;
                let columns = columns.iter().map(f);
                let columns: Result<Vec<Column<'a>>, syn::Error> = columns.try_collect_all();
                let columns = columns?;
                Ok(Columns::Model {
                    id,
                    columns,
                    controller: controller.as_ref(),
                })
            }
            Self::ManyModel { a, b } => {
                let a = f(a)?;
                let a = ColumnMolecule::try_from(a)?;
                let b = f(b)?;
                let b = ColumnMolecule::try_from(b)?;
                Ok(Columns::ManyModel { a, b })
            }
        }
    }
}
impl<'a, C> Columns<Column<'a>, ColumnMolecule<'a>, C> {
    pub fn iter(&'a self) -> impl Iterator<Item = ColumnRef<'a>> + Clone {
        let (a, b, c) = match self {
            Self::CollectionOnly { columns } => (None, None, columns.iter().map(ColumnRef::from)),
            Self::Model { id, columns, .. } => (
                Some(ColumnRef::Molecule(id)),
                None,
                columns.iter().map(ColumnRef::from),
            ),
            Self::ManyModel { a, b } => (
                Some(ColumnRef::Molecule(a)),
                Some(ColumnRef::Molecule(b)),
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
    pub aggregate_rs_name: Option<&'a Ident>,
    pub db_rs_name: &'a Ident,
    pub rs_attrs: &'a [syn::Attribute],
    pub columns: Columns<Column<'a>, ColumnMolecule<'a>, &'a stage2::TableAttrController>,
}

impl<'a> Table<'a> {
    #[expect(clippy::too_many_lines)]
    fn try_new(table: &'a stage2::Table, db: &'a stage2::Db) -> syn::Result<Self> {
        fn rs_ty_compound_request(rs_ty: &Type, is_optional: bool) -> CowBoxDeref<'_, Type> {
            if is_optional {
                CowBoxDeref::Owned(Box::new(syn::parse_quote!(Option<#rs_ty>)))
            } else {
                CowBoxDeref::Borrowed(rs_ty)
            }
        }
        fn rs_ty_collection_request(
            many_foreign_table_rs_name: &Ident,
            aggregate_rs_name: &Ident,
        ) -> Type {
            syn::parse_quote!(
                Vec<
                    <
                        #many_foreign_table_rs_name as ::laraxum::ManyModel<#aggregate_rs_name>
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
                    stage2::TyMolecule::Element(ref ty_element) => ResponseColumnGetter::Molecule(
                        ResponseColumnGetterMolecule::Element(ResponseColumnGetterElement {
                            name_intern: column_name_intern,
                            name_extern: column_name_extern,
                            rs_name,
                            is_optional: ty_element.is_optional(),
                        }),
                    ),
                    stage2::TyMolecule::Compound(stage2::TyCompound {
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
                            columns.try_collect_all();
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
                        ResponseColumnGetter::Molecule(ResponseColumnGetterMolecule::Compound(
                            compound,
                        ))
                    }
                    stage2::TyMolecule::Compound(stage2::TyCompound {
                        rs_ty_name: _,
                        multiplicity:
                            stage2::TyCompoundMultiplicity::Many(stage2::ColumnAttrTyCollection {
                                model_rs_name: ref many_foreign_table_rs_name,
                                ref aggregate_rs_ty,
                            }),
                    }) => {
                        let table_rs_name = &table.rs_name;
                        let aggregate_rs_name = aggregate_rs_ty.as_ref().unwrap_or(table_rs_name);
                        let table_id = table.columns.model().ok_or_else(|| {
                            syn::Error::new(table_rs_name.span(), TABLE_MUST_HAVE_ID)
                        })?;
                        let table_id_name_extern = name_extern((table_name_extern, &table_id.name));
                        ResponseColumnGetter::Collection(ResponseColumnGetterCollection {
                            rs_name,
                            aggregate_rs_name,
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
                let &stage2::Column {
                    ref name,
                    ref rs_name,
                    ref ty,
                    ref rs_ty,
                    ref response,
                    ref request,
                    is_mut,
                    ref borrow,
                    ref aggregates,
                    ref struct_name,
                    ref rs_attrs,
                } = column;
                let (column_name_intern, column_name_extern) =
                    name_intern_extern((&*table_name_extern, name));
                let aggregates = &**aggregates;
                let borrow = borrow.as_ref().map(Option::as_deref);
                let struct_name = struct_name.as_ref();

                let column0 = match *ty {
                    stage2::TyMolecule::Element(ref ty_element) => {
                        Column::Molecule(ColumnMolecule {
                            create: CreateColumn {
                                name,
                                ty: TyMolecule::Element(ty_element.clone()),
                            },
                            response: ResponseColumnMolecule {
                                getter: ResponseColumnGetterMolecule::Element(
                                    ResponseColumnGetterElement {
                                        name_intern: column_name_intern,
                                        name_extern: column_name_extern,
                                        is_optional: ty_element.is_optional(),
                                        rs_name,
                                    },
                                ),
                                field: ResponseColumnField {
                                    rs_name,
                                    rs_ty,
                                    attr: response,
                                    rs_attrs,
                                },
                            },
                            request: match ty_element {
                                TyElement::Value(_) => {
                                    Some(RequestColumnMolecule::Mutable(
                                        RequestColumnMoleculeMutable {
                                            field: RequestColumnField {
                                                rs_name,
                                                rs_ty: CowBoxDeref::Borrowed(rs_ty),
                                                attr: request,
                                                rs_attrs,
                                            },
                                            setter: RequestColumnSetterMolecule {
                                                rs_name,
                                                name,
                                                is_optional: ty_element.is_optional(),
                                                is_mut,
                                                validate: &request.validate,
                                            },
                                            // is_mut,
                                        },
                                    ))
                                }
                                TyElement::AutoTime(TyElementAutoTime {
                                    event: AutoTimeEvent::OnUpdate,
                                    ty,
                                }) => Some(RequestColumnMolecule::OnUpdate(
                                    RequestColumnMoleculeOnUpdate {
                                        name,
                                        time_ty: ty.clone(),
                                    },
                                )),
                                TyElement::AutoTime(_) | TyElement::Id(_) => None,
                            },
                            aggregates,
                            borrow,
                            struct_name,
                        })
                    }
                    stage2::TyMolecule::Compound(stage2::TyCompound {
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
                            columns.try_collect_all();
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

                        Column::Molecule(ColumnMolecule {
                            create: CreateColumn {
                                name,
                                ty: TyMolecule::Compound(TyCompound {
                                    foreign_table_name: &foreign_table.name,
                                    foreign_table_id_name: &foreign_table_id.name,
                                    ty: foreign_table_id_ty,
                                    is_optional,
                                    is_unique,
                                }),
                            },
                            response: ResponseColumnMolecule {
                                field: ResponseColumnField {
                                    rs_name,
                                    rs_ty,
                                    attr: response,
                                    rs_attrs,
                                },
                                getter: ResponseColumnGetterMolecule::Compound(compound),
                            },
                            request: Some(RequestColumnMolecule::Mutable(
                                RequestColumnMoleculeMutable {
                                    field: RequestColumnField {
                                        rs_name,
                                        rs_ty: rs_ty_compound_request(
                                            foreign_table_id_rs_ty,
                                            is_optional,
                                        ),
                                        attr: request,
                                        rs_attrs,
                                    },
                                    setter: RequestColumnSetterMolecule {
                                        rs_name,
                                        name,
                                        is_optional,
                                        is_mut,
                                        validate: &request.validate,
                                    },
                                    // is_mut,
                                },
                            )),
                            aggregates,
                            borrow,
                            struct_name,
                        })
                    }
                    stage2::TyMolecule::Compound(stage2::TyCompound {
                        rs_ty_name: _,
                        multiplicity:
                            stage2::TyCompoundMultiplicity::Many(stage2::ColumnAttrTyCollection {
                                model_rs_name: ref many_foreign_table_rs_name,
                                ref aggregate_rs_ty,
                            }),
                    }) => {
                        let table_rs_name = &table.rs_name;
                        let aggregate_rs_name = aggregate_rs_ty.as_ref().unwrap_or(table_rs_name);
                        let table_id = table.columns.model().ok_or_else(|| {
                            syn::Error::new(table_rs_name.span(), TABLE_MUST_HAVE_ID)
                        })?;
                        let table_id_name_extern =
                            name_extern((&table_name_extern, &table_id.name));
                        Column::Collection(ColumnCollection {
                            response: ResponseColumnCollection {
                                field: ResponseColumnField {
                                    rs_name,
                                    rs_ty,
                                    attr: response,
                                    rs_attrs,
                                },
                                getter: ResponseColumnGetterCollection {
                                    rs_name,
                                    aggregate_rs_name,
                                    table_id_name_extern,
                                    many_foreign_table_rs_name,
                                },
                            },
                            request: RequestColumnCollection {
                                field: RequestColumnField {
                                    rs_name,
                                    rs_ty: CowBoxDeref::Owned(Box::new(rs_ty_collection_request(
                                        many_foreign_table_rs_name,
                                        aggregate_rs_name,
                                    ))),
                                    attr: request,
                                    rs_attrs,
                                },
                                setter: RequestColumnSetterCollection {
                                    rs_name,
                                    aggregate_rs_name,
                                    many_foreign_table_rs_name,
                                },
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
            aggregate_rs_name: table.aggregate_rs_name.as_ref(),
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
            .try_collect_all();
        let tables = tables?;

        Ok(Self {
            name: &db.name,
            rs_name: &db.rs_name,
            tables,
            rs_vis: &db.rs_vis,
        })
    }
}
