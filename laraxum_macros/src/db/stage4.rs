use super::stage3;

use crate::utils::syn::from_str_to_rs_ident;

use std::{borrow::Cow, vec};

use quote::{ToTokens, quote};
use syn::Ident;

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
    pub const fn ty(&self) -> &'static str {
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
    pub const fn current_time_func(&self) -> &'static str {
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
            Self::u8 => Cow::Borrowed("TINYINT UNSIGNED"),
            Self::i8 => Cow::Borrowed("TINYINT"),
            Self::u16 => Cow::Borrowed("SMALLINT UNSIGNED"),
            Self::i16 => Cow::Borrowed("SMALLINT"),
            Self::u32 => Cow::Borrowed("INT UNSIGNED"),
            Self::i32 => Cow::Borrowed("INT"),
            Self::u64 => Cow::Borrowed("BIGINT UNSIGNED"),
            Self::i64 => Cow::Borrowed("BIGINT"),
            Self::f32 => Cow::Borrowed("FLOAT"),
            Self::f64 => Cow::Borrowed("DOUBLE"),

            Self::String(string) => string.ty(),
            Self::Time(time) => Cow::Borrowed(time.ty()),
        }

        #[cfg(feature = "sqlite")]
        match self {
            Self::bool => Cow::Borrowed("BOOLEAN"),
            Self::u8 => Cow::Borrowed("INTEGER"),
            Self::i8 => Cow::Borrowed("INTEGER"),
            Self::u16 => Cow::Borrowed("INTEGER"),
            Self::i16 => Cow::Borrowed("INTEGER"),
            Self::u32 => Cow::Borrowed("INTEGER"),
            Self::i32 => Cow::Borrowed("INTEGER"),
            Self::u64 => Cow::Borrowed("INTEGER"),
            Self::i64 => Cow::Borrowed("BIGINT"),
            Self::f32 => Cow::Borrowed("FLOAT"),
            Self::f64 => Cow::Borrowed("DOUBLE"),

            Self::String(string) => string.sql_ty(),
            Self::Time(time) => Cow::Borrowed(time.sql_ty()),
        }

        #[cfg(feature = "postgres")]
        match self {
            Self::bool => Cow::Borrowed("BOOL"),
            Self::u8 => Cow::Borrowed("CHAR"),
            Self::i8 => Cow::Borrowed("CHAR"),
            Self::u16 => Cow::Borrowed("INT2"),
            Self::i16 => Cow::Borrowed("INT2"),
            Self::u32 => Cow::Borrowed("INT4"),
            Self::i32 => Cow::Borrowed("INT4"),
            Self::u64 => Cow::Borrowed("INT8"),
            Self::i64 => Cow::Borrowed("INT8"),
            Self::f32 => Cow::Borrowed("FLOAT4"),
            Self::f64 => Cow::Borrowed("FLOAT8"),

            Self::String(string) => string.sql_ty(),
            Self::Time(time) => Cow::Borrowed(time.sql_ty()),
        }
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
            Self::Id => Cow::Borrowed({
                #[cfg(feature = "mysql")]
                {
                    "BIGINT UNSIGNED"
                }
                #[cfg(feature = "sqlite")]
                {
                    "INTEGER"
                }
                #[cfg(feature = "postgres")]
                {
                    "BIGSERIAL"
                }
            }),
            Self::Value(value) => value.ty.ty(),
            Self::AutoTime(auto_time) => Cow::Borrowed(auto_time.ty.ty()),
        }
    }
}

