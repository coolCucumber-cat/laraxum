use super::stage3;

use crate::utils::syn::from_str_to_rs_ident;

use std::{borrow::Cow, vec};

use quote::{ToTokens, quote};
use syn::{Ident, Type};

impl stage3::AtomicTyInt {
    const fn ty(&self) -> &'static str {
        match self {
            Self::u8 => "TINYINT UNSIGNED",
            Self::i8 => "TINYINT",
            Self::u16 => "SMALLINT UNSIGNED",
            Self::i16 => "SMALLINT",
            Self::u32 => "INT UNSIGNED",
            Self::i32 => "INT",
            Self::u64 => "BIGINT UNSIGNED",
            Self::i64 => "BIGINT",
        }
    }
}

impl stage3::AtomicTyFloat {
    const fn ty(&self) -> &'static str {
        match self {
            Self::f32 => "FLOAT",
            Self::f64 => "DOUBLE",
        }
    }
}

impl stage3::AtomicTyString {
    fn ty(&self) -> Cow<'static, str> {
        #[cfg(feature = "mysql")]
        match self {
            Self::Varchar(len) => Cow::Owned(fmt2::fmt! { { str } => "VARCHAR(" {len} ")" }),
            Self::Char(len) => Cow::Owned(fmt2::fmt! { { str } => "CHAR(" {len} ")" }),
            Self::Text => Cow::Borrowed("TEXT"),
        }
    }
}

impl stage3::AtomicTyTime {
    const fn ty(&self) -> &'static str {
        #[cfg(feature = "mysql")]
        match self {
            Self::ChronoDateTimeUtc | Self::ChronoDateTimeLocal | Self::TimeOffsetDateTime => {
                "TIMESTAMP"
            }
            Self::ChronoNaiveDateTime | Self::TimePrimitiveDateTime => "DATETIME",
            Self::ChronoNaiveDate | Self::TimeDate => "DATE",
            Self::ChronoNaiveTime | Self::TimeTime | Self::ChronoTimeDelta | Self::TimeDuration => {
                "TIME"
            }
        }
    }
    const fn current_time_func(&self) -> &'static str {
        #[cfg(feature = "mysql")]
        match self {
            Self::ChronoDateTimeUtc => "UTC_TIMESTAMP()",
            Self::ChronoDateTimeLocal
            | Self::TimeOffsetDateTime
            | Self::ChronoNaiveDateTime
            | Self::TimePrimitiveDateTime => "CURRENT_TIMESTAMP()",
            Self::ChronoNaiveDate | Self::TimeDate => "CURRENT_DATE()",
            Self::ChronoNaiveTime | Self::TimeTime | Self::ChronoTimeDelta | Self::TimeDuration => {
                "CURRENT_TIME()"
            }
        }
    }
}

impl stage3::AtomicTy {
    fn ty(&self) -> Cow<'static, str> {
        #[cfg(feature = "mysql")]
        match self {
            Self::bool => Cow::Borrowed("BOOL"),
            Self::Int(int) => Cow::Borrowed(int.ty()),
            Self::Float(float) => Cow::Borrowed(float.ty()),
            Self::String(string) => string.ty(),
            Self::Time(time) => Cow::Borrowed(time.ty()),
        }
        //         #[cfg(feature = "sqlite")]
        //         match self {
        //             Self::bool => Cow::Borrowed("BOOLEAN"),
        //             Self::u8 => Cow::Borrowed("INTEGER"),
        //             Self::i8 => Cow::Borrowed("INTEGER"),
        //             Self::u16 => Cow::Borrowed("INTEGER"),
        //             Self::i16 => Cow::Borrowed("INTEGER"),
        //             Self::u32 => Cow::Borrowed("INTEGER"),
        //             Self::i32 => Cow::Borrowed("INTEGER"),
        //             Self::u64 => Cow::Borrowed("INTEGER"),
        //             Self::i64 => Cow::Borrowed("BIGINT"),
        //             Self::f32 => Cow::Borrowed("FLOAT"),
        //             Self::f64 => Cow::Borrowed("DOUBLE"),
        //
        //             Self::String(string) => string.sql_ty(),
        //             Self::Time(time) => Cow::Borrowed(time.sql_ty()),
        //         }
        //
        //         #[cfg(feature = "postgres")]
        //         match self {
        //             Self::bool => Cow::Borrowed("BOOL"),
        //             Self::u8 => Cow::Borrowed("CHAR"),
        //             Self::i8 => Cow::Borrowed("CHAR"),
        //             Self::u16 => Cow::Borrowed("INT2"),
        //             Self::i16 => Cow::Borrowed("INT2"),
        //             Self::u32 => Cow::Borrowed("INT4"),
        //             Self::i32 => Cow::Borrowed("INT4"),
        //             Self::u64 => Cow::Borrowed("INT8"),
        //             Self::i64 => Cow::Borrowed("INT8"),
        //             Self::f32 => Cow::Borrowed("FLOAT4"),
        //             Self::f64 => Cow::Borrowed("FLOAT8"),
        //
        //             Self::String(string) => string.sql_ty(),
        //             Self::Time(time) => Cow::Borrowed(time.sql_ty()),
        //         }
    }
}

impl stage3::DefaultValue<'_> {
    const fn default_value(&self) -> Cow<'static, str> {
        match self {
            Self::AutoTime(time_ty) => Cow::Borrowed(time_ty.current_time_func()),
        }
    }
}

impl stage3::TyElement {
    fn ty(&self) -> Cow<'static, str> {
        match self {
            Self::Id(id) => Cow::Borrowed(id.ty()),
            // Self::Id(id) => Cow::Borrowed(
            //     {
            //     #[cfg(feature = "mysql")]
            //     {
            //         "BIGINT UNSIGNED"
            //     }
            //     #[cfg(feature = "sqlite")]
            //     {
            //         "INTEGER"
            //     }
            //     #[cfg(feature = "postgres")]
            //     {
            //         "BIGSERIAL"
            //     }
            //     }
            // ),
            Self::Value(value) => value.ty.ty(),
            Self::AutoTime(auto_time) => Cow::Borrowed(auto_time.ty.ty()),
        }
    }
}

impl stage3::TyCompound<'_> {
    const fn ty(&self) -> &'static str {
        self.ty.ty()
        // {
        //     #[cfg(feature = "mysql")]
        //     {
        //         "BIGINT UNSIGNED"
        //     }
        //     #[cfg(feature = "sqlite")]
        //     {
        //         "INTEGER"
        //     }
        //     #[cfg(feature = "postgres")]
        //     {
        //         "BIGINT"
        //     }
        // }
    }
}

impl stage3::TyMolecule<'_> {
    fn ty(&self) -> Cow<'static, str> {
        match self {
            Self::Compound(compound) => Cow::Borrowed(compound.ty()),
            Self::Element(element) => element.ty(),
        }
    }
}
impl fmt2::write_to::WriteTo for stage3::TyMolecule<'_> {
    fn write_to<W>(&self, w: &mut W) -> Result<(), W::Error>
    where
        W: fmt2::write::Write + ?Sized,
    {
        fmt2::fmt! { (? w) => {self.ty()} }?;
        if self.is_optional() {
            fmt2::fmt! { (? w) => " NOT NULL" }?;
        }
        if self.is_unique() && !self.is_id() {
            fmt2::fmt! { (? w) => " UNIQUE" }?;
        }
        if let Some(default_value) = self.default_value() {
            let default_value = default_value.default_value();
            fmt2::fmt! { (? w) => " DEFAULT " {default_value} }?;
        }
        match self {
            Self::Element(stage3::TyElement::Id(_)) => {
                #[cfg(feature = "mysql")]
                fmt2::fmt! { (? w) => " PRIMARY KEY AUTO_INCREMENT" }?;
                #[cfg(feature = "sqlite")]
                fmt2::fmt! { (? w) => " PRIMARY KEY AUTOINCREMENT" }?;
                #[cfg(feature = "postgres")]
                fmt2::fmt! { (? w) => " PRIMARY KEY" }?;
            }
            Self::Element(_) => {}
            Self::Compound(stage3::TyCompound {
                foreign_table_name,
                foreign_table_id_name,
                ..
            }) => {
                fmt2::fmt! { (? w) => " FOREIGN KEY REFERENCES " {foreign_table_name} "(" {foreign_table_id_name} ")" }?;
            }
        }
        Ok(())
    }
}

impl fmt2::write_to::WriteTo for stage3::CreateColumn<'_> {
    fn write_to<W>(&self, w: &mut W) -> Result<(), W::Error>
    where
        W: fmt2::write::Write + ?Sized,
    {
        fmt2::fmt! { (? w) => {self.name} " " {self.ty} }
    }
}

