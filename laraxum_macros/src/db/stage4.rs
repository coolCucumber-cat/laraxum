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
            Self::ChronoDateTimeUtc => "TIMESTAMP",
            Self::ChronoDateTimeLocal | Self::TimeOffsetDateTime => "TIMESTAMP",
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
            Self::ChronoDateTimeLocal | Self::TimeOffsetDateTime => "CURRENT_TIMESTAMP()",
            Self::ChronoNaiveDateTime | Self::TimePrimitiveDateTime => "CURRENT_TIMESTAMP()",
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

    /// If the type requires the constraint `UNIQUE`, not if it is unique
    ///
    /// A primary key is unique but it doesn't require the constraint, since it is already implied
    const fn unique_constraint(&self) -> bool {
        match self {
            Self::Id => false,
            // TODO: unique
            Self::Value(_value) => false,
            Self::AutoTime(_) => false,
        }
    }
}

impl stage3::TyCompound<'_> {
    fn ty(&self) -> &'static str {
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

    const fn unique_constraint(&self) -> bool {
        match self {
            Self::Compound(compound) => compound.unique(),
            Self::Element(element) => element.unique_constraint(),
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
        if self.unique_constraint() {
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
            Self::Compound(stage3::TyCompound {
                foreign_table_name,
                foreign_table_id_name,
                ..
            }) => {
                fmt2::fmt! { (? w) => " FOREIGN KEY REFERENCES " {foreign_table_name} "(" {foreign_table_id_name} ")" }?;
            }
            _ => {}
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
    pub fn request_setter_column(&self) -> Option<(&str, &str)> {
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

fn create_table<'columns>(
    table_name_intern: &str,
    columns: impl IntoIterator<Item = &'columns stage3::Column<'columns>>,
) -> String {
    let create_columns = columns.into_iter().flat_map(|column| column.create());

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
fn get_all<'elements, 'compounds>(
    table_name_intern: &str,
    table_name_extern: &str,
    response_getter_column_elements: &[&stage3::ResponseColumnGetterElement<'elements>],
    response_getter_column_compounds: &[&stage3::ResponseColumnGetterCompound<'compounds>],
) -> String {
    fmt2::fmt! { { str } =>
        "SELECT "
        @..join(response_getter_column_elements => "," => |element|
            {element.name_intern}
            " AS "
            {element.name_extern}
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
fn get_filter<'elements, 'compounds>(
    table_name_intern: &str,
    table_name_extern: &str,
    id_name_intern: &str,
    response_getter_column_elements: &[&stage3::ResponseColumnGetterElement<'elements>],
    response_getter_column_compounds: &[&stage3::ResponseColumnGetterCompound<'compounds>],
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
        .flat_map(|column| column.request_setter_column());
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
        .flat_map(|column| column.request_setter_column());
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
    response_getter_column_elements: &mut Vec<
        &'columns stage3::ResponseColumnGetterElement<'columns>,
    >,
    response_getter_column_compounds: &mut Vec<
        &'columns stage3::ResponseColumnGetterCompound<'columns>,
    >,
) {
    for response_getter_column in response_getter_columns {
        match response_getter_column {
            stage3::ResponseColumnGetterRef::One(stage3::ResponseColumnGetterOne::Element(
                element,
            )) => {
                response_getter_column_elements.push(element);
            }
            stage3::ResponseColumnGetterRef::One(stage3::ResponseColumnGetterOne::Compound(
                compound,
            )) => {
                response_getter_column_compounds.push(compound);
                flatten_internal(
                    compound
                        .columns
                        .iter()
                        .map(stage3::ResponseColumnGetterRef::from),
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
    Vec<&'columns stage3::ResponseColumnGetterElement<'columns>>,
    Vec<&'columns stage3::ResponseColumnGetterCompound<'columns>>,
) {
    let mut response_getter_column_elements = vec![];
    let mut response_getter_column_compounds = vec![];

    flatten_internal(
        response_getter_columns,
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
    foreign: bool,
) -> proc_macro2::TokenStream {
    let field_access = quote! {
        response.#name_extern
    };

    let field_access = if foreign && !optional {
        #[cfg(feature = "try_blocks")]
        {
            quote! {
                #field_access?
            }
        }
        #[cfg(not(feature = "try_blocks"))]
        {
            quote! {
                match #field_access {
                    ::core::option::Option::Some(val) => val,
                    ::core::option::Option::None => {
                        break 'response_block ::core::option::Option::None
                    }
                }
            }
        }
    } else {
        field_access
    };
    if optional {
        quote! {
            ::core::option::Option::map(#field_access, ::laraxum::Decode::decode)
        }
    } else {
        quote! {
            ::laraxum::Decode::decode(#field_access)
        }
    }
}

fn response_getter(
    column: stage3::ResponseColumnGetterRef<'_>,
    foreign: bool,
) -> proc_macro2::TokenStream {
    match column {
        stage3::ResponseColumnGetterRef::One(stage3::ResponseColumnGetterOne::Element(element)) => {
            let stage3::ResponseColumnGetterElement {
                name_extern,
                optional,
                ..
            } = element;
            let name_extern = from_str_to_rs_ident(name_extern);
            response_getter_column(&name_extern, *optional, foreign)
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
            let getter = response_getter_compound(
                rs_ty_name,
                columns.iter().map(stage3::ResponseColumnGetterRef::from),
                true,
            );
            if *optional {
                // catch any early returns and replace it with `None`
                catch_option(&getter)
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
            let table_id_name_extern = from_str_to_rs_ident(table_id_name_extern);
            let one_id = response_getter_column(&table_id_name_extern, false, foreign);
            quote! {
                match {
                    <#many_foreign_table_rs_name as ::laraxum::ManyModel::<
                        #table_rs_name,
                    >>::get_many(
                        db,
                        #one_id,
                    ).await
                } {
                    ::core::result::Result::Ok(response) => response,
                    ::core::result::Result::Err(_) => {
                        return ::core::result::Result::Err(::sqlx::Error::RowNotFound);
                    }
                }
            }
        }
    }
}

fn response_getter_compound<'columns>(
    table_ty: &Ident,
    columns: impl IntoIterator<Item = stage3::ResponseColumnGetterRef<'columns>>,
    foreign: bool,
) -> proc_macro2::TokenStream {
    let columns = columns.into_iter().map(|column| {
        let rs_name = column.rs_name();
        let response_getter = response_getter(column, foreign);
        quote! {
            #rs_name: #response_getter
        }
    });

    quote! {
        #table_ty { #( #columns ),* }
    }
}

fn catch_option(getter: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    #[cfg(feature = "try_blocks")]
    quote! {
        try { #getter }
    }
    #[cfg(not(feature = "try_blocks"))]
    quote! {
        'response_block: { ::core::option::Option::Some(#getter) }
    }
}

fn response_getter_fn(getter: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let getter = catch_option(getter);
    quote! {
        async |response| match response {
            ::core::result::Result::Ok(response) => {
                match #getter {
                    ::core::option::Option::Some(response) => {
                        ::core::result::Result::Ok(response)
                    }
                    ::core::option::Option::None => {
                        ::core::result::Result::Err(::sqlx::Error::RowNotFound)
                    }
                }
            }
            ::core::result::Result::Err(err) => ::core::result::Result::Err(err),
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

            impl ::laraxum::Decode for #table_rs_name {
                type Decode = Self;
                #[inline]
                fn decode(decode: Self::Decode) -> Self {
                    decode
                }
            }

            impl ::laraxum::Encode for #table_rs_name {
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

            let request_setter_columns = table
                .columns
                .iter()
                .filter_map(|column| match &column {
                    stage3::Column::One(stage3::ColumnOne {
                        request: stage3::RequestColumnOne::Some { setter, .. },
                        ..
                    }) => Some(setter),
                    _ => None,
                })
                .map(|column| {
                    let stage3::RequestColumnSetterOne {
                        rs_name, optional, ..
                    } = column;
                    if *optional {
                        quote! {
                            ::core::option::Option::map(request.#rs_name, ::laraxum::Encode::encode)
                        }
                    } else {
                        quote! {
                            ::laraxum::Encode::encode(request.#rs_name)
                        }
                    }
                });
            let request_setter = quote! {
                #(#request_setter_columns,)*
            };

            let request_setter_compounds_columns =
                table.columns.iter().filter_map(|column| match column {
                    stage3::Column::Compounds(compounds) => Some(&compounds.request.setter),
                    _ => None,
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

            let request_column_validate = table
                .columns
                .iter()
                .filter_map(|column| column.request_field())
                .filter(|column| !column.attr.validate.is_empty());
            let request_error_token_stream = if request_column_validate.clone().next().is_some() {
                let request_error_columns = request_column_validate.clone().map(
                    |stage3::RequestColumnField { rs_name, .. }| {
                        quote! {
                            // #[serde(skip_serializing_if = "(|v| v.is_empty())")]
                            #[serde(skip_serializing_if = "<[&'static str]>::is_empty")]
                            pub #rs_name: ::std::vec::Vec::<&'static str>
                            // #[serde(skip_serializing_if = "::core::option::Option::is_none")]
                            // pub #rs_name: ::core::option::Option::<::std::vec::Vec::<&'static str>>
                        }
                    },
                );
                let request_column_validate_rules = request_column_validate
                    .flat_map(|stage3::RequestColumnField { rs_name, attr, .. }| {
                        attr.validate
                            .iter()
                            .map(|validate_rule| (*rs_name, validate_rule))
                    })
                    .map(|(rs_name, validate_rule)| {
                        use super::stage1::ValidateRule;
                        use crate::utils::syn::TokenStreamAttr;
                        // let result = quote! {
                        //     (#validate_rule)(&self.#rs_name)
                        // };
                        let result = match validate_rule {
                            ValidateRule::Func(TokenStreamAttr(f)) => {
                                quote! {
                                    (#f)(&self.#rs_name)
                                }
                            }
                            ValidateRule::Range(TokenStreamAttr(range)) => {
                                let err_message =
                                    format!("value must be in range {}", range.to_token_stream());
                                quote! {
                                    if (#range).contains(&self.#rs_name) {
                                        ::core::result::Result::Ok(())
                                    } else {
                                        ::core::result::Result::Err(#err_message)
                                    }
                                }
                            }
                        };
                        quote! {
                            if let ::core::result::Result::Err(err) = #result {
                                ::laraxum::error_builder::<(), #table_request_error_rs_name>(
                                    &mut e,
                                    |e| e.#rs_name.push(err),
                                );
                            }
                        }
                    });

                quote! {
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

                    impl ::laraxum::Request::<::laraxum::request_type::Create>
                        for #table_request_rs_name
                    {
                        type Error = #table_request_error_rs_name;
                        fn validate(&self) -> ::core::result::Result::<(), Self::Error> {
                            let mut e = ::core::result::Result::Ok(());
                            #( #request_column_validate_rules )*
                            e
                        }
                    }
                    impl ::laraxum::Request::<::laraxum::request_type::Update>
                        for #table_request_rs_name
                    {
                        type Error = #table_request_error_rs_name;
                        fn validate(&self) -> ::core::result::Result::<(), Self::Error> {
                            <
                                Self as ::laraxum::Request::<::laraxum::request_type::Create>
                            >::validate(self)
                        }
                    }

                }
            } else {
                quote! {
                    pub type #table_request_error_rs_name = ();
                    // pub type #table_request_error_rs_name = ::core::convert::Infallible;
                    impl ::laraxum::Request::<::laraxum::request_type::Create>
                        for #table_request_rs_name
                    {
                        type Error = #table_request_error_rs_name;
                        fn validate(&self) -> ::core::result::Result::<(), Self::Error> {
                            ::core::result::Result::Ok(())
                        }
                    }
                    impl ::laraxum::Request::<::laraxum::request_type::Update>
                        for #table_request_rs_name
                    {
                        type Error = #table_request_error_rs_name;
                        fn validate(&self) -> ::core::result::Result::<(), Self::Error> {
                            ::core::result::Result::Ok(())
                        }
                    }

                }
            };

            let get_all = get_all(
                &table.name_intern,
                &table.name_extern,
                &response_getter_column_elements,
                &response_getter_column_compounds,
            );
            let request_columns = table.columns.iter().flat_map(|column| column.request_one());
            let create_one = create_one(&table.name_intern, request_columns.clone());

            let collection_token_stream = quote! {
                #[derive(::serde::Deserialize)]
                pub struct #table_request_rs_name {
                    #(#request_column_fields),*
                }

                impl ::laraxum::Collection for #table_rs_name {
                    type GetAllRequestQuery = ();
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
                            as ::laraxum::Request::<::laraxum::request_type::Create>
                        >::validate(&request)?;
                        let response = ::sqlx::query!(#create_one, #request_setter);
                        let response = response.execute(&db.pool).await?;
                        let id = response.last_insert_id();
                        #request_setter_compounds_create_many
                        ::core::result::Result::Ok(())
                    }
                }
            };

            let Some(table_id) = table.columns.model() else {
                return collection_token_stream;
            };

            let (table_id_name, table_id_name_intern) = match table_id {
                stage3::Column::One(one) => (one.create.name, one.response.getter.name_intern()),
                stage3::Column::Compounds(_) => {
                    unimplemented!("unreachable error. id does not have corresponding fields");
                }
            };

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
                #request_error_token_stream
                #collection_token_stream

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
                            as ::laraxum::Request::<::laraxum::request_type::Create>
                        >::validate(&request)?;
                        let response = ::sqlx::query!(#create_one, #request_setter);
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
                            as ::laraxum::Request::<::laraxum::request_type::Update>
                        >::validate(&request)?;

                        let response = ::sqlx::query!(#update_one, #request_setter id);
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

        let controller_token_stream = table.columns.is_controller().then(|| {
            quote! {
                impl ::laraxum::Controller for #table_rs_name {
                    type State = #db_rs_name;
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
            let a = match a {
                stage3::Column::One(one) => one,
                stage3::Column::Compounds(_) => {
                    unimplemented!("unreachable error. id does not have corresponding fields")
                }
            };
            let b = match b {
                stage3::Column::One(one) => one,
                stage3::Column::Compounds(_) => {
                    unimplemented!("unreachable error. id does not have corresponding fields")
                }
            };

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

            impl ::laraxum::AnyDb for #db_ident {
                type Db = Self;
                type Driver = #db_pool_type;
                type ConnectionOptions = <
                    <
                        Self::Driver as sqlx::Database
                    >::Connection as sqlx::Connection
                >::Options;
                fn default_options()
                -> Self::ConnectionOptions
                {
                    ::core::default::Default::default()
                }
                async fn connect_with_options(
                    options: Self::ConnectionOptions,
                ) -> Result<Self, sqlx::Error> {
                    ::core::result::Result::Ok(Self {
                        pool: ::sqlx::Pool::<Self::Driver>::connect_with(options).await?,
                    })
                }
                fn db(&self) -> &Self::Db {
                    self
                }
            }

            #(#tables_token_stream)*
        }
    }
}