impl stage3::TyCompound<'_> {
    #[expect(clippy::unused_self)]
    const fn ty(&self) -> &'static str {
        {
            #[cfg(feature = "mysql")]
            {
                "BIGINT UNSIGNED"
            }
            #[cfg(feature = "sqlite")]
            {
                "INTEGER"
            }
            #[cfg(feature = "postgres")]
            {
                "BIGINT"
            }
        }
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
            Self::Element(stage3::TyElement::Id) => {
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
fn get_all(
    table_name_intern: &str,
    table_name_extern: &str,
    response_getter_column_elements: &[ResponseColumnGetterElement],
    response_getter_column_compounds: &[&stage3::ResponseColumnGetterCompound],
) -> String {
    fmt2::fmt! { { str } =>
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
    }
}
fn get_filter(
    table_name_intern: &str,
    table_name_extern: &str,
    id_name_intern: &str,
    response_getter_column_elements: &[ResponseColumnGetterElement],
    response_getter_column_compounds: &[&stage3::ResponseColumnGetterCompound],
) -> String {
    let mut get_all = get_all(
        table_name_intern,
        table_name_extern,
        response_getter_column_elements,
        response_getter_column_compounds,
    );
    fmt2::fmt! { (get_all) => " WHERE " {id_name_intern} "=?" };
    get_all
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
                ::core::option::Option::Some(::laraxum::backend::Decode::decode(v))
            } else {
                ::core::option::Option::None
            }
        }
    } else if parent_optional {
        quote! {
            if let ::core::option::Option::Some(v) = #field_access {
                ::laraxum::backend::Decode::decode(v)
            } else {
                return ::core::result::Result::Ok(::core::option::Option::None);
            }
        }
    } else {
        quote! {
            ::laraxum::backend::Decode::decode(#field_access)
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
                table_rs_name,
                table_id_name_extern,
                foreign_table_rs_name: _,
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
                // <#many_foreign_table_rs_name as ::laraxum::ManyModel::<
                //     #table_rs_name,
                // >>::get_many(
                //     db,
                //     #one_id,
                // ).await?
                // match {
                //     <#many_foreign_table_rs_name as ::laraxum::ManyModel::<
                //         #table_rs_name,
                //     >>::get_many(
                //         db,
                //         #one_id,
                //     ).await
                // } {
                //     ::core::result::Result::Ok(response) => response,
                //     ::core::result::Result::Err(_) => {
                //         return ::core::result::Result::Err(::sqlx::Error::RowNotFound);
                //     }
                // }
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
impl super::stage2::ValidateRule {
    fn to_token_stream(&self, value: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        match *self {
            Self::MaxLen(max_len) => {
                let err_message = format!("max length is {max_len}");
                quote! {
                    if #value.len() <= #max_len {
                        ::core::result::Result::Ok(())
                    } else { ::core::result::Result::Err(#err_message) }
                }
            }
            Self::MinLen(ref min_len) => {
                let min_len = min_len.to_token_stream();
                let err_message = format!("min length is {min_len}");
                quote! {
                    if #value.len() >= #min_len {
                        ::core::result::Result::Ok(())
                    } else { ::core::result::Result::Err(#err_message) }
                }
            }
            Self::Func(ref f) => {
                quote! {
                    (#f)(#value)
                }
            }
            Self::Matches(ref matches) => {
                let matches = matches.to_token_stream();
                let err_message = format!("must match pattern {matches}");
                quote! {
                    match #value {
                        #matches => ::core::result::Result::Ok(()),
                        _ => ::core::result::Result::Err(#err_message),
                    }
                }
            }
            Self::NMatches(ref n_matches) => {
                let n_matches = n_matches.to_token_stream();
                let err_message = format!("must not match pattern {n_matches}");
                quote! {
                    match #value {
                        #n_matches => ::core::result::Result::Err(#err_message),
                        _ => ::core::result::Result::Ok(()),
                    }
                }
            }
            Self::Eq(ref eq) => {
                let eq = eq.to_token_stream();
                let err_message = format!("must be equal to {eq}");
                quote! {
                    if #value == &#eq {
                        ::core::result::Result::Ok(())
                    } else { ::core::result::Result::Err(#err_message) }
                }
            }
            Self::NEq(ref n_eq) => {
                let n_eq = n_eq.to_token_stream();
                let err_message = format!("must not be equal to {n_eq}");
                quote! {
                    if #value != &#n_eq {
                        ::core::result::Result::Ok(())
                    } else { ::core::result::Result::Err(#err_message) }
                }
            }
            Self::Gt(ref gt) => {
                let gt = gt.to_token_stream();
                let err_message = format!("must be greater than {gt}");
                quote! {
                    if #value > &#gt {
                        ::core::result::Result::Ok(())
                    } else { ::core::result::Result::Err(#err_message) }
                }
            }
            Self::Lt(ref lt) => {
                let lt = lt.to_token_stream();
                let err_message = format!("must be less than {lt}");
                quote! {
                    if #value < &#lt {
                        ::core::result::Result::Ok(())
                    } else { ::core::result::Result::Err(#err_message) }
                }
            }
            Self::Gte(ref gte) => {
                let gte = gte.to_token_stream();
                let err_message = format!("must be greater than or equal to {gte}");
                quote! {
                    if #value >= &#gte {
                        ::core::result::Result::Ok(())
                    } else { ::core::result::Result::Err(#err_message) }
                }
            }
            Self::Lte(ref lte) => {
                let lte = lte.to_token_stream();
                let err_message = format!("must less than or equal to {lte}");
                quote! {
                    if #value <= &#lte {
                        ::core::result::Result::Ok(())
                    } else { ::core::result::Result::Err(#err_message) }
                }
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

                quote! {
                    #(#rs_attrs)* #serde_name
                    pub #rs_name: #rs_ty
                }
            });

        let create_table = create_table(&table.name_intern, table.columns.iter());
        let delete_table = delete_table(&table.name_intern);

        let table_rs_name = &table.rs_name;
        let table_request_rs_name = &table.request_rs_name;
        let table_request_error_rs_name = &table.request_error_rs_name;
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

            impl ::laraxum::backend::Decode for #table_rs_name {
                type Decode = Self;
                #[inline]
                fn decode(decode: Self::Decode) -> Self {
                    decode
                }
            }

            impl ::laraxum::backend::Encode for #table_rs_name {
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

            let (response_getter_column_elements, response_getter_column_compounds) =
                flatten(response_getter_columns);

            let request_setters = table.columns.iter().filter_map(|column| match column {
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
                            ::laraxum::backend::Encode::encode
                        )
                    }
                } else {
                    quote! {
                        ::laraxum::backend::Encode::encode(request.#rs_name)
                    }
                }
            });
            let request_setter_token_stream = quote! {
                #(#request_setter_columns,)*
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
                        table_rs_name,
                        foreign_table_rs_name: _,
                        many_foreign_table_rs_name,
                    } = column;
                    quote! {{
                        <#many_foreign_table_rs_name as ::laraxum::ManyModel::<
                            #table_rs_name,
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
                        table_rs_name,
                        foreign_table_rs_name: _,
                        many_foreign_table_rs_name,
                    } = column;
                    quote! {{
                        <#many_foreign_table_rs_name as ::laraxum::ManyModel::<
                            #table_rs_name,
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
                        table_rs_name,
                        foreign_table_rs_name: _,
                        many_foreign_table_rs_name,
                    } = column;
                    quote! {{
                        <#many_foreign_table_rs_name as ::laraxum::ManyModel::<
                            #table_rs_name,
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
                &response_getter_column_elements,
                &response_getter_column_compounds,
            );
            let request_columns = table
                .columns
                .iter()
                .filter_map(|column| column.request_one());
            let create_one = create_one(&table.name_intern, request_columns.clone());

            let collection_token_stream = quote! {
                #[derive(::serde::Deserialize)]
                pub struct #table_request_rs_name {
                    #(#request_column_fields),*
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
                        let response = ::sqlx::query!(#get_all);
                        let response = response.fetch(&db.pool);
                        let response = ::futures::StreamExt::then(response, #response_getter);
                        let response: ::std::vec::Vec<_> =
                            ::futures::TryStreamExt::try_collect(response).await?;
                        ::core::result::Result::Ok(response)
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
                        let response = ::sqlx::query!(#create_one, #request_setter_token_stream);
                        let response = response.execute(&db.pool).await?;
                        let id = response.last_insert_id();
                        #request_setter_compounds_create_many
                        ::core::result::Result::Ok(())
                    }
                }

            };

            let request_column_validate =
                request_setters.filter(|column| !column.validate.is_empty());
            // let request_error_token_stream = if request_column_validate.clone().next().is_some() {
            let request_error_columns = request_column_validate.clone().map(
                |stage3::RequestColumnSetterOne { rs_name, .. }| {
                    quote! {
                        // #[serde(skip_serializing_if = "(|v| v.is_empty())")]
                        #[serde(skip_serializing_if = "<[&str]>::is_empty")]
                        pub #rs_name: ::std::vec::Vec::<&'static str>
                        // #[serde(skip_serializing_if = "::core::option::Option::is_none")]
                        // pub #rs_name: ::core::option::Option::<::std::vec::Vec::<&'static str>>
                    }
                },
            );
            let request_column_validate_rules = request_column_validate
                .flat_map(|column| {
                    column
                        .validate
                        .iter()
                        .map(|validate_rule| (validate_rule, column.optional, column.rs_name))
                })
                .map(|(validate_rule, optional, rs_name)| {
                    let var = quote! { value };
                    let value = quote! {
                        &self.#rs_name
                    };
                    let result = validate_rule.to_token_stream(&var);
                    let validate = quote! {
                        if let ::core::result::Result::Err(err) = #result {
                            ::laraxum::request::error_builder::<(), #table_request_error_rs_name>(
                                &mut e,
                                |e| e.#rs_name.push(err),
                            );
                        }
                    };
                    if optional {
                        quote! {
                            if let ::core::option::Option::Some(#var) = #value {
                                #validate
                            }
                        }
                    } else {
                        quote! {
                            let #var = #value;
                            #validate
                        }
                    }
                });

            let request_error_token_stream = quote! {
                #[derive(Default, ::serde::Serialize)]
                pub struct #table_request_error_rs_name {
                    #( #request_error_columns ),*
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
                        #( #request_column_validate_rules )*
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

            //             let request_error_token_stream = quote! {
            //                 #[derive(Default, ::serde::Serialize)]
            //                 pub struct #table_request_error_rs_name<'error> {
            //                     #( #request_error_columns ),*
            //                 }
            //                 impl ::core::convert::From<#table_request_error_rs_name<'_>>
            //                     for ::laraxum::ModelError<#table_request_error_rs_name<'_>>
            //                 {
            //                     fn from(value: #table_request_error_rs_name<'_>) -> Self {
            //                         Self::UnprocessableEntity(value)
            //                     }
            //                 }
            //
            //                 impl ::laraxum::Request::<::laraxum::request::method::Create>
            //                     for #table_request_rs_name
            //                 {
            //                     type Error = #table_request_error_rs_name<'_>;
            //                     fn validate(&self) -> ::core::result::Result::<(), Self::Error> {
            //                         let mut e = ::core::result::Result::Ok(());
            //                         #( #request_column_validate_rules )*
            //                         e
            //                     }
            //                 }
            //                 impl ::laraxum::Request::<::laraxum::request::method::Update>
            //                     for #table_request_rs_name
            //                 {
            //                     type Error = #table_request_error_rs_name<'_>;
            //                     fn validate(&self) -> ::core::result::Result::<(), Self::Error> {
            //                         <
            //                             Self as ::laraxum::Request::<::laraxum::request::method::Create>
            //                         >::validate(self)
            //                     }
            //                 }
            //             };

            //             } else {
            //                 quote! {
            //                     pub type #table_request_error_rs_name = ();
            //                     // pub type #table_request_error_rs_name = ::core::convert::Infallible;
            //                     impl ::laraxum::Request::<::laraxum::request::method::Create>
            //                         for #table_request_rs_name
            //                     {
            //                         type Error = #table_request_error_rs_name;
            //                         fn validate(&self) -> ::core::result::Result::<(), Self::Error> {
            //                             ::core::result::Result::Ok(())
            //                         }
            //                     }
            //                     impl ::laraxum::Request::<::laraxum::request::method::Update>
            //                         for #table_request_rs_name
            //                     {
            //                         type Error = #table_request_error_rs_name;
            //                         fn validate(&self) -> ::core::result::Result::<(), Self::Error> {
            //                             ::core::result::Result::Ok(())
            //                         }
            //                     }
            //
            //                 }
            //             };

            let indexes = table
                .columns
                .iter()
                .filter_map(|column| match column {
                    stage3::ColumnRef::One(one) => Some(one),
                    stage3::ColumnRef::Compounds(_) => None,
                })
                .filter_map(|column| {
                    let index = column.index?;
                    let index_name = &index.name;
                    let rs_ty = index
                        .request_ty
                        .as_deref()
                        .unwrap_or(column.response.field.rs_ty);
                    let rs_ty = if index.request_ty_ref {
                        quote! {
                            &'a #rs_ty
                        }
                    } else {
                        rs_ty.to_token_stream()
                    };
                    let name_intern = column.name_intern();
                    let unique = column.create.ty.unique();

                    let get_filter = get_filter(
                        &table.name_intern,
                        &table.name_extern,
                        name_intern,
                        &response_getter_column_elements,
                        &response_getter_column_compounds,
                    );

                    let index_token_stream = if unique {
                        quote! {
                            impl ::laraxum::CollectionIndexOne<#index_name> for #table_rs_name {
                                type OneRequest<'a> = #rs_ty;
                                type OneResponse = Self;
                                async fn get_index_one<'a>(
                                    db: &Self::Db,
                                    one: Self::OneRequest<'a>,
                                )
                                    -> ::core::result::Result<
                                        Self::OneResponse,
                                        ::laraxum::Error,
                                    >
                                {
                                    let response = ::sqlx::query!(#get_filter, one);
                                    let response = response.fetch(&db.pool);
                                    let mut response =
                                        ::futures::StreamExt::then(response, #response_getter);
                                    let mut response = ::core::pin::pin!(response);
                                    let response =
                                        ::futures::TryStreamExt::try_next(&mut response).await?;
                                    ::core::option::Option::ok_or(
                                        response,
                                        ::laraxum::Error::NotFound,
                                    )
                                }
                            }
                        }
                    } else {
                        quote! {
                            impl ::laraxum::CollectionIndexMany<#index_name> for #table_rs_name {
                                type OneRequest<'a> = #rs_ty;
                                type ManyResponse = Self;
                                async fn get_index_many<'a>(
                                    db: &Self::Db,
                                    one: Self::OneRequest<'a>,
                                )
                                    -> ::core::result::Result<
                                        ::std::vec::Vec<Self::ManyResponse>,
                                        ::laraxum::Error,
                                    >
                                {
                                    let response = ::sqlx::query!(#get_filter, one);
                                    let response = response.fetch(&db.pool);
                                    let response =
                                        ::futures::StreamExt::then(response, #response_getter);
                                    let response: ::std::vec::Vec<_> =
                                        ::futures::TryStreamExt::try_collect(response).await?;
                                    ::core::result::Result::Ok(response)
                                }
                            }
                        }
                    };
                    let index_token_stream = quote! {
                        pub struct #index_name;
                        #index_token_stream
                    };
                    Some(index_token_stream)
                });

            let token_stream = quote! {
                #collection_token_stream
                #request_error_token_stream
                #( #indexes )*
            };

            let Some(table_id) = table.columns.model() else {
                return token_stream;
            };

            let table_id_name = table_id.create.name;
            let table_id_name_intern = table_id.response.getter.name_intern();

            let get_one = get_filter(
                &table.name_intern,
                &table.name_extern,
                table_id_name_intern,
                &response_getter_column_elements,
                &response_getter_column_compounds,
            );
            let update_one = update_one(&table.name_intern, table_id_name, request_columns);
            let delete_one = delete_one(&table.name_intern, table_id_name);

            quote! {
                #token_stream

                impl ::laraxum::Model for #table_rs_name {
                    type Id = u64;
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
                        let response = ::sqlx::query!(#get_one, id);
                        let response = response.fetch(&db.pool);
                        let mut response = ::futures::StreamExt::then(response, #response_getter);
                        let mut response = ::core::pin::pin!(response);
                        let response = ::futures::TryStreamExt::try_next(&mut response).await?;
                        ::core::option::Option::ok_or(response, ::laraxum::Error::NotFound)
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
                        let response = ::sqlx::query!(#create_one, #request_setter_token_stream);
                        let response = response.execute(&db.pool).await?;
                        let id = response.last_insert_id();
                        #request_setter_compounds_create_many
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

                        let response = ::sqlx::query!(#update_one, #request_setter_token_stream id);
                        response.execute(&db.pool).await?;
                        #request_setter_compounds_update_many
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
                        response.execute(&db.pool).await?;
                        #request_setter_compounds_delete_many
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
            quote! {
                impl ::laraxum::Controller for #table_rs_name {
                    type State = #db_rs_name;
                    type Auth = #auth;
                    type GetManyRequestQuery = ();
                }
            }
        });

        let many_model_token_stream = table.columns.many_model().map(|(a, b)| {
            fn many_model(
                table: &stage3::Table,
                one: &stage3::ColumnOne,
                many: &stage3::ColumnOne,
            ) -> proc_macro2::TokenStream {
                let one_response_rs_ty = one.response.field.rs_ty;
                let many_response_rs_ty = many.response.field.rs_ty;

                let many_response_getter =
                    stage3::ResponseColumnGetterRef::One(&many.response.getter);

                let response_getter = response_getter(many_response_getter, false);
                let response_getter = response_getter_fn(&response_getter);

                let (response_getter_column_elements, response_getter_column_compounds) =
                    flatten(core::iter::once(many_response_getter));

                let get_many = get_filter(
                    &table.name_intern,
                    &table.name_extern,
                    one.name_intern(),
                    &response_getter_column_elements,
                    &response_getter_column_compounds,
                );
                let request_columns = [&one.request, &many.request];
                let create_one = create_one(&table.name_intern, request_columns);
                let delete_many = delete_one(&table.name_intern, one.name());

                let table_rs_name = table.rs_name;

                quote! {
                    impl ::laraxum::ManyModel<#one_response_rs_ty> for #table_rs_name {
                        type OneRequest = u64;
                        type ManyRequest = u64;
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
                            let response = ::sqlx::query!(#get_many, one);
                            let response = response.fetch(&db.pool);
                            let response = ::futures::StreamExt::then(response, #response_getter);
                            let response: ::std::vec::Vec<_> =
                                ::futures::TryStreamExt::try_collect(response).await?;
                            ::core::result::Result::Ok(response)
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
                            let transaction = db.pool.begin().await?;
                            for many in many {
                                let response = ::sqlx::query!(#create_one, one, many);
                                response.execute(&db.pool).await?;
                            }
                            transaction.commit().await?;
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
                                Self as ::laraxum::ManyModel<#one_response_rs_ty>
                            >::delete_many(db, one).await?;
                            <
                                Self as ::laraxum::ManyModel<#one_response_rs_ty>
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

        let table_token_stream = quote! {
            #table_token_stream
            #collection_model_token_stream
            #controller_token_stream
            #many_model_token_stream
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
            {migration_down}
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
                pool: ::sqlx::Pool<#db_pool_type>,
            }

            impl ::laraxum::Connect for #db_ident {
                type Error = ::sqlx::Error;
                async fn connect() -> ::core::result::Result<Self, Self::Error> {
                    let connect_options = ::laraxum::backend::database_url()
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