struct ResponseColumnGetterElement<'a> {
    element: &'a stage3::ResponseColumnGetterElement<'a>,
    parent_optional: bool,
}
impl fmt2::write_to::WriteTo for ResponseColumnGetterElement<'_> {
    fn write_to<W>(&self, w: &mut W) -> Result<(), W::Error>
    where
        W: fmt2::write::Write + ?Sized,
    {
        if self.parent_optional || self.element.is_optional {
            fmt2::fmt! { (? w) =>
                {self.element.name_intern}
                " AS "
                {self.element.name_extern}
            }
        } else {
            #[cfg(feature = "mysql")]
            fmt2::fmt! { (? w) =>
                {self.element.name_intern}
                " AS `"
                {self.element.name_extern}
                "!`"
            }
            #[cfg(any(feature = "sqlite", feature = "postgres"))]
            fmt2::fmt! { (? w) =>
                {self.element.name_intern}
                " AS \""
                {self.element.name_extern}
                "!\""
            }
            #[cfg(not(any(feature = "mysql", feature = "sqlite", feature = "postgres")))]
            unimplemented!();
        }
    }
}

enum Sort {
    Ascending,
    Descending,
}

fn create_table<'columns>(
    table_name_intern: &str,
    columns: impl IntoIterator<Item = stage3::ColumnRef<'columns>>,
) -> String {
    let create_columns = columns.into_iter().filter_map(|column| column.create());

    fmt2::fmt! { { str } =>
        "CREATE TABLE IF NOT EXISTS " {table_name_intern} " ("
            @..join(create_columns => "," => |column| {column})
        ");"
    }
}
fn delete_table(table_name_intern: &str) -> String {
    fmt2::fmt! { { str } =>
        "DROP TABLE " {table_name_intern} ";"
    }
}
fn get(
    table_name_intern: &str,
    table_name_extern: &str,
    (response_getter_column_elements, response_getter_column_compounds): (
        &[ResponseColumnGetterElement],
        &[&stage3::ResponseColumnGetterCompound],
    ),
    aggregate_filter: Option<(stage3::ColumnAttrAggregateFilter, &str)>,
    aggregate_sort: Option<(Sort, &str)>,
    aggregate_limit: Option<stage3::ColumnAttrAggregateLimit>,
    is_one: bool,
) -> String {
    let mut get = fmt2::fmt! { { str } =>
        "SELECT "
        @..join(response_getter_column_elements => "," => |element|
            {element}
        )
        " FROM " {table_name_intern} " AS " {table_name_extern}
        @..(response_getter_column_compounds => |compound|
            " LEFT JOIN "
            {compound.foreign_table_name_intern} " AS " {compound.foreign_table_name_extern}
            " ON "
            {compound.name_intern} "=" {compound.foreign_table_id_name_intern}
        )
    };
    if let Some((aggregate_filter, filter_column_name_intern)) = aggregate_filter {
        match aggregate_filter {
            stage3::ColumnAttrAggregateFilter::None => {}
            stage3::ColumnAttrAggregateFilter::Eq => {
                fmt2::fmt! { (get) => " WHERE " {filter_column_name_intern} "=?" };
            }
            stage3::ColumnAttrAggregateFilter::Like => {
                fmt2::fmt! { (get) => " WHERE " {filter_column_name_intern} " LIKE CONCAT('%', ?, '%')" };
            }
            stage3::ColumnAttrAggregateFilter::Gt => {
                fmt2::fmt! { (get) => " WHERE " {filter_column_name_intern} ">?" };
            }
            stage3::ColumnAttrAggregateFilter::Lt => {
                fmt2::fmt! { (get) => " WHERE " {filter_column_name_intern} "<?" };
            }
            stage3::ColumnAttrAggregateFilter::Gte => {
                fmt2::fmt! { (get) => " WHERE " {filter_column_name_intern} ">=?" };
            }
            stage3::ColumnAttrAggregateFilter::Lte => {
                fmt2::fmt! { (get) => " WHERE " {filter_column_name_intern} "<=?" };
            }
        }
    }
    if let Some((aggregate_sort, sort_column_name_intern)) = aggregate_sort {
        match aggregate_sort {
            Sort::Ascending => {
                fmt2::fmt! { (get) => " ORDER BY " {sort_column_name_intern} " ASC" };
            }
            Sort::Descending => {
                fmt2::fmt! { (get) => " ORDER BY " {sort_column_name_intern} " DESC" };
            }
        }
    }
    if is_one {
        fmt2::fmt! { (get) => " LIMIT 1" };
    } else if let Some(aggregate_limit) = aggregate_limit {
        match aggregate_limit {
            stage3::ColumnAttrAggregateLimit::None => {}
            stage3::ColumnAttrAggregateLimit::Limit => {
                fmt2::fmt! { (get) => " LIMIT ?" };
            }
            stage3::ColumnAttrAggregateLimit::Page { per_page } => {
                // the `OFFSET` is set in the parameter as `OFFSET * per_page`
                fmt2::fmt! { (get) => " LIMIT " {per_page} " OFFSET ? " };
            }
        }
    }
    get
}
fn get_all(
    table_name_intern: &str,
    table_name_extern: &str,
    response_getters: (
        &[ResponseColumnGetterElement],
        &[&stage3::ResponseColumnGetterCompound],
    ),
) -> String {
    get(
        table_name_intern,
        table_name_extern,
        response_getters,
        None,
        None,
        None,
        false,
    )
}
fn get_one(
    table_name_intern: &str,
    table_name_extern: &str,
    response_getters: (
        &[ResponseColumnGetterElement],
        &[&stage3::ResponseColumnGetterCompound],
    ),
    aggregate_filter: (stage3::ColumnAttrAggregateFilter, &str),
) -> String {
    get(
        table_name_intern,
        table_name_extern,
        response_getters,
        Some(aggregate_filter),
        None,
        None,
        true,
    )
}
fn get_many(
    table_name_intern: &str,
    table_name_extern: &str,
    response_getters: (
        &[ResponseColumnGetterElement],
        &[&stage3::ResponseColumnGetterCompound],
    ),
    aggregate_filter: (stage3::ColumnAttrAggregateFilter, &str),
) -> String {
    get(
        table_name_intern,
        table_name_extern,
        response_getters,
        Some(aggregate_filter),
        None,
        None,
        false,
    )
}
fn get_sort_asc_desc(
    table_name_intern: &str,
    table_name_extern: &str,
    response_getters: (
        &[ResponseColumnGetterElement],
        &[&stage3::ResponseColumnGetterCompound],
    ),
    aggregate_filter: Option<(stage3::ColumnAttrAggregateFilter, &str)>,
    sort_column_name_intern: &str,
    aggregate_limit: Option<stage3::ColumnAttrAggregateLimit>,
    is_one: bool,
) -> (String, String) {
    (
        get(
            table_name_intern,
            table_name_extern,
            response_getters,
            aggregate_filter,
            Some((Sort::Ascending, sort_column_name_intern)),
            aggregate_limit,
            is_one,
        ),
        get(
            table_name_intern,
            table_name_extern,
            response_getters,
            aggregate_filter,
            Some((Sort::Descending, sort_column_name_intern)),
            aggregate_limit,
            is_one,
        ),
    )
}
pub const fn request_setter_column<'a>(
    column: &'a stage3::RequestColumnMolecule<'a>,
) -> (&'a str, &'a str) {
    match column {
        stage3::RequestColumnMolecule::Mutable(value) => (value.setter.name, "?"),
        stage3::RequestColumnMolecule::OnUpdate(on_update) => {
            (on_update.name, on_update.time_ty.current_time_func())
        }
    }
}
fn create_one<'columns, I>(table_name_intern: &str, columns: I) -> String
where
    I: IntoIterator<Item = &'columns stage3::RequestColumnMolecule<'columns>>,
    I::IntoIter: Iterator<Item = I::Item> + Clone,
{
    let request_columns = columns.into_iter().map(request_setter_column);
    fmt2::fmt! { { str } =>
        "INSERT INTO " {table_name_intern} " ("
            @..join(request_columns.clone() => "," => |column| {column.0})
        ") VALUES ("
            @..join(request_columns => "," => |column| {column.1})
        ")"
    }
}
fn update_one<'columns, I>(table_name_intern: &str, id_name: &str, columns: I) -> String
where
    I: IntoIterator<Item = &'columns stage3::RequestColumnMolecule<'columns>>,
{
    let request_columns = columns.into_iter().map(request_setter_column);
    fmt2::fmt! { { str } =>
        "UPDATE " {table_name_intern} " SET "
        @..join(request_columns => "," => |column| {column.0} "=" {column.1})
        " WHERE " {id_name} "=?"
    }
}
fn patch_one(
    table_name_intern: &str,
    id_name: &str,
    column: &stage3::RequestColumnMolecule,
) -> String {
    update_one(table_name_intern, id_name, core::iter::once(column))
}
fn delete_one(table_name_intern: &str, id_name: &str) -> String {
    fmt2::fmt! { { str } =>
        "DELETE FROM " {table_name_intern}
        " WHERE " {id_name} "=?"
    }
}

