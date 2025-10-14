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
    #[expect(clippy::unused_self)]
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

impl stage3::Ty<'_> {
    fn ty(&self) -> Cow<'static, str> {
        match self {
            Self::Compound(compound) => Cow::Borrowed(compound.ty()),
            Self::Element(element) => element.ty(),
        }
    }
}
impl fmt2::write_to::WriteTo for stage3::Ty<'_> {
    fn write_to<W>(&self, w: &mut W) -> Result<(), W::Error>
    where
        W: fmt2::write::Write + ?Sized,
    {
        fmt2::fmt! { (? w) => {self.ty()} }?;
        if self.optional() {
            fmt2::fmt! { (? w) => " NOT NULL" }?;
        }
        if self.unique() && !self.is_id() {
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

impl stage3::RequestColumnOne<'_> {
    pub const fn request_setter_column(&self) -> Option<(&str, &str)> {
        match self {
            Self::Some {
                setter: stage3::RequestColumnSetterOne { name, .. },
                ..
            } => Some((name, "?")),
            Self::AutoTime { name, time_ty } => Some((name, time_ty.current_time_func())),
            Self::None => None,
        }
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
        if self.parent_optional || self.element.optional {
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
    index_filter: Option<(stage3::ColumnAttrIndexFilter, &str)>,
    index_sort: Option<(Sort, &str)>,
    index_limit: Option<stage3::ColumnAttrIndexLimit>,
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
    if let Some((index_filter, filter_column_name_intern)) = index_filter {
        match index_filter {
            stage3::ColumnAttrIndexFilter::None => {}
            stage3::ColumnAttrIndexFilter::Eq => {
                fmt2::fmt! { (get) => " WHERE " {filter_column_name_intern} "=?" };
            }
            stage3::ColumnAttrIndexFilter::Like => {
                fmt2::fmt! { (get) => " WHERE " {filter_column_name_intern} " LIKE CONCAT('%', ?, '%')" };
            }
            stage3::ColumnAttrIndexFilter::Gt => {
                fmt2::fmt! { (get) => " WHERE " {filter_column_name_intern} ">?" };
            }
            stage3::ColumnAttrIndexFilter::Lt => {
                fmt2::fmt! { (get) => " WHERE " {filter_column_name_intern} "<?" };
            }
            stage3::ColumnAttrIndexFilter::Gte => {
                fmt2::fmt! { (get) => " WHERE " {filter_column_name_intern} ">=?" };
            }
            stage3::ColumnAttrIndexFilter::Lte => {
                fmt2::fmt! { (get) => " WHERE " {filter_column_name_intern} "<=?" };
            }
        }
    }
    if let Some((index_sort, sort_column_name_intern)) = index_sort {
        match index_sort {
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
    } else if let Some(index_limit) = index_limit {
        match index_limit {
            stage3::ColumnAttrIndexLimit::None => {}
            stage3::ColumnAttrIndexLimit::Limit => {
                fmt2::fmt! { (get) => " LIMIT ?" };
            }
            stage3::ColumnAttrIndexLimit::Page { per_page } => {
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
    response_getter_columns: (
        &[ResponseColumnGetterElement],
        &[&stage3::ResponseColumnGetterCompound],
    ),
) -> String {
    get(
        table_name_intern,
        table_name_extern,
        response_getter_columns,
        None,
        None,
        None,
        false,
    )
}
fn get_one(
    table_name_intern: &str,
    table_name_extern: &str,
    response_getter_columns: (
        &[ResponseColumnGetterElement],
        &[&stage3::ResponseColumnGetterCompound],
    ),
    index_filter: (stage3::ColumnAttrIndexFilter, &str),
) -> String {
    get(
        table_name_intern,
        table_name_extern,
        response_getter_columns,
        Some(index_filter),
        None,
        None,
        true,
    )
}
fn get_many(
    table_name_intern: &str,
    table_name_extern: &str,
    response_getter_columns: (
        &[ResponseColumnGetterElement],
        &[&stage3::ResponseColumnGetterCompound],
    ),
    index_filter: (stage3::ColumnAttrIndexFilter, &str),
) -> String {
    get(
        table_name_intern,
        table_name_extern,
        response_getter_columns,
        Some(index_filter),
        None,
        None,
        false,
    )
}
fn get_sort_asc_desc(
    table_name_intern: &str,
    table_name_extern: &str,
    response_getter_columns: (
        &[ResponseColumnGetterElement],
        &[&stage3::ResponseColumnGetterCompound],
    ),
    index_filter: Option<(stage3::ColumnAttrIndexFilter, &str)>,
    sort_column_name_intern: &str,
    index_limit: Option<stage3::ColumnAttrIndexLimit>,
    is_one: bool,
) -> (String, String) {
    (
        get(
            table_name_intern,
            table_name_extern,
            response_getter_columns,
            index_filter,
            Some((Sort::Ascending, sort_column_name_intern)),
            index_limit,
            is_one,
        ),
        get(
            table_name_intern,
            table_name_extern,
            response_getter_columns,
            index_filter,
            Some((Sort::Descending, sort_column_name_intern)),
            index_limit,
            is_one,
        ),
    )
}
fn create_one<'columns, I>(table_name_intern: &str, columns: I) -> String
where
    I: IntoIterator<Item = &'columns stage3::RequestColumnOne<'columns>>,
    I::IntoIter: Iterator<Item = I::Item> + Clone,
{
    let request_columns = columns
        .into_iter()
        .filter_map(|column| column.request_setter_column());
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
    I: IntoIterator<Item = &'columns stage3::RequestColumnOne<'columns>>,
{
    let request_columns = columns
        .into_iter()
        .filter_map(|column| column.request_setter_column());
    fmt2::fmt! { { str } =>
        "UPDATE " {table_name_intern} " SET "
        @..join(request_columns => "," => |column| {column.0} "=" {column.1})
        " WHERE " {id_name} "=?"
    }
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
            stage3::ResponseColumnGetterRef::One(stage3::ResponseColumnGetterOne::Element(
                element,
            )) => {
                response_getter_column_elements.push(ResponseColumnGetterElement {
                    element,
                    parent_optional,
                });
            }
            stage3::ResponseColumnGetterRef::One(stage3::ResponseColumnGetterOne::Compound(
                compound,
            )) => {
                let parent_optional = parent_optional || compound.optional;
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
            stage3::ResponseColumnGetterRef::Compounds(_) => {}
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

fn serde_skip() -> proc_macro2::TokenStream {
    quote! {
        #[serde(skip)]
    }
}
fn serde_name(serde_name: &str) -> proc_macro2::TokenStream {
    quote! {
        #[serde(rename = #serde_name)]
    }
}

fn response_getter_column(
    name_extern: &Ident,
    optional: bool,
    parent_optional: bool,
) -> proc_macro2::TokenStream {
    let field_access = quote! {
        response.#name_extern
    };
    if optional {
        quote! {
            if let ::core::option::Option::Some(v) = #field_access {
                ::core::option::Option::Some(::laraxum::model::Decode::decode(v))
            } else {
                ::core::option::Option::None
            }
        }
    } else if parent_optional {
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
    parent_optional: bool,
) -> proc_macro2::TokenStream {
    match column {
        stage3::ResponseColumnGetterRef::One(stage3::ResponseColumnGetterOne::Element(element)) => {
            let stage3::ResponseColumnGetterElement {
                name_extern,
                optional,
                ..
            } = element;
            let optional = *optional;
            let name_extern = from_str_to_rs_ident(name_extern);
            response_getter_column(&name_extern, optional, parent_optional)
        }
        stage3::ResponseColumnGetterRef::One(stage3::ResponseColumnGetterOne::Compound(
            compound,
        )) => {
            let stage3::ResponseColumnGetterCompound {
                optional,
                foreign_table_rs_name: rs_ty_name,
                columns,
                ..
            } = compound;
            let optional = *optional;
            let parent_optional = parent_optional || optional;

            let getter = response_getter_compound(
                rs_ty_name,
                columns.iter().map(stage3::ResponseColumnGetterRef::from),
                parent_optional,
            );
            if optional {
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
        stage3::ResponseColumnGetterRef::Compounds(compounds) => {
            let stage3::ResponseColumnGetterCompounds {
                rs_name: _,
                index_rs_name: table_rs_name,
                table_id_name_extern,
                many_foreign_table_rs_name,
            } = compounds;
            let one_id = {
                let table_id_name_extern = from_str_to_rs_ident(table_id_name_extern);
                response_getter_column(&table_id_name_extern, false, parent_optional)
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
    one: bool,
) -> proc_macro2::TokenStream {
    if one {
        transform_response_one(response, response_getter)
    } else {
        transform_response_many(response, response_getter)
    }
}

impl stage3::Validate {
    fn to_token_stream(
        &self,
        value: &proc_macro2::TokenStream,
    ) -> [Option<proc_macro2::TokenStream>; 4] {
        [
            self.max_len.map(|max_len| {
                let err_message = format!("max length is {max_len}");
                quote! {
                    if #value.len() <= #max_len {
                        ::core::result::Result::Ok(())
                    } else { ::core::result::Result::Err(#err_message) }
                }
            }),
            self.min_len.map(|min_len| {
                let err_message = format!("min length is {min_len}");
                quote! {
                    if #value.len() >= #min_len {
                        ::core::result::Result::Ok(())
                    } else { ::core::result::Result::Err(#err_message) }
                }
            }),
            self.func.as_ref().map(|func| {
                quote! {
                    (#func)(#value)
                }
            }),
            self.matches.as_ref().and_then(|matches| {
                let end = matches.end.as_deref().map(|end| match matches.limits {
                    syn::RangeLimits::Closed(_) => {
                        let lte = end.to_token_stream();
                        let err_message = format!("must less than or equal to {lte}");
                        quote! {
                            if #value <= &#lte {
                                ::core::result::Result::Ok(())
                            } else { ::core::result::Result::Err(#err_message) }
                        }
                    }
                    syn::RangeLimits::HalfOpen(_) => {
                        let lt = end.to_token_stream();
                        let err_message = format!("must be less than {lt}");
                        quote! {
                            if #value < &#lt {
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
                        if #value >= &#gte {
                            #end
                        } else { ::core::result::Result::Err(#err_message) }
                    }
                });
                start.or(end)
            }),
        ]
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
                // #( #index_variant_deserialize_token_streams )*
                // ::core::result::Result::Ok(#table_index_rs_name::#table_index_rs_name)
            }
        }
    }
}

// fn request_setter_column(rs_name: &Ident, optional: bool) -> proc_macro2::TokenStream {
//     if optional {
//         quote! {
//             ::core::option::Option::map(request.#rs_name, ::laraxum::Encode::encode)
//         }
//     } else {
//         quote! {
//             ::laraxum::Encode::encode(request.#rs_name)
//         }
//     }
// }

struct Table {
    token_stream: proc_macro2::TokenStream,
    migration_up: String,
    migration_down: String,
}
impl From<stage3::Table<'_>> for Table {
    #[allow(clippy::too_many_lines)]
    fn from(table: stage3::Table) -> Self {
        let response_column_fields = table.columns.iter().map(|column| {
            let stage3::ResponseColumnField {
                rs_name,
                rs_ty,
                attr,
                rs_attrs,
            } = column.response_field();

            let serde_skip = attr.skip.then(serde_skip);
            let serde_name = attr.name.as_deref().map(serde_name);

            quote! {
                #(#rs_attrs)* #serde_skip #serde_name
                pub #rs_name: #rs_ty
            }
        });

        let request_column_fields = table
            .columns
            .iter()
            .filter_map(|column| column.request_field())
            .map(|column| {
                let stage3::RequestColumnField {
                    rs_name,
                    rs_ty,
                    attr,
                    rs_attrs,
                    ..
                } = column;

                let serde_name = attr.name.as_deref().map(serde_name);

                let token_stream = quote! {
                    #(#rs_attrs)* #serde_name
                    pub #rs_name:
                };
                let request_column_field = quote! {
                    #token_stream #rs_ty,
                };
                let request_patch_column_field = quote! {
                    #token_stream ::core::option::Option<#rs_ty>,
                };
                [request_column_field, request_patch_column_field]
            });
        let [request_column_fields, request_patch_column_fields] =
            crate::utils::syn::unzip_token_streams(request_column_fields);

        let create_table = create_table(&table.name_intern, table.columns.iter());
        let delete_table = delete_table(&table.name_intern);

        let table_rs_name = table.rs_name;
        let table_request_rs_name = &*table.request_rs_name;
        let table_request_patch_rs_name = &*table.request_patch_rs_name;
        let table_request_error_rs_name = &*table.request_error_rs_name;
        // let table_record_rs_name = quote::format_ident!("{table_rs_name}Record");
        let table_rs_attrs = table.rs_attrs;
        let db_rs_name = &table.db_rs_name;
        let doc = fmt2::fmt! { { str } => "`" {table.name_intern} "`"};
        let table_token_stream = quote! {
            #[doc = #doc]
            #[derive(::serde::Serialize)]
            #(#table_rs_attrs)*
            pub struct #table_rs_name {
                #(#response_column_fields),*
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

        let collection_model_token_stream = table.columns.is_collection().then(|| {
            let response_getter_columns =
                table.columns.iter().map(|column| column.response_getter());
            let response_getter =
                &response_getter_compound(table.rs_name, response_getter_columns.clone(), false);
            let response_getter = response_getter_fn(response_getter);
            let response_getter = &response_getter;

            let (response_getter_column_elements, response_getter_column_compounds) =
                flatten(response_getter_columns);
            let response_getter_columns = (
                &*response_getter_column_elements,
                &*response_getter_column_compounds,
            );

            let request_setters = table.columns.iter().filter_map(|column| match column {
                // TODO: use methods
                stage3::ColumnRef::One(stage3::ColumnOne {
                    request: stage3::RequestColumnOne::Some { setter, .. },
                    ..
                }) => Some(setter),
                _ => None,
            });
            let request_setter_columns = request_setters.clone().map(|column| {
                let stage3::RequestColumnSetterOne {
                    rs_name, optional, ..
                } = column;
                if *optional {
                    quote! {
                        ::core::option::Option::map(
                            request.#rs_name,
                            ::laraxum::model::Encode::encode
                        )
                    }
                } else {
                    quote! {
                        ::laraxum::model::Encode::encode(request.#rs_name)
                    }
                }
            });
            let request_setter_token_stream = quote! {
                #( #request_setter_columns, )*
            };

            let request_setter_compounds_columns =
                table.columns.iter().filter_map(|column| match column {
                    stage3::ColumnRef::Compounds(compounds) => Some(&compounds.request.setter),
                    stage3::ColumnRef::One(_) => None,
                });

            let request_setter_compounds_create_many =
                request_setter_compounds_columns.clone().map(|column| {
                    let stage3::RequestColumnSetterCompounds {
                        rs_name,
                        index_rs_name,
                        many_foreign_table_rs_name,
                    } = column;
                    quote! {{
                        <#many_foreign_table_rs_name as ::laraxum::ManyModel::<
                            #index_rs_name,
                        >>::create_many(
                            db,
                            id,
                            &request.#rs_name,
                        ).await?;
                    }}
                });
            let request_setter_compounds_create_many = quote! {
                #( #request_setter_compounds_create_many )*
            };

            let request_setter_compounds_update_many =
                request_setter_compounds_columns.clone().map(|column| {
                    let stage3::RequestColumnSetterCompounds {
                        rs_name,
                        index_rs_name,
                        many_foreign_table_rs_name,
                    } = column;
                    quote! {{
                        <#many_foreign_table_rs_name as ::laraxum::ManyModel::<
                            #index_rs_name,
                        >>::update_many(
                            db,
                            id,
                            &request.#rs_name,
                        ).await?;
                    }}
                });
            let request_setter_compounds_update_many = quote! {
                #( #request_setter_compounds_update_many )*
            };

            let request_setter_compounds_delete_many =
                request_setter_compounds_columns.clone().map(|column| {
                    let stage3::RequestColumnSetterCompounds {
                        rs_name: _,
                        index_rs_name,
                        many_foreign_table_rs_name,
                    } = column;
                    quote! {{
                        <#many_foreign_table_rs_name as ::laraxum::ManyModel::<
                            #index_rs_name,
                        >>::delete_many(
                            db,
                            id,
                        ).await?;
                    }}
                });
            let request_setter_compounds_delete_many = quote! {
                #( #request_setter_compounds_delete_many )*
            };

            let get_all = get_all(
                &table.name_intern,
                &table.name_extern,
                response_getter_columns,
            );
            let get_all_response = transform_response_many(
                &quote! {
                    ::sqlx::query!(#get_all)
                },
                response_getter,
            );
            let request_columns = table
                .columns
                .iter()
                .filter_map(|column| column.request_one());
            let create_one = create_one(&table.name_intern, request_columns.clone());

            let collection_token_stream = quote! {
                #[derive(::serde::Deserialize)]
                pub struct #table_request_rs_name {
                    #request_column_fields
                }

                impl ::laraxum::Collection for #table_rs_name {
                    type CreateRequest = #table_request_rs_name;
                    type CreateRequestError = #table_request_error_rs_name;

                    /// `get_all`
                    ///
                    /// ```sql
                    #[doc = #get_all]
                    /// ```
                    async fn get_all(db: &Self::Db)
                        -> ::core::result::Result<
                            ::std::vec::Vec<Self::Response>,
                            ::laraxum::Error,
                        >
                    {
                        #get_all_response
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
                        let response = ::sqlx::query!(#create_one, #request_setter_token_stream);
                        let response = response.execute(&db.pool).await?;
                        let id = response.last_insert_id();
                        #request_setter_compounds_create_many
                        transaction.commit().await?;
                        ::core::result::Result::Ok(())
                    }
                }
            };

            let validate_var_rs_name = quote! { value };
            let request_column_validate = request_setters.filter_map(|column| {
                let validate = column.validate.to_token_stream(&validate_var_rs_name);
                let validate = validate.into_iter().flatten();
                let not_empty = validate.clone().count() != 0;
                not_empty.then_some((column, validate))
            });
            let request_validate = request_column_validate.map(|(column, validate)| {
                let rs_name = column.rs_name;
                let error_field = quote! {
                    #[serde(skip_serializing_if = "<[&str]>::is_empty")]
                    pub #rs_name: ::std::vec::Vec::<&'static str>,
                };
                let value = quote! {
                    &self.#rs_name
                };
                let validate_results = validate.map(|validate_result| {
                    quote! {
                        if let ::core::result::Result::Err(err) = #validate_result {
                            ::laraxum::request::error_builder::<
                                (),
                                #table_request_error_rs_name,
                            >(
                                &mut e,
                                |e| e.#rs_name.push(err),
                            );
                        }
                    }
                });
                let validate_results = quote! { #( #validate_results )* };
                let validate_result_update_create = if column.optional {
                    quote! {
                        if let ::core::option::Option::Some(#validate_var_rs_name) = #value {
                            #validate_results
                        }
                    }
                } else {
                    quote! {
                        let #validate_var_rs_name = #value;
                        #validate_results
                    }
                };
                let validate_result_patch = if column.optional {
                    quote! {
                        if let ::core::option::Option::Some(
                            ::core::option::Option::Some(#validate_var_rs_name)
                        ) = #value {
                            #validate_results
                        }
                    }
                } else {
                    quote! {
                        if let ::core::option::Option::Some(#validate_var_rs_name) = #value {
                            #validate_results
                        }
                    }
                };
                [
                    error_field,
                    validate_result_update_create,
                    validate_result_patch,
                ]
            });
            let [
                request_error_fields,
                request_validate_results_update_create,
                request_validate_results_patch,
            ] = crate::utils::syn::unzip_token_streams(request_validate);

            let collection_token_stream = quote! {
                #collection_token_stream

                #[derive(Default, ::serde::Serialize)]
                pub struct #table_request_error_rs_name {
                    #request_error_fields
                }
                impl ::core::convert::From<#table_request_error_rs_name>
                    for ::laraxum::ModelError<#table_request_error_rs_name>
                {
                    fn from(value: #table_request_error_rs_name) -> Self {
                        Self::UnprocessableEntity(value)
                    }
                }

                impl ::laraxum::Request::<::laraxum::request::method::Create>
                    for #table_request_rs_name
                {
                    type Error = #table_request_error_rs_name;
                    fn validate(&self) -> ::core::result::Result::<(), Self::Error> {
                        let mut e = ::core::result::Result::Ok(());
                        #request_validate_results_update_create
                        e
                    }
                }
                impl ::laraxum::Request::<::laraxum::request::method::Update>
                    for #table_request_rs_name
                {
                    type Error = #table_request_error_rs_name;
                    fn validate(&self) -> ::core::result::Result::<(), Self::Error> {
                        <
                            Self as ::laraxum::Request::<::laraxum::request::method::Create>
                        >::validate(self)
                    }
                }
            };

            let indexes = table
                .columns
                .iter()
                .filter_map(|column| match column {
                    stage3::ColumnRef::One(one) => Some(one),
                    stage3::ColumnRef::Compounds(_) => None,
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
                    let is_unique = column.create.ty.unique();

                    let table_name_intern = &*table.name_intern;
                    let table_name_extern = &*table.name_extern;
                    let column_response_name = column.response.field.rs_name.to_string();
                    column.index.iter().map(move |index| {
                        let is_one = is_unique && index.filter.is_eq();
                        let index_rs_name = &index.rs_name;

                        let filter = index.filter.parameter().map(|parameter_name| {
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
                            index
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
                        let sort = index.sort.then(|| -> (Ident, Ident, Type) {
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

                        let index_struct_token_stream = quote! {
                            #[derive(::serde::Deserialize)]
                            pub struct #index_rs_name<#lifetime> {
                                #filter_field
                                #limit_field
                                #sort_field
                            }
                        };
                        let filter_parameter = filter.as_ref().map(|(short_name, _, _, _)| {
                            quote! { request.#short_name, }
                        });
                        let limit_parameter = limit.as_ref().map(|(name, _)| match index.limit {
                            stage3::ColumnAttrIndexLimit::Page { per_page } => {
                                quote! { request.#name * #per_page, }
                            }
                            _ => {
                                quote! { request.#name, }
                            }
                        });

                        let response = if index.sort {
                            let (get_sort_asc, get_sort_desc) = get_sort_asc_desc(
                                table_name_intern,
                                table_name_extern,
                                response_getter_columns,
                                Some((index.filter, name_intern)),
                                name_intern,
                                Some(index.limit),
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
                                response_getter_columns,
                                Some((index.filter, name_intern)),
                                None,
                                Some(index.limit),
                                is_one,
                            );
                            let response = quote! {
                                ::sqlx::query!(#get, #filter_parameter #limit_parameter)
                            };
                            transform_response(&response, response_getter, is_one)
                        };

                        let index_impl_token_stream = if is_one {
                            quote! {
                                impl
                                    ::laraxum::CollectionIndexOne<#index_rs_name<#auto_lifetime>>
                                    for #table_rs_name
                                {
                                    type OneRequest<'b> = #index_rs_name<#lifetime>;
                                    type OneResponse = Self;
                                    async fn get_index_one<'a>(
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
                                    ::laraxum::CollectionIndexMany<#index_rs_name<#auto_lifetime>>
                                    for #table_rs_name
                                {
                                    type OneRequest<'b> = #index_rs_name<#lifetime>;
                                    type ManyResponse = Self;
                                    async fn get_index_many<'a>(
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
                        let index_struct_token_stream = quote! {
                            #index_struct_token_stream
                            #index_impl_token_stream
                        };

                        // only include variant in enum if:
                        // - there is a controller index query
                        // - this variant is in the controller query index
                        let index_enum = table.index_rs_name.filter(|_| index.controller);
                        let index_enum = index_enum.map(|table_index_rs_name| {
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

                            let index_variant_def_token_stream = quote! {
                                #index_rs_name {
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

                            let index_get = quote! {
                                #table_index_rs_name::#index_rs_name {
                                    #filter_get
                                    #limit_get
                                    #sort_get
                                }
                            };
                            let index_set = quote! {
                                #index_rs_name {
                                    #filter_set
                                    #limit_set
                                    #sort_set
                                }
                            };
                            let index_variant_match_token_stream = if is_one {
                                quote! {
                                    #index_get => {
                                        <#table_rs_name as
                                            ::laraxum::CollectionIndexOne<
                                                #index_rs_name<#auto_lifetime>
                                            >
                                        >::get_index_one_vec(db, #index_set).await
                                    }
                                }
                            } else {
                                quote! {
                                    #index_get => {
                                        <#table_rs_name as
                                            ::laraxum::CollectionIndexMany<
                                                #index_rs_name<#auto_lifetime>
                                            >
                                        >::get_index_many(db, #index_set).await
                                    }
                                }
                            };
                            let index_variants = [
                                filter.map(|(_, name, _, rs_ty_owned)| (name, rs_ty_owned.clone())),
                                limit.map(|(name, rs_ty)| (name, rs_ty)),
                                sort.map(|(_, name, rs_ty)| (name, rs_ty)),
                            ];
                            let index_variant_type_signature = (index_rs_name, index_variants);
                            (
                                index_variant_def_token_stream,
                                index_variant_match_token_stream,
                                index_variant_type_signature,
                            )
                        });

                        (index_struct_token_stream, index_enum)
                    })
                });

            let collection_token_stream = if let Some(table_index_rs_name) = table.index_rs_name {
                let indexes = indexes.collect::<Vec<_>>();
                let index_token_streams = indexes.iter().map(|(i, _)| i);
                let index_variants = indexes
                    .iter()
                    .filter_map(|(_, index_variant)| index_variant.as_ref());
                let index_variant_def_token_streams = index_variants.clone().map(|(i, _, _)| i);
                let index_variant_match_token_streams = index_variants.clone().map(|(_, i, _)| i);
                let index_variant_type_signatures =
                    index_variants.map(|(_, _, (index_rs_name, fields))| {
                        (
                            *index_rs_name,
                            fields.iter().flat_map(|field| {
                                field.as_ref().map(|(rs_name, rs_ty)| (rs_name, rs_ty))
                            }),
                        )
                    });

                let impl_deserialize_for_table_index = impl_deserialize_for_untagged_enum(
                    table_index_rs_name,
                    index_variant_type_signatures,
                    Some(table_index_rs_name),
                );

                quote! {
                    #collection_token_stream
                    #( #index_token_streams )*

                    pub enum #table_index_rs_name {
                        #( #index_variant_def_token_streams, )*
                        #table_index_rs_name,
                    }
                    #impl_deserialize_for_table_index
                    impl ::laraxum::CollectionIndexMany<#table_index_rs_name> for #table_rs_name {
                        type OneRequest<'b> = #table_index_rs_name;
                        type ManyResponse = Self;
                        async fn get_index_many<'a>(
                            db: &Self::Db,
                            request: Self::OneRequest<'a>,
                        )
                            -> ::core::result::Result<
                                ::std::vec::Vec<Self::ManyResponse>,
                                ::laraxum::Error,
                            >
                        {
                            match request {
                                #( #index_variant_match_token_streams, )*
                                #table_index_rs_name::#table_index_rs_name => {
                                    <#table_rs_name as ::laraxum::Collection>::get_all(db).await
                                }
                            }
                        }
                    }
                }
            } else {
                let index_token_streams = indexes.map(|(i, _)| i);

                quote! {
                    #collection_token_stream
                    #( #index_token_streams )*
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
                response_getter_columns,
                (stage3::ColumnAttrIndexFilter::Eq, table_id_name_intern),
            );
            let get_one_response = transform_response_one(
                &quote! {
                    ::sqlx::query!(#get_one, id)
                },
                response_getter,
            );
            let update_one = update_one(&table.name_intern, table_id_name, request_columns);
            let delete_one = delete_one(&table.name_intern, table_id_name);

            quote! {
                #collection_token_stream

                #[derive(::serde::Deserialize)]
                pub struct #table_request_patch_rs_name {
                    #request_patch_column_fields
                }

                impl ::laraxum::Request::<::laraxum::request::method::Patch>
                    for #table_request_patch_rs_name
                {
                    type Error = #table_request_error_rs_name;
                    fn validate(&self) -> ::core::result::Result::<(), Self::Error> {
                        let mut e = ::core::result::Result::Ok(());
                        #request_validate_results_patch
                        e
                    }
                }

                impl ::laraxum::Model for #table_rs_name {
                    type Id = #table_id_rs_ty;
                    type UpdateRequest = #table_request_rs_name;
                    type UpdateRequestError = #table_request_error_rs_name;

                    /// `get_one`
                    ///
                    /// ```sql
                    #[doc = #get_one]
                    /// ```
                    async fn get_one(
                        db: &Self::Db,
                        id: Self::Id,
                    )
                        -> ::core::result::Result<
                            Self::Response,
                            ::laraxum::Error,
                        >
                    {
                        #get_one_response
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
                        let response = ::sqlx::query!(#create_one, #request_setter_token_stream);
                        let response = response.execute(&db.pool).await?;
                        let id = response.last_insert_id();
                        #request_setter_compounds_create_many
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
                        let response =
                            ::sqlx::query!(#update_one, #request_setter_token_stream id);
                        response.execute(&db.pool).await?;
                        #request_setter_compounds_update_many
                        transaction.commit().await?;
                        ::core::result::Result::Ok(())
                    }
                    async fn update_get_one(
                        db: &Self::Db,
                        request: Self::UpdateRequest,
                        id: Self::Id,
                    )
                        -> ::core::result::Result<
                            Self::Response,
                            ::laraxum::ModelError<Self::UpdateRequestError>,
                        >
                    {
                        Self::update_one(db, request, id).await?;
                        let response = Self::get_one(db, id).await?;
                        ::core::result::Result::Ok(response)
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
                        #request_setter_compounds_delete_many
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

            let get_many_request_query = table.index_rs_name.map_or_else(
                || quote! { () },
                |index_rs_name| index_rs_name.to_token_stream(),
            );

            let get_many = table.index_rs_name.map(|index_rs_name| {
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
                            #table_rs_name as ::laraxum::CollectionIndexMany<#index_rs_name>
                        >::get_index_many(&*state, query).await?;
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
                one: &stage3::ColumnOne,
                many: &stage3::ColumnOne,
            ) -> proc_macro2::TokenStream {
                let index_rs_ty = many.struct_name.map_or_else(
                    || one.response.field.rs_ty.to_token_stream(),
                    |struct_name| struct_name.to_token_stream(),
                );
                let one_request_rs_ty = one.request.request_field().map(|field| &*field.rs_ty);
                let many_request_rs_ty = many.request.request_field().map(|field| &*field.rs_ty);
                let many_response_rs_ty = many.response.field.rs_ty;

                let many_response_getter =
                    stage3::ResponseColumnGetterRef::One(&many.response.getter);

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
                    (stage3::ColumnAttrIndexFilter::Eq, one.name_intern()),
                );
                let get_many_response = transform_response_many(
                    &quote! {
                        ::sqlx::query!(#get_many, one)
                    },
                    &response_getter,
                );

                let request_columns = [&one.request, &many.request];
                let create_one = create_one(&table.name_intern, request_columns);
                let delete_many = delete_one(&table.name_intern, one.name());

                let table_rs_name = table.rs_name;

                quote! {
                    impl ::laraxum::ManyModel<#index_rs_ty> for #table_rs_name {
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
                                Self as ::laraxum::ManyModel<#index_rs_ty>
                            >::delete_many(db, one).await?;
                            <
                                Self as ::laraxum::ManyModel<#index_rs_ty>
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