fn flatten_internal<'columns>(
    response_getter_columns: impl IntoIterator<Item = stage3::ResponseColumnGetterRef<'columns>>,
    parent_optional: bool,
    response_getter_column_elements: &mut Vec<ResponseColumnGetterElement<'columns>>,
    response_getter_column_compounds: &mut Vec<
        &'columns stage3::ResponseColumnGetterCompound<'columns>,
    >,
) {
    for response_getter_column in response_getter_columns {
        match response_getter_column {
            stage3::ResponseColumnGetterRef::Molecule(
                stage3::ResponseColumnGetterMolecule::Element(element),
            ) => {
                response_getter_column_elements.push(ResponseColumnGetterElement {
                    element,
                    parent_optional,
                });
            }
            stage3::ResponseColumnGetterRef::Molecule(
                stage3::ResponseColumnGetterMolecule::Compound(compound),
            ) => {
                let parent_optional = parent_optional || compound.is_optional;
                let compound_columns = compound
                    .columns
                    .iter()
                    .map(stage3::ResponseColumnGetterRef::from);
                response_getter_column_compounds.push(compound);
                flatten_internal(
                    compound_columns,
                    parent_optional,
                    response_getter_column_elements,
                    response_getter_column_compounds,
                );
            }
            stage3::ResponseColumnGetterRef::Collection(_) => {}
        }
    }
}
fn flatten<'columns>(
    response_getter_columns: impl Iterator<Item = stage3::ResponseColumnGetterRef<'columns>>,
) -> (
    Vec<ResponseColumnGetterElement<'columns>>,
    Vec<&'columns stage3::ResponseColumnGetterCompound<'columns>>,
) {
    let mut response_getter_column_elements = vec![];
    let mut response_getter_column_compounds = vec![];
    flatten_internal(
        response_getter_columns,
        false,
        &mut response_getter_column_elements,
        &mut response_getter_column_compounds,
    );
    (
        response_getter_column_elements,
        response_getter_column_compounds,
    )
}

fn serde_skip_rs_attr() -> proc_macro2::TokenStream {
    quote! {
        #[serde(skip)]
    }
}
fn serde_rename_rs_attr(serde_name: &str) -> proc_macro2::TokenStream {
    quote! {
        #[serde(rename = #serde_name)]
    }
}

fn response_getter_column(
    name_extern: &Ident,
    is_optional: bool,
    is_parent_optional: bool,
) -> proc_macro2::TokenStream {
    let field_access = quote! {
        response.#name_extern
    };
    if is_optional {
        quote! {
            if let ::core::option::Option::Some(v) = #field_access {
                ::core::option::Option::Some(::laraxum::model::Decode::decode(v))
            } else {
                ::core::option::Option::None
            }
        }
    } else if is_parent_optional {
        quote! {
            if let ::core::option::Option::Some(v) = #field_access {
                ::laraxum::model::Decode::decode(v)
            } else {
                return ::core::result::Result::Ok(::core::option::Option::None);
            }
        }
    } else {
        quote! {
            ::laraxum::model::Decode::decode(#field_access)
        }
    }
}

fn response_getter_compound<'columns>(
    table_ty: &Ident,
    columns: impl IntoIterator<Item = stage3::ResponseColumnGetterRef<'columns>>,
    parent_optional: bool,
) -> proc_macro2::TokenStream {
    let columns = columns.into_iter().map(|column| {
        let rs_name = column.rs_name();
        let response_getter = response_getter(column, parent_optional);
        quote! {
            #rs_name: #response_getter
        }
    });

    quote! {
        #table_ty { #( #columns ),* }
    }
}

fn response_getter(
    column: stage3::ResponseColumnGetterRef<'_>,
    is_parent_optional: bool,
) -> proc_macro2::TokenStream {
    match column {
        stage3::ResponseColumnGetterRef::Molecule(
            stage3::ResponseColumnGetterMolecule::Element(element),
        ) => {
            let &stage3::ResponseColumnGetterElement {
                ref name_extern,
                is_optional,
                ..
            } = element;
            let name_extern = from_str_to_rs_ident(name_extern);
            response_getter_column(&name_extern, is_optional, is_parent_optional)
        }
        stage3::ResponseColumnGetterRef::Molecule(
            stage3::ResponseColumnGetterMolecule::Compound(compound),
        ) => {
            let &stage3::ResponseColumnGetterCompound {
                is_optional,
                foreign_table_rs_name: rs_ty_name,
                ref columns,
                ..
            } = compound;
            let is_parent_optional = is_parent_optional || is_optional;

            let getter = response_getter_compound(
                rs_ty_name,
                columns.iter().map(stage3::ResponseColumnGetterRef::from),
                is_parent_optional,
            );
            if is_optional {
                // catch any returns in the closure, else return `Ok(Some(T))`
                quote! {
                    (async || {
                        ::core::result::Result::Ok::<_, ::sqlx::Error>(
                            ::core::option::Option::Some(#getter)
                        )
                    })().await?
                }
            } else {
                getter
            }
        }
        stage3::ResponseColumnGetterRef::Collection(collection) => {
            let &stage3::ResponseColumnGetterCollection {
                rs_name: _,
                aggregate_rs_name: table_rs_name,
                ref table_id_name_extern,
                many_foreign_table_rs_name,
            } = collection;
            let one_id = {
                let table_id_name_extern = from_str_to_rs_ident(table_id_name_extern);
                response_getter_column(&table_id_name_extern, false, is_parent_optional)
            };
            quote! {
                ::core::result::Result::map_err(
                    <#many_foreign_table_rs_name as ::laraxum::ManyModel::<
                        #table_rs_name,
                    >>::get_many(
                        db,
                        #one_id,
                    ).await,
                    |_| {
                        ::sqlx::Error::ColumnNotFound(
                            ::std::string::String::from(#table_id_name_extern)
                        )
                    }
                )?
            }
        }
    }
}

fn response_getter_fn(getter: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    quote! {
        async |response| match response {
            ::core::result::Result::Ok(response) => ::core::result::Result::Ok(#getter),
            ::core::result::Result::Err(err) => ::core::result::Result::Err(err),
        }
    }
}

fn transform_response_one(
    response: &proc_macro2::TokenStream,
    response_getter: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {{
        let response = #response;
        let response = response.fetch(&db.pool);
        let mut response = ::futures::StreamExt::then(response, #response_getter);
        let mut response = ::core::pin::pin!(response);
        let response: ::core::option::Option<_> =
            ::futures::TryStreamExt::try_next(&mut response).await?;
        ::core::option::Option::ok_or(response, ::laraxum::Error::NotFound)
    }}
}
fn transform_response_many(
    response: &proc_macro2::TokenStream,
    response_getter: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {{
        let response = #response;
        let response = response.fetch(&db.pool);
        let response = ::futures::StreamExt::then(response, #response_getter);
        let response: ::std::vec::Vec<_> =
            ::futures::TryStreamExt::try_collect(response).await?;
        ::core::result::Result::Ok(response)
    }}
}
fn transform_response(
    response: &proc_macro2::TokenStream,
    response_getter: &proc_macro2::TokenStream,
    is_one: bool,
) -> proc_macro2::TokenStream {
    if is_one {
        transform_response_one(response, response_getter)
    } else {
        transform_response_many(response, response_getter)
    }
}

fn request_field(
    rs_name: &Ident,
    rs_ty: &impl quote::ToTokens,
    request_name: Option<&str>,
    rs_attrs: &[syn::Attribute],
) -> proc_macro2::TokenStream {
    let serde_rename_rs_attr = request_name.map(serde_rename_rs_attr);
    quote! {
        #(#rs_attrs)* #serde_rename_rs_attr
        pub #rs_name: #rs_ty,
    }
}

fn request_setter(
    request: &proc_macro2::TokenStream,
    is_optional: bool,
) -> proc_macro2::TokenStream {
    if is_optional {
        quote! {
            ::core::option::Option::map(
                #request,
                ::laraxum::model::Encode::encode,
            )
        }
    } else {
        quote! {
            ::laraxum::model::Encode::encode(#request)
        }
    }
}

fn impl_deserialize_for_untagged_enum<'a, 'b>(
    enum_ident: &Ident,
    enum_variants: impl Iterator<
        Item = (
            &'a Ident,
            impl Iterator<Item = (&'a Ident, &'a Type)> + Clone,
        ),
    > + Clone,
    enum_default_variant: Option<&'b Ident>,
) -> proc_macro2::TokenStream {
    fn field_matcher(ident: &Ident) -> proc_macro2::TokenStream {
        quote! {
            #ident: ::core::option::Option::Some(#ident)
        }
    }
    fn forbidden_field_matcher(ident: &Ident) -> proc_macro2::TokenStream {
        quote! {
            #ident: ::core::option::Option::None
        }
    }
    let struct_fields_iter = enum_variants.clone().flat_map(|(_, fields)| fields);
    let mut struct_fields: Vec<(&Ident, &Type)> = vec![];
    for (ident, ty) in struct_fields_iter {
        let other_field = struct_fields
            .iter()
            .find(|&&(other_ident, _)| other_ident == ident);
        if let Some(&(_, other_ty)) = other_field {
            // if field already exists

            // types for same field must be equal
            assert_eq!(ty, other_ty);
        } else {
            // if doesn't already exist
            struct_fields.push((ident, ty));
        }
    }
    let struct_fields = struct_fields;
    let struct_field_defs = struct_fields.iter().map(|(ident, ty)| {
        quote! {
            #ident: ::core::option::Option<#ty>
        }
    });
    let matchers = enum_variants.map(|(enum_variant_ident, enum_fields)| {
        let enum_field_idents = enum_fields.clone().map(|(ident, _)| ident);
        let forbidden_field_matchers = struct_fields
            .iter()
            .filter(|&&(struct_field_ident, _)| {
                enum_field_idents
                    .clone()
                    .all(|enum_field_ident| enum_field_ident != struct_field_ident)
            })
            .map(|&(field_ident, _)| forbidden_field_matcher(field_ident));
        let field_matchers = enum_field_idents.clone().map(field_matcher);
        let field_setters = enum_field_idents.clone();
        quote! {
            __Struct {
                #( #field_matchers, )*
                #( #forbidden_field_matchers, )*
            } => {
                ::core::result::Result::Ok(#enum_ident::#enum_variant_ident {
                    #( #field_setters, )*
                })
            }
        }
    });
    let default_matcher = enum_default_variant.map(|ident| {
        let forbidden_field_matchers = struct_fields
            .iter()
            .map(|&(other_ident, _)| forbidden_field_matcher(other_ident));
        quote! {
            __Struct {
                #( #forbidden_field_matchers, )*
            } => {
                ::core::result::Result::Ok(#enum_ident::#ident)
            }
        }
    });

    quote! {
        impl<'de> ::serde::Deserialize<'de> for #enum_ident {
            fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                #[derive(::serde::Deserialize)]
                struct __Struct {
                    #( #struct_field_defs ),*
                }
                let deserialized = <__Struct as ::serde::Deserialize>::deserialize(deserializer)?;
                match deserialized {
                    #( #matchers )*
                    #default_matcher
                    _ => ::core::result::Result::Err(
                        ::serde::de::Error::custom("Unknown combination of fields")
                    )
                }
            }
        }
    }
}

struct Table {
    token_stream: proc_macro2::TokenStream,
    migration_up: String,
    migration_down: String,
}
impl From<stage3::Table<'_>> for Table {
    #[allow(clippy::too_many_lines)]
    fn from(table: stage3::Table) -> Self {
        let response_fields = table.columns.iter().map(|column| {
            let &stage3::ResponseColumnField {
                rs_name,
                rs_ty,
                attr,
                rs_attrs,
            } = column.response_field();

            let serde_skip = attr.skip.then(serde_skip_rs_attr);
            let serde_name = attr.name.as_deref().map(serde_rename_rs_attr);

            quote! {
                #( #rs_attrs )* #serde_skip #serde_name
                pub #rs_name: #rs_ty
            }
        });

        let create_table = create_table(&table.name_intern, table.columns.iter());
        let delete_table = delete_table(&table.name_intern);

        let table_rs_name = table.rs_name;
        let create_request_rs_name = &*table.create_request_rs_name;
        let update_request_rs_name = &*table.update_request_rs_name;
        let patch_request_rs_name = &*table.patch_request_rs_name;
        let request_error_rs_name = &*table.request_error_rs_name;
        // let table_record_rs_name = quote::format_ident!("{table_rs_name}Record");
        let table_rs_attrs = table.rs_attrs;
        let db_rs_name = &table.db_rs_name;
        let doc = fmt2::fmt! { { str } => "`" {table.name_intern} "`"};
        let table_token_stream = quote! {
            #[doc = #doc]
            #[derive(::serde::Serialize)]
            #(#table_rs_attrs)*
            pub struct #table_rs_name {
                #( #response_fields ),*
            }

            impl ::laraxum::model::Decode for #table_rs_name {
                type Decode = Self;
                #[inline]
                fn decode(decode: Self::Decode) -> Self {
                    decode
                }
            }

            impl ::laraxum::model::Encode for #table_rs_name {
                type Encode = Self;
                #[inline]
                fn encode(self) -> Self::Encode {
                    self
                }
            }

            impl ::laraxum::Db<#table_rs_name> for #db_rs_name {}

            impl ::laraxum::Table for #table_rs_name {
                type Db = #db_rs_name;
                type Response = #table_rs_name;
            }
        };

        // molecule vs collection
        let collection_model_token_stream = table.columns.is_collection().then(|| {
            let response_getters = table.columns.iter().map(|column| column.response_getter());
            let response_getter =
                &response_getter_compound(table.rs_name, response_getters.clone(), false);
            let response_getter = response_getter_fn(response_getter);
            let response_getter = &response_getter;

            let (response_getter_elements, response_getter_compounds) = flatten(response_getters);
            let response_getters = (&*response_getter_elements, &*response_getter_compounds);

            let get_all = get_all(&table.name_intern, &table.name_extern, response_getters);
            let get_all = transform_response_many(
                &quote! {
                    ::sqlx::query!(#get_all)
                },
                response_getter,
            );

            let create_columns = table.columns.iter();

            let create_request_fields = create_columns
                .clone()
                .filter_map(|column| column.request_field())
                .map(|field| {
                    request_field(
                        field.rs_name,
                        &*field.rs_ty,
                        field.attr.name.as_deref(),
                        field.rs_attrs,
                    )
                });

            let request_setters = create_columns
                .clone()
                .filter_map(|column| column.request_setter_molecule());

            let create_request_setters = create_columns
                .clone()
                .filter_map(|column| column.request_setter_molecule())
                .map(|setter| {
                    let rs_name = setter.rs_name;
                    request_setter(&quote! { request.#rs_name }, setter.is_optional)
                });

            let create_request_columns = create_columns
                .clone()
                .filter_map(|column| column.request_molecule());

            let create_one = create_one(&table.name_intern, create_request_columns);
            let create_one = quote! {
                ::sqlx::query!(#create_one, #(#create_request_setters,)*)
            };

            let request_setter_collections = table
                .columns
                .iter()
                .filter_map(|column| column.request_setter_collection());

            let create_request_setter_collections =
                request_setter_collections.clone().map(|column| {
                    let &stage3::RequestColumnSetterCollection {
                        rs_name,
                        aggregate_rs_name,
                        many_foreign_table_rs_name,
                    } = column;
                    quote! {{
                        <#many_foreign_table_rs_name as ::laraxum::ManyModel::<
                            #aggregate_rs_name,
                        >>::create_many(
                            db,
                            id,
                            &request.#rs_name,
                        ).await?;
                    }}
                });
            let create_request_setter_collections =
                quote! { #( #create_request_setter_collections )*};

            let collection_token_stream = quote! {
                #[derive(::serde::Deserialize)]
                pub struct #create_request_rs_name {
                    #( #create_request_fields )*
                }

                impl ::laraxum::Collection for #table_rs_name {
                    type CreateRequest = #create_request_rs_name;
                    type CreateRequestError = #request_error_rs_name;

                    async fn get_all(db: &Self::Db)
                        -> ::core::result::Result<
                            ::std::vec::Vec<Self::Response>,
                            ::laraxum::Error,
                        >
                    {
                        #get_all
                    }
                    async fn create_one(
                        db: &Self::Db,
                        request: Self::CreateRequest,
                    )
                        -> ::core::result::Result<
                            (),
                            ::laraxum::ModelError<Self::CreateRequestError>>
                    {
                        <
                            Self::CreateRequest
                            as ::laraxum::Request::<::laraxum::request::method::Create>
                        >::validate(&request)?;
                        let transaction = db.pool.begin().await?;
                        let response = #create_one;
                        let response = response.execute(&db.pool).await?;
                        let id = response.last_insert_id();
                        #create_request_setter_collections
                        transaction.commit().await?;
                        ::core::result::Result::Ok(())
                    }
                }
            };

            let validates = request_setters
                .filter_map(|column| {
                    let rs_name = column.rs_name;
                    let validates = [
                        column.validate.max_len.map(|max_len| {
                            let err_message = format!("max length is {max_len}");
                            quote! {
                                if #rs_name.len() <= #max_len {
                                    ::core::result::Result::Ok(())
                                } else { ::core::result::Result::Err(#err_message) }
                            }
                        }),
                        column.validate.min_len.map(|min_len| {
                            let err_message = format!("min length is {min_len}");
                            quote! {
                                if #rs_name.len() >= #min_len {
                                    ::core::result::Result::Ok(())
                                } else { ::core::result::Result::Err(#err_message) }
                            }
                        }),
                        column.validate.func.as_ref().map(|func| {
                            quote! {
                                (#func)(#rs_name)
                            }
                        }),
                        column.validate.matches.as_ref().and_then(|matches| {
                            let end = matches.end.as_deref().map(|end| match matches.limits {
                                syn::RangeLimits::Closed(_) => {
                                    let lte = end.to_token_stream();
                                    let err_message = format!("must less than or equal to {lte}");
                                    quote! {
                                        if #rs_name <= &#lte {
                                            ::core::result::Result::Ok(())
                                        } else { ::core::result::Result::Err(#err_message) }
                                    }
                                }
                                syn::RangeLimits::HalfOpen(_) => {
                                    let lt = end.to_token_stream();
                                    let err_message = format!("must be less than {lt}");
                                    quote! {
                                        if #rs_name < &#lt {
                                            ::core::result::Result::Ok(())
                                        } else { ::core::result::Result::Err(#err_message) }
                                    }
                                }
                            });
                            let start = matches.start.as_deref().map(|gte| {
                                let ok = std::cell::LazyCell::new(|| {
                                    quote! { ::core::result::Result::Ok(()) }
                                });
                                let end = end.as_ref().unwrap_or_else(|| &*ok);

                                let gte = gte.to_token_stream();
                                let err_message = format!("must be greater than or equal to {gte}");
                                quote! {
                                    if #rs_name >= &#gte {
                                        #end
                                    } else { ::core::result::Result::Err(#err_message) }
                                }
                            });
                            start.or(end)
                        }),
                    ];
                    let validates = validates.into_iter().flatten();
                    let not_empty = validates.clone().next().is_some();
                    not_empty.then(|| {
                        let value = quote! {
                            &self.#rs_name
                        };
                        let validates = validates.map(|validate| {
                            quote! {
                                if let ::core::result::Result::Err(err) = #validate {
                                    ::laraxum::request::error_builder::<
                                        (),
                                        Self::Error,
                                    >(
                                        &mut e,
                                        |e| e.#rs_name.push(err),
                                    );
                                }
                            }
                        });
                        let validates = quote! { #( #validates )* };
                        (column, value, validates)
                    })
                })
                .collect::<Vec<_>>();

            let request_error_fields = validates.iter().map(|(column, _, _)| {
                let rs_name = column.rs_name;
                quote! {
                    #[serde(skip_serializing_if = "<[&str]>::is_empty")]
                    pub #rs_name: ::std::vec::Vec::<&'static str>,
                }
            });

            let create_request_validates = validates.iter();

            let create_request_validates =
                create_request_validates.map(|(column, value, validate)| {
                    let rs_name = column.rs_name;
                    if column.is_optional {
                        quote! {
                            if let ::core::option::Option::Some(#rs_name) = #value {
                                #validate
                            }
                        }
                    } else {
                        quote! {
                            let #rs_name = #value;
                            #validate
                        }
                    }
                });

            let collection_token_stream = quote! {
                #collection_token_stream

                #[derive(Default, ::serde::Serialize)]
                pub struct #request_error_rs_name {
                    #( #request_error_fields )*
                }
                impl ::core::convert::From<#request_error_rs_name>
                    for ::laraxum::ModelError<#request_error_rs_name>
                {
                    fn from(value: #request_error_rs_name) -> Self {
                        Self::UnprocessableEntity(value)
                    }
                }

                impl ::laraxum::Request::<::laraxum::request::method::Create>
                    for #create_request_rs_name
                {
                    type Error = #request_error_rs_name;
                    fn validate(&self) -> ::core::result::Result::<(), Self::Error> {
                        let mut e = ::core::result::Result::Ok(());
                        #( #create_request_validates )*
                        e
                    }
                }
            };

            let aggregates = table
                .columns
                .iter()
                .filter_map(|column| match column {
                    stage3::ColumnRef::Molecule(molecule) => Some(molecule),
                    stage3::ColumnRef::Collection(_) => None,
                })
                .flat_map(|column| {
                    let filter_rs_ty_owned = column.response.field.rs_ty;
                    let is_borrowed = column.borrow.is_some();
                    let lifetime = is_borrowed.then(|| quote! { 'b });

                    let auto_lifetime = is_borrowed.then(|| quote! { '_ });
                    let filter_rs_ty = if let Some(borrow) = column.borrow {
                        let borrow = borrow.unwrap_or(filter_rs_ty_owned);
                        syn::parse_quote! {
                            &'b #borrow
                        }
                    } else {
                        filter_rs_ty_owned.clone()
                    };
                    let name_intern = column.name_intern();
                    let is_unique = column.create.ty.is_unique();

                    let table_name_intern = &*table.name_intern;
                    let table_name_extern = &*table.name_extern;
                    let column_response_name = column.response.field.rs_name.to_string();
                    column.aggregates.iter().map(move |aggregate| {
                        let is_one = is_unique && aggregate.filter.is_eq();
                        let aggregate_rs_name = &aggregate.rs_name;

                        let filter = aggregate.filter.parameter().map(|parameter_name| {
                            (
                                quote::format_ident!("filter"),
                                quote::format_ident!(
                                    "filter_{}_{}",
                                    column_response_name,
                                    parameter_name
                                ),
                                &filter_rs_ty,
                                filter_rs_ty_owned,
                            )
                        });
                        let limit =
                            aggregate
                                .limit
                                .parameter()
                                .filter(|_| !is_one)
                                .map(|parameter_name| {
                                    (
                                        quote::format_ident!("{}", parameter_name),
                                        syn::parse_quote! {
                                            u64
                                        },
                                    )
                                });
                        let sort = aggregate.is_sort.then(|| -> (Ident, Ident, Type) {
                            (
                                quote::format_ident!("sort"),
                                quote::format_ident!("sort_{}", column_response_name),
                                syn::parse_quote! {
                                    ::laraxum::model::Sort
                                },
                            )
                        });

                        let filter_field = filter.as_ref().map(|(short_name, name, rs_ty, _)| {
                            let name = name.to_string();
                            quote! {
                                #[serde(rename = #name)]
                                pub #short_name: #rs_ty,
                            }
                        });
                        let limit_field = limit.as_ref().map(|(name, rs_ty)| {
                            quote! {
                                pub #name: #rs_ty,
                            }
                        });
                        let sort_field = sort.as_ref().map(|(short_name, name, rs_ty)| {
                            let name = name.to_string();
                            quote! {
                                #[serde(rename = #name)]
                                pub #short_name: #rs_ty,
                            }
                        });

                        let aggregate_struct_token_stream = quote! {
                            #[derive(::serde::Deserialize)]
                            pub struct #aggregate_rs_name<#lifetime> {
                                #filter_field
                                #limit_field
                                #sort_field
                            }
                        };
                        let filter_parameter = filter.as_ref().map(|(short_name, _, _, _)| {
                            quote! { request.#short_name, }
                        });
                        let limit_parameter =
                            limit.as_ref().map(|(name, _)| match aggregate.limit {
                                stage3::ColumnAttrAggregateLimit::Page { per_page } => {
                                    quote! { request.#name * #per_page, }
                                }
                                _ => {
                                    quote! { request.#name, }
                                }
                            });

                        let response = if aggregate.is_sort {
                            let (get_sort_asc, get_sort_desc) = get_sort_asc_desc(
                                table_name_intern,
                                table_name_extern,
                                response_getters,
                                Some((aggregate.filter, name_intern)),
                                name_intern,
                                Some(aggregate.limit),
                                is_one,
                            );

                            let response_sort_asc = quote! {
                                ::sqlx::query!(#get_sort_asc, #filter_parameter #limit_parameter)
                            };
                            let response_sort_asc =
                                transform_response(&response_sort_asc, response_getter, is_one);

                            let response_sort_desc = quote! {
                                ::sqlx::query!(#get_sort_desc, #filter_parameter #limit_parameter)
                            };
                            let response_sort_desc =
                                transform_response(&response_sort_desc, response_getter, is_one);
                            quote! {
                                match request.sort {
                                    ::laraxum::model::Sort::Ascending => #response_sort_asc,
                                    ::laraxum::model::Sort::Descending => #response_sort_desc,
                                }
                            }
                        } else {
                            let get = get(
                                table_name_intern,
                                table_name_extern,
                                response_getters,
                                Some((aggregate.filter, name_intern)),
                                None,
                                Some(aggregate.limit),
                                is_one,
                            );
                            let response = quote! {
                                ::sqlx::query!(#get, #filter_parameter #limit_parameter)
                            };
                            transform_response(&response, response_getter, is_one)
                        };

                        let aggregate_impl_token_stream = if is_one {
                            quote! {
                                impl
                                    ::laraxum::AggregateOne<#aggregate_rs_name<#auto_lifetime>>
                                    for #table_rs_name
                                {
                                    type OneRequest<'b> = #aggregate_rs_name<#lifetime>;
                                    type OneResponse = Self;
                                    async fn aggregate_one<'a>(
                                        db: &Self::Db,
                                        request: Self::OneRequest<'a>,
                                    )
                                        -> ::core::result::Result<
                                            Self::OneResponse,
                                            ::laraxum::Error,
                                        >
                                    {
                                        #response
                                    }
                                }
                            }
                        } else {
                            quote! {
                                impl
                                    ::laraxum::AggregateMany<#aggregate_rs_name<#auto_lifetime>>
                                    for #table_rs_name
                                {
                                    type OneRequest<'b> = #aggregate_rs_name<#lifetime>;
                                    type ManyResponse = Self;
                                    async fn aggregate_many<'a>(
                                        db: &Self::Db,
                                        request: Self::OneRequest<'a>,
                                    )
                                        -> ::core::result::Result<
                                            ::std::vec::Vec<Self::ManyResponse>,
                                            ::laraxum::Error,
                                        >
                                    {
                                        #response
                                    }
                                }
                            }
                        };
                        let aggregate_struct_token_stream = quote! {
                            #aggregate_struct_token_stream
                            #aggregate_impl_token_stream
                        };

                        // only include variant in enum if:
                        // - there is a controller aggregate query
                        // - this variant is in the controller query aggregate
                        let aggregate_enum = table.aggregate_rs_name.filter(|_| aggregate.is_pub);
                        let aggregate_enum = aggregate_enum.map(|table_aggregate_rs_name| {
                            let filter_field = filter.as_ref().map(|(_, name, _, rs_ty_owned)| {
                                quote! {
                                    #name: #rs_ty_owned,
                                }
                            });
                            let limit_field = limit.as_ref().map(|(name, rs_ty)| {
                                quote! {
                                    #name: #rs_ty,
                                }
                            });
                            let sort_field = sort.as_ref().map(|(_, name, rs_ty)| {
                                quote! {
                                    #name: #rs_ty,
                                }
                            });

                            let aggregate_variant_def_token_stream = quote! {
                                #aggregate_rs_name {
                                    #filter_field
                                    #limit_field
                                    #sort_field
                                }
                            };

                            let filter_get = filter.as_ref().map(|(short_name, name, _, _)| {
                                if is_borrowed {
                                    quote! {
                                        #name: ref #short_name,
                                    }
                                } else {
                                    quote! {
                                        #name: #short_name,
                                    }
                                }
                            });
                            let limit_get = limit.as_ref().map(|(name, _)| {
                                quote! {
                                    #name,
                                }
                            });
                            let sort_get = sort.as_ref().map(|(short_name, name, _)| {
                                quote! {
                                    #name: #short_name,
                                }
                            });

                            let filter_set = filter.as_ref().map(|(short_name, _, _, _)| {
                                quote! {
                                    #short_name,
                                }
                            });
                            let limit_set = limit.as_ref().map(|(name, _)| {
                                quote! {
                                    #name,
                                }
                            });
                            let sort_set = sort.as_ref().map(|(short_name, _, _)| {
                                quote! {
                                    #short_name,
                                }
                            });

                            let aggregate_get = quote! {
                                #table_aggregate_rs_name::#aggregate_rs_name {
                                    #filter_get
                                    #limit_get
                                    #sort_get
                                }
                            };
                            let aggregate_set = quote! {
                                #aggregate_rs_name {
                                    #filter_set
                                    #limit_set
                                    #sort_set
                                }
                            };
                            let aggregate_variant_match_token_stream = if is_one {
                                quote! {
                                    #aggregate_get => {
                                        <#table_rs_name as
                                            ::laraxum::AggregateOne<
                                                #aggregate_rs_name<#auto_lifetime>
                                            >
                                        >::aggregate_one_vec(db, #aggregate_set).await
                                    }
                                }
                            } else {
                                quote! {
                                    #aggregate_get => {
                                        <#table_rs_name as
                                            ::laraxum::AggregateMany<
                                                #aggregate_rs_name<#auto_lifetime>
                                            >
                                        >::aggregate_many(db, #aggregate_set).await
                                    }
                                }
                            };
                            let aggregate_variants = [
                                filter.map(|(_, name, _, rs_ty_owned)| (name, rs_ty_owned.clone())),
                                limit.map(|(name, rs_ty)| (name, rs_ty)),
                                sort.map(|(_, name, rs_ty)| (name, rs_ty)),
                            ];
                            let aggregate_variant_type_signature =
                                (aggregate_rs_name, aggregate_variants);
                            (
                                aggregate_variant_def_token_stream,
                                aggregate_variant_match_token_stream,
                                aggregate_variant_type_signature,
                            )
                        });

                        (aggregate_struct_token_stream, aggregate_enum)
                    })
                });

            let collection_token_stream =
                if let Some(table_aggregate_rs_name) = table.aggregate_rs_name {
                    let aggregates = aggregates.collect::<Vec<_>>();
                    let aggregate_token_streams = aggregates.iter().map(|(i, _)| i);
                    let aggregate_variants = aggregates
                        .iter()
                        .filter_map(|(_, aggregate_variant)| aggregate_variant.as_ref());
                    let aggregate_variant_def_token_streams =
                        aggregate_variants.clone().map(|(i, _, _)| i);
                    let aggregate_variant_match_token_streams =
                        aggregate_variants.clone().map(|(_, i, _)| i);
                    let aggregate_variant_type_signatures =
                        aggregate_variants.map(|(_, _, (aggregate_rs_name, fields))| {
                            (
                                *aggregate_rs_name,
                                fields.iter().flat_map(|field| {
                                    field.as_ref().map(|(rs_name, rs_ty)| (rs_name, rs_ty))
                                }),
                            )
                        });

                    let impl_deserialize_for_table_aggregate = impl_deserialize_for_untagged_enum(
                        table_aggregate_rs_name,
                        aggregate_variant_type_signatures,
                        Some(table_aggregate_rs_name),
                    );

                    quote! {
                        #collection_token_stream
                        #( #aggregate_token_streams )*

                        pub enum #table_aggregate_rs_name {
                            #( #aggregate_variant_def_token_streams, )*
                            #table_aggregate_rs_name,
                        }
                        #impl_deserialize_for_table_aggregate
                        impl ::laraxum::AggregateMany<#table_aggregate_rs_name> for #table_rs_name {
                            type OneRequest<'b> = #table_aggregate_rs_name;
                            type ManyResponse = Self;
                            async fn aggregate_many<'a>(
                                db: &Self::Db,
                                request: Self::OneRequest<'a>,
                            )
                                -> ::core::result::Result<
                                    ::std::vec::Vec<Self::ManyResponse>,
                                    ::laraxum::Error,
                                >
                            {
                                match request {
                                    #( #aggregate_variant_match_token_streams, )*
                                    #table_aggregate_rs_name::#table_aggregate_rs_name => {
                                        <#table_rs_name as ::laraxum::Collection>::get_all(db).await
                                    }
                                }
                            }
                        }
                    }
                } else {
                    let aggregate_token_streams = aggregates.map(|(i, _)| i);

                    quote! {
                        #collection_token_stream
                        #( #aggregate_token_streams )*
                    }
                };

            let Some(table_id) = table.columns.model() else {
                return collection_token_stream;
            };

            let table_id_rs_ty = table_id.response.field.rs_ty;
            let table_id_name = table_id.create.name;
            let table_id_name_intern = table_id.response.getter.name_intern();

            let get_one = get_one(
                &table.name_intern,
                &table.name_extern,
                response_getters,
                (stage3::ColumnAttrAggregateFilter::Eq, table_id_name_intern),
            );
            let get_one = transform_response_one(
                &quote! {
                    ::sqlx::query!(#get_one, id)
                },
                response_getter,
            );

            let update_patch_columns = table.columns.iter().filter(|column| column.is_mut());

            let update_patch_request_fields = update_patch_columns
                .clone()
                .filter_map(|column| column.request_field());
            let update_request_fields = update_patch_request_fields.clone().map(|field| {
                request_field(
                    field.rs_name,
                    &*field.rs_ty,
                    field.attr.name.as_deref(),
                    field.rs_attrs,
                )
            });
            let patch_request_fields = update_patch_request_fields.clone().map(|field| {
                let rs_ty = &*field.rs_ty;
                let rs_ty = quote! {
                    ::core::option::Option<#rs_ty>
                };
                request_field(
                    field.rs_name,
                    &rs_ty,
                    field.attr.name.as_deref(),
                    field.rs_attrs,
                )
            });

            let update_request_setters = update_patch_columns
                .clone()
                .filter_map(|column| column.request_setter_molecule())
                .map(|setter| {
                    let rs_name = setter.rs_name;
                    request_setter(&quote! { request.#rs_name }, setter.is_optional)
                });

            let update_patch_request_columns = update_patch_columns
                .clone()
                .filter_map(|column| column.request_molecule());

            let update_one = update_one(
                &table.name_intern,
                table_id_name,
                update_patch_request_columns.clone(),
            );
            let update_one = quote! {
                ::sqlx::query!(#update_one, #( #update_request_setters, )* id)
            };

            let patch_one = update_patch_request_columns.clone().map(|request| {
                let patch_one = patch_one(&table.name_intern, table_id_name, request);
                if let Some(setter) = request.setter() {
                    let rs_name = setter.rs_name;
                    let setter = request_setter(&rs_name.to_token_stream(), setter.is_optional);
                    quote! {
                        if let ::core::option::Option::Some(#rs_name) = request.#rs_name {
                            let response = ::sqlx::query!(#patch_one, #setter, id);
                            response.execute(&db.pool).await?;
                        }
                    }
                } else {
                    quote! {
                        let response = ::sqlx::query!(#patch_one, id);
                        response.execute(&db.pool).await?;
                    }
                }
            });

            let delete_one = delete_one(&table.name_intern, table_id_name);

            let update_request_setter_collections =
                request_setter_collections.clone().map(|column| {
                    let &stage3::RequestColumnSetterCollection {
                        rs_name,
                        aggregate_rs_name,
                        many_foreign_table_rs_name,
                    } = column;
                    quote! {{
                        <#many_foreign_table_rs_name as ::laraxum::ManyModel::<
                            #aggregate_rs_name,
                        >>::update_many(
                            db,
                            id,
                            &request.#rs_name,
                        ).await?;
                    }}
                });
            let update_request_setter_collections =
                quote! { #( #update_request_setter_collections )*};
            let patch_request_setter_collections =
                request_setter_collections.clone().map(|column| {
                    let &stage3::RequestColumnSetterCollection {
                        rs_name,
                        aggregate_rs_name,
                        many_foreign_table_rs_name,
                    } = column;
                    quote! { if let ::core::option::Option::Some(#rs_name) = &request.#rs_name {
                        <#many_foreign_table_rs_name as ::laraxum::ManyModel::<
                            #aggregate_rs_name,
                        >>::update_many(
                            db,
                            id,
                            #rs_name,
                        ).await?;
                    };}
                });
            let patch_request_setter_collections =
                quote! { #( #patch_request_setter_collections )*};
            let delete_request_setter_collections =
                request_setter_collections.clone().map(|column| {
                    let &stage3::RequestColumnSetterCollection {
                        rs_name: _,
                        aggregate_rs_name,
                        many_foreign_table_rs_name,
                    } = column;
                    quote! {{
                        <#many_foreign_table_rs_name as ::laraxum::ManyModel::<
                            #aggregate_rs_name,
                        >>::delete_many(
                            db,
                            id,
                        ).await?;
                    }}
                });
            let delete_request_setter_collections =
                quote! { #( #delete_request_setter_collections )*};

            let update_patch_request_validates =
                validates.iter().filter(|(column, _, _)| column.is_mut);
            let update_request_validates =
                update_patch_request_validates
                    .clone()
                    .map(|(column, value, validate)| {
                        let rs_name = column.rs_name;
                        if column.is_optional {
                            quote! {
                                if let ::core::option::Option::Some(#rs_name) = #value {
                                    #validate
                                }
                            }
                        } else {
                            quote! {
                                let #rs_name = #value;
                                #validate
                            }
                        }
                    });
            let patch_request_validates =
                update_patch_request_validates.map(|(column, value, validate)| {
                    let rs_name = column.rs_name;
                    if column.is_optional {
                        quote! {
                            if let ::core::option::Option::Some(
                                ::core::option::Option::Some(#rs_name)
                            ) = #value {
                                #validate
                            }
                        }
                    } else {
                        quote! {
                            if let ::core::option::Option::Some(#rs_name) = #value {
                                #validate
                            }
                        }
                    }
                });

            quote! {
                #collection_token_stream

                #[derive(::serde::Deserialize)]
                pub struct #update_request_rs_name {
                    #( #update_request_fields )*
                }
                #[derive(::serde::Deserialize)]
                pub struct #patch_request_rs_name {
                    #( #patch_request_fields )*
                }

                impl ::laraxum::Request::<::laraxum::request::method::Update>
                    for #update_request_rs_name
                {
                    type Error = #request_error_rs_name;
                    fn validate(&self) -> ::core::result::Result::<(), Self::Error> {
                        let mut e = ::core::result::Result::Ok(());
                        #( #update_request_validates )*
                        e
                    }
                }
                impl ::laraxum::Request::<::laraxum::request::method::Patch>
                    for #patch_request_rs_name
                {
                    type Error = #request_error_rs_name;
                    fn validate(&self) -> ::core::result::Result::<(), Self::Error> {
                        let mut e = ::core::result::Result::Ok(());
                        #( #patch_request_validates )*
                        e
                    }
                }

                impl ::laraxum::Model for #table_rs_name {
                    type Id = #table_id_rs_ty;
                    type UpdateRequest = #update_request_rs_name;
                    type UpdateRequestError = #request_error_rs_name;
                    type PatchRequest = #patch_request_rs_name;
                    type PatchRequestError = #request_error_rs_name;

                    async fn get_one(
                        db: &Self::Db,
                        id: Self::Id,
                    )
                        -> ::core::result::Result<
                            Self::Response,
                            ::laraxum::Error,
                        >
                    {
                        #get_one
                    }
                    async fn create_get_one(
                        db: &Self::Db,
                        request: Self::CreateRequest,
                    )
                        -> ::core::result::Result<
                            Self::Response,
                            ::laraxum::ModelError<Self::CreateRequestError>
                        >
                    {
                        <
                            Self::CreateRequest
                            as ::laraxum::Request::<::laraxum::request::method::Create>
                        >::validate(&request)?;
                        let transaction = db.pool.begin().await?;
                        let response = #create_one;
                        let response = response.execute(&db.pool).await?;
                        let id = response.last_insert_id();
                        #create_request_setter_collections
                        transaction.commit().await?;
                        let response = Self::get_one(db, id).await?;
                        ::core::result::Result::Ok(response)
                    }
                    async fn update_one(
                        db: &Self::Db,
                        request: Self::UpdateRequest,
                        id: Self::Id,
                    )
                        -> ::core::result::Result<
                            (),
                            ::laraxum::ModelError<Self::UpdateRequestError>,
                        >
                    {
                        <
                            Self::UpdateRequest
                            as ::laraxum::Request::<::laraxum::request::method::Update>
                        >::validate(&request)?;
                        let transaction = db.pool.begin().await?;
                        let response = #update_one;
                        response.execute(&db.pool).await?;
                        #update_request_setter_collections
                        transaction.commit().await?;
                        ::core::result::Result::Ok(())
                    }
                    async fn patch_one(
                        db: &Self::Db,
                        request: Self::PatchRequest,
                        id: Self::Id,
                    )
                        -> ::core::result::Result<
                            (),
                            ::laraxum::ModelError<Self::PatchRequestError>,
                        >
                    {
                        <
                            Self::PatchRequest
                            as ::laraxum::Request::<::laraxum::request::method::Patch>
                        >::validate(&request)?;
                        let transaction = db.pool.begin().await?;
                        #( #patch_one )*
                        #patch_request_setter_collections
                        transaction.commit().await?;
                        ::core::result::Result::Ok(())
                    }
                    async fn delete_one(
                        db: &Self::Db,
                        id: Self::Id,
                    )
                        -> ::core::result::Result<
                            (),
                            ::laraxum::Error,
                        >
                    {
                        let response = ::sqlx::query!(#delete_one, id);
                        let transaction = db.pool.begin().await?;
                        response.execute(&db.pool).await?;
                        #delete_request_setter_collections
                        transaction.commit().await?;
                        ::core::result::Result::Ok(())
                    }
                }
            }
        });

        let controller_token_stream = table.columns.controller().map(|controller| {
            let auth = controller
                .auth
                .as_deref()
                .map_or_else(|| quote! { () }, quote::ToTokens::to_token_stream);

            let get_many_request_query = table
                .aggregate_rs_name
                .map_or_else(|| quote! { () }, quote::ToTokens::to_token_stream);

            let get_many = table.aggregate_rs_name.map(|aggregate_rs_name_rs_name| {
                quote! {
                    async fn get_many(
                        ::axum::extract::State(state):
                            ::axum::extract::State<::std::sync::Arc<Self::State>>,
                        ::laraxum::AuthToken(_): ::laraxum::AuthToken<Self::Auth>,
                        ::axum::extract::Query(query):
                            ::axum::extract::Query<Self::GetManyRequestQuery>,
                    ) -> ::core::result::Result<
                            ::laraxum::Json<::std::vec::Vec<Self::Response>>,
                            ::laraxum::Error,
                        >
                    {
                        let records = <
                            #table_rs_name as ::laraxum::AggregateMany<#aggregate_rs_name_rs_name>
                        >::aggregate_many(&*state, query).await?;
                        ::core::result::Result::Ok(::laraxum::Json(records))

                    }
                }
            });

            quote! {
                impl ::laraxum::Controller for #table_rs_name {
                    type State = #db_rs_name;
                    type Auth = #auth;
                    type GetManyRequestQuery = #get_many_request_query;
                    #get_many
                }
            }
        });

        let many_model_token_stream = table.columns.many_model().map(|(a, b)| {
            fn many_model(
                table: &stage3::Table,
                one: &stage3::ColumnMolecule,
                many: &stage3::ColumnMolecule,
            ) -> proc_macro2::TokenStream {
                let aggregate_rs_ty = many.struct_name.map_or_else(
                    || one.response.field.rs_ty.to_token_stream(),
                    |struct_name| struct_name.to_token_stream(),
                );
                let one_request_rs_ty = one
                    .request
                    .as_ref()
                    .and_then(|request| request.field())
                    .map(|field| &*field.rs_ty);
                let many_request_rs_ty = many
                    .request
                    .as_ref()
                    .and_then(|request| request.field())
                    .map(|field| &*field.rs_ty);
                let many_response_rs_ty = many.response.field.rs_ty;

                let many_response_getter =
                    stage3::ResponseColumnGetterRef::Molecule(&many.response.getter);

                let response_getter = response_getter(many_response_getter, false);
                let response_getter = response_getter_fn(&response_getter);

                let (response_getter_column_elements, response_getter_column_compounds) =
                    flatten(core::iter::once(many_response_getter));

                let get_many = get_many(
                    &table.name_intern,
                    &table.name_extern,
                    (
                        &response_getter_column_elements,
                        &response_getter_column_compounds,
                    ),
                    (stage3::ColumnAttrAggregateFilter::Eq, one.name_intern()),
                );
                let get_many_response = transform_response_many(
                    &quote! {
                        ::sqlx::query!(#get_many, one)
                    },
                    &response_getter,
                );

                let request_columns = [&one.request, &many.request].into_iter().flatten();
                let create_one = create_one(&table.name_intern, request_columns);
                let delete_many = delete_one(&table.name_intern, one.name());

                let table_rs_name = table.rs_name;

                quote! {
                    impl ::laraxum::ManyModel<#aggregate_rs_ty> for #table_rs_name {
                        type OneRequest = #one_request_rs_ty;
                        type ManyRequest = #many_request_rs_ty;
                        type ManyResponse = #many_response_rs_ty;

                        async fn get_many(
                            db: &Self::Db,
                            one: Self::OneRequest,
                        )
                            -> ::core::result::Result<
                                ::std::vec::Vec<Self::ManyResponse>,
                                ::laraxum::Error,
                            >
                        {
                            #get_many_response
                        }
                        async fn create_many(
                            db: &Self::Db,
                            one: Self::OneRequest,
                            many: &[Self::ManyRequest],
                        )
                            -> ::core::result::Result<
                                (),
                                ::laraxum::Error,
                            >
                        {
                            for many in many {
                                let response = ::sqlx::query!(#create_one, one, many);
                                response.execute(&db.pool).await?;
                            }
                            ::core::result::Result::Ok(())
                        }
                        async fn update_many(
                            db: &Self::Db,
                            one: Self::OneRequest,
                            many: &[Self::ManyRequest],
                        )
                            -> ::core::result::Result<
                                (),
                                ::laraxum::Error,
                            >
                        {
                            <
                                Self as ::laraxum::ManyModel<#aggregate_rs_ty>
                            >::delete_many(db, one).await?;
                            <
                                Self as ::laraxum::ManyModel<#aggregate_rs_ty>
                            >::create_many(db, one, many).await?;
                            ::core::result::Result::Ok(())
                        }
                        async fn delete_many(
                            db: &Self::Db,
                            one: Self::OneRequest
                        )
                            -> ::core::result::Result<
                                (),
                                ::laraxum::Error,
                            >
                        {
                            let response = ::sqlx::query!(#delete_many, one);
                            response.execute(&db.pool).await?;
                            ::core::result::Result::Ok(())
                        }
                    }
                }
            }

            let a_token_stream = many_model(&table, a, b);
            let b_token_stream = many_model(&table, b, a);
            quote! {
                #a_token_stream
                #b_token_stream
            }
        });

        let structs = table
            .columns
            .iter()
            .filter_map(|column| column.struct_name())
            .map(|struct_name| {
                quote! {
                    pub struct #struct_name;
                }
            });

        let table_token_stream = quote! {
            #table_token_stream
            #collection_model_token_stream
            #controller_token_stream
            #many_model_token_stream
            #( #structs )*
        };

        Self {
            token_stream: table_token_stream,
            migration_up: create_table,
            migration_down: delete_table,
        }
    }
}

pub use proc_macro2::TokenStream as Db;
#[expect(clippy::fallible_impl_from, clippy::unwrap_used)]
impl From<stage3::Db<'_>> for Db {
    fn from(db: stage3::Db) -> Self {
        let tables: Vec<Table> = db.tables.into_iter().map(Table::from).collect();

        let tables_token_stream = tables.iter().map(|table| &table.token_stream);

        let migration_up = fmt2::fmt! { { str } =>
            "BEGIN TRANSACTION;"
            @..(tables.iter() => |table| {table.migration_up})
            "COMMIT;"
        };
        let migration_down = fmt2::fmt! { { str } =>
            "BEGIN TRANSACTION;"
            @..(tables.iter().rev() => |table| {table.migration_down})
            "COMMIT;"
        };
        let migration_up_full = fmt2::fmt! { { str } =>
            "CREATE DATABASE IF NOT EXISTS " {db.name} ";"
            {migration_up}
        };
        let migration_down_full = fmt2::fmt! { { str } =>
            "DROP DATABASE " {db.name} ";"
        };

        let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let root = root.join("laraxum");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("migration_up.sql"), &migration_up).unwrap();
        std::fs::write(root.join("migration_down.sql"), &migration_down).unwrap();
        std::fs::write(root.join("migration_up_full.sql"), &migration_up_full).unwrap();
        std::fs::write(root.join("migration_down_full.sql"), &migration_down_full).unwrap();

        let db_ident = &db.rs_name;
        let db_pool_type = {
            #[cfg(feature = "mysql")]
            {
                quote! { ::sqlx::MySql }
            }
            #[cfg(feature = "sqlite")]
            {
                quote! { ::sqlx::Sqlite }
            }
            #[cfg(feature = "postgres")]
            {
                quote! { ::sqlx::Postgres }
            }
        };

        quote! {
            /// ```sql
            #[doc = #migration_up_full]
            /// ```
            pub struct #db_ident {
                pub pool: ::sqlx::Pool<#db_pool_type>,
            }

            impl ::laraxum::Connect for #db_ident {
                type Error = ::sqlx::Error;
                async fn connect() -> ::core::result::Result<Self, Self::Error> {
                    let connect_options = ::laraxum::model::database_url()
                        .map(|url| {
                            <
                                <
                                    <
                                        #db_pool_type as ::sqlx::Database
                                    >::Connection as ::sqlx::Connection
                                >::Options as ::core::str::FromStr
                            >::from_str(&url)
                        })
                        .transpose()?
                        .unwrap_or_default();
                    let pool_options = ::sqlx::pool::PoolOptions::<#db_pool_type>::new();
                    let pool = pool_options.connect_with(connect_options).await?;
                    ::core::result::Result::Ok(Self { pool })
                }
            }

            impl ::core::ops::Deref for #db_ident {
                type Target = Self;
                fn deref(&self) -> &Self::Target {
                    self
                }
            }

            #(#tables_token_stream)*
        }
    }
}
