use super::stage2;

use crate::{
    db::stage3,
    utils::{collections::TryCollectAll, syn::from_str_to_rs_ident},
};

use std::{borrow::Cow, vec};

use quote::quote;
use syn::{Attribute, Ident, Type};

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

impl stage2::AtomicTyString {
    fn ty(&self) -> Cow<'static, str> {
        #[cfg(feature = "mysql")]
        match self {
            Self::Varchar(len) => Cow::Owned(fmt2::fmt! { { str } => "VARCHAR(" {len} ")" }),
            Self::Char(len) => Cow::Owned(fmt2::fmt! { { str } => "CHAR(" {len} ")" }),
            Self::Text => Cow::Borrowed("TEXT"),
        }
    }
}

impl stage2::AtomicTyTime {
    pub fn ty(&self) -> &'static str {
        AtomicTyTime::from(self).ty
    }
}

struct AtomicTyTime {
    ty: &'static str,
    current_time_func: &'static str,
}
impl From<&stage2::AtomicTyTime> for AtomicTyTime {
    fn from(ty: &stage2::AtomicTyTime) -> Self {
        #[cfg(feature = "mysql")]
        match ty {
            stage2::AtomicTyTime::ChronoDateTimeUtc => Self {
                ty: "TIMESTAMP",
                current_time_func: "UTC_TIMESTAMP()",
            },
            stage2::AtomicTyTime::ChronoDateTimeLocal
            | stage2::AtomicTyTime::TimeOffsetDateTime => Self {
                ty: "TIMESTAMP",
                current_time_func: "CURRENT_TIMESTAMP()",
            },
            stage2::AtomicTyTime::ChronoNaiveDateTime
            | stage2::AtomicTyTime::TimePrimitiveDateTime => Self {
                ty: "DATETIME",
                current_time_func: "CURRENT_TIMESTAMP()",
            },
            stage2::AtomicTyTime::ChronoNaiveDate | stage2::AtomicTyTime::TimeDate => Self {
                ty: "DATE",
                current_time_func: "CURRENT_DATE()",
            },
            stage2::AtomicTyTime::ChronoNaiveTime
            | stage2::AtomicTyTime::TimeTime
            | stage2::AtomicTyTime::ChronoTimeDelta
            | stage2::AtomicTyTime::TimeDuration => Self {
                ty: "TIME",
                current_time_func: "CURRENT_TIME()",
            },
        }
    }
}

impl stage2::AtomicTy {
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

impl stage2::TyElementValue {
    fn ty(&self) -> TyValue {
        TyValue {
            ty: self.ty.ty(),
            optional: self.optional,
        }
    }
}

impl stage2::TyElementAutoTime {
    fn ty(&self) -> Ty {
        let atomic_ty_time = AtomicTyTime::from(&self.ty);
        Ty {
            ty: TyValue {
                ty: Cow::Borrowed(atomic_ty_time.ty),
                optional: false,
            },
            default_value: Some(atomic_ty_time.current_time_func),
            on_update: match self.event {
                stage2::AutoTimeEvent::OnUpdate => Some(atomic_ty_time.current_time_func),
                stage2::AutoTimeEvent::OnCreate => None,
            },
            primary_key: false,
            foreign_key: None,
            on_delete: None,
            unique: false,
        }
    }
}

impl stage2::TyElement {
    const TY_ID: &str = {
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
    };

    fn ty(&self) -> Ty {
        match self {
            Self::Value(value) => Ty {
                ty: value.ty(),

                primary_key: false,
                foreign_key: None,
                on_delete: None,
                on_update: None,
                default_value: None,
                unique: false,
            },
            Self::Id => Ty {
                ty: TyValue {
                    ty: Cow::Borrowed(Self::TY_ID),
                    optional: false,
                },
                primary_key: true,
                unique: true,

                foreign_key: None,
                on_delete: None,
                on_update: None,
                default_value: None,
            },
            Self::AutoTime(auto_time) => auto_time.ty(),
        }
    }
}

impl stage2::TyCompound {
    const TY: &str = {
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
    };

    fn ty<'ty>(&self, table_name: &'ty str, table_id_name: &'ty str) -> Ty<'ty> {
        if matches!(self.multiplicity, stage2::TyCompoundMultiplicity::Many(_)) {
            todo!("multiple foreign keys not yet implemented");
        }
        Ty {
            ty: TyValue {
                ty: Cow::Borrowed(Self::TY),
                optional: self.multiplicity.optional(),
            },
            foreign_key: Some((table_name, table_id_name)),

            primary_key: false,
            on_delete: None,
            on_update: None,
            default_value: None,
            unique: false,
        }
    }
}

// type AtomicTy<'ty> = Cow<'ty, str>;

// #[derive(Default)]
// struct TyValue {
//     ty: Cow<'static, str>,
//     optional: bool,
// }
impl fmt2::write_to::WriteTo for TyValue {
    fn write_to<W>(&self, w: &mut W) -> Result<(), W::Error>
    where
        W: fmt2::write::Write + ?Sized,
    {
        fmt2::fmt! { (? w) => {self.ty} }?;
        if !self.optional {
            fmt2::fmt! { (? w) => " NOT NULL" }?;
        }
        Ok(())
    }
}

// // #[derive(Default)]
// struct Ty<'a> {
//     ty: TyValue,
//     primary_key: bool,
//     foreign_key: Option<(&'a str, &'a str)>,
//     on_delete: Option<&'a str>,
//     on_update: Option<&'a str>,
//     default_value: Option<&'a str>,
//     unique: bool,
// }
// // #[derive(Default)]
// struct Ty<'fk_table, 'fk_id, 'default_value, 'on_update, 'on_delete> {
//     ty: TyValue,
//     primary_key: bool,
//     foreign_key: Option<(&'fk_table str, &'fk_id str)>,
//     on_delete: Option<&'on_delete str>,
//     on_update: Option<&'on_update str>,
//     default_value: Option<&'default_value str>,
//     unique: bool,
// }
impl fmt2::write_to::WriteTo for Ty<'_> {
    // impl fmt2::write_to::WriteTo for Ty<'_, '_, '_, '_, '_> {
    fn write_to<W>(&self, w: &mut W) -> Result<(), W::Error>
    where
        W: fmt2::write::Write + ?Sized,
    {
        fmt2::fmt! { (? w) => {self.ty} }?;
        if self.primary_key {
            #[cfg(feature = "mysql")]
            fmt2::fmt! { (? w) => " PRIMARY KEY AUTO_INCREMENT" }?;
            #[cfg(feature = "sqlite")]
            fmt2::fmt! { (? w) => " PRIMARY KEY AUTOINCREMENT" }?;
            #[cfg(feature = "postgres")]
            fmt2::fmt! { (? w) => " PRIMARY KEY" }?;
        }
        if let Some((table_name, table_id_name)) = self.foreign_key {
            fmt2::fmt! { (? w) => " FOREIGN KEY REFERENCES " {table_name} "(" {table_id_name} ")" }?;
        }
        if let Some(on_delete) = self.on_delete {
            fmt2::fmt! { (? w) => " ON DELETE " {on_delete} }?;
        }
        if let Some(on_update) = self.on_update {
            fmt2::fmt! { (? w) => " ON UPDATE " {on_update} }?;
        }
        if let Some(default_value) = self.default_value {
            fmt2::fmt! { (? w) => " DEFAULT " {default_value} }?;
        }
        if self.unique {
            fmt2::fmt! { (? w) => " UNIQUE" }?;
        }

        Ok(())
    }
}

// struct CreateColumn<'name, 'ty> {
//     name: &'name str,
//     ty: Ty<'ty>,
// }
// struct CreateColumn<'name, 'fk_table, 'fk_id, 'default_value, 'on_update, 'on_delete> {
//     name: &'name str,
//     ty: Ty<'fk_table, 'fk_id, 'default_value, 'on_update, 'on_delete>,
// }
impl fmt2::write_to::WriteTo for CreateColumn<'_, '_> {
    // impl fmt2::write_to::WriteTo for CreateColumn<'_, '_, '_, '_, '_, '_> {
    fn write_to<W>(&self, w: &mut W) -> Result<(), W::Error>
    where
        W: fmt2::write::Write + ?Sized,
    {
        fmt2::fmt! { (? w) => {self.name} " " {self.ty} }
    }
}

// struct ResponseColumn<'a> {
//     rs_name: &'a Ident,
//     rs_ty: &'a Type,
//     attr: &'a stage2::ColumnAttrResponse,
//     rs_attrs: &'a [Attribute],
// }
//
// struct ResponseGetterColumnElement<'a> {
//     name_intern: String,
//     name_extern: String,
//     optional: bool,
//     rs_name: &'a Ident,
// }
//
// struct ResponseGetterColumnCompound<'a> {
//     name_intern: String,
//     foreign_table_id_name_intern: String,
//     foreign_table_name_intern: String,
//     foreign_table_name_extern: String,
//     optional: bool,
//     rs_name: &'a Ident,
//     rs_ty_name: &'a Ident,
//     columns: Vec<ResponseGetterColumn<'a>>,
// }
//
// enum ResponseGetterColumn<'a> {
//     Element(ResponseGetterColumnElement<'a>),
//     Compound(ResponseGetterColumnCompound<'a>),
// }
//
// struct stage3::RequestColumn<'a> {
//     name: &'a str,
//     optional: bool,
//     rs_name: &'a Ident,
//     rs_ty: Cow<'a, Type>,
//     attr: &'a stage2::ColumnAttrRequest,
//     rs_attrs: &'a [Attribute],
// }
//
// struct Column<'a> {
//     create: CreateColumn<'a, 'a>,
//     response: ResponseColumn<'a>,
//     response_getter: ResponseGetterColumn<'a>,
//     request: Option<stage3::RequestColumn<'a>>,
// }

fn create_table<'columns, 'name: 'columns, 'ty: 'columns>(
    table_name_intern: &str,
    columns: impl IntoIterator<Item = &'columns CreateColumn<'name, 'ty>>,
) -> String {
    fmt2::fmt! { { str } =>
        "CREATE TABLE IF NOT EXISTS " {table_name_intern} " ("
            @..join(columns => "," => |c| {c})
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
    response_getter_column_elements: impl IntoIterator<
        Item = &'elements ResponseGetterColumnElement<'elements>,
    >,
    response_getter_column_compounds: impl IntoIterator<
        Item = &'compounds ResponseGetterColumnCompound<'compounds>,
    >,
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
fn get_one<'elements, 'compounds>(
    table_name_intern: &str,
    table_name_extern: &str,
    id_name_intern: &str,
    response_getter_column_elements: impl IntoIterator<
        Item = &'elements ResponseGetterColumnElement<'elements>,
    >,
    response_getter_column_compounds: impl IntoIterator<
        Item = &'compounds ResponseGetterColumnCompound<'compounds>,
    >,
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
fn create_one<'request_columns>(
    table_name_intern: &str,
    request_columns: impl IntoIterator<Item = &'request_columns stage3::RequestColumn<'request_columns>>
    + Clone,
) -> String {
    fmt2::fmt! { { str } =>
        "INSERT INTO " {table_name_intern} " ("
            @..join(request_columns.clone() => "," => |c| {c.name})
        ") VALUES ("
            @..join(request_columns => "," => |_c| "?")
        ")"
    }
}
fn update_one<'request_columns>(
    table_name_intern: &str,
    id_name: &str,
    columns: impl IntoIterator<Item = &'request_columns stage3::Column<'request_columns>>,
) -> String {
    let request_columns = columns
        .into_iter()
        .flat_map(|column| match &column.request {
            stage3::RequestColumn::Some {
                setter: stage3::RequestColumnSetter { name, .. },
                ..
            } => Some((name, "?")),
            stage3::RequestColumn::AutoTime { name, time_ty } => Some((name, time_ty.ty())),
            _ => None,
        });
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

struct Table {
    token_stream: proc_macro2::TokenStream,
    migration_up: String,
    migration_down: String,
}

impl From<stage3::Table<'_>> for Table {
    fn from(table: stage3::Table) -> Self {
        fn serde_skip() -> proc_macro2::TokenStream {
            quote! { #[serde(skip)] }
        }
        fn serde_name(serde_name: &str) -> proc_macro2::TokenStream {
            quote! { #[serde(rename = #serde_name)] }
        }

        fn map_option_to_result(
            token_stream: proc_macro2::TokenStream,
        ) -> proc_macro2::TokenStream {
            quote! {
                match #token_stream {
                    ::core::option::Option::Some(val) => ::core::result::Result::Ok(val),
                    ::core::option::Option::None => ::core::result::Result::Err(::sqlx::Error::RowNotFound),
                    // ::core::option::Option::None => ::core::result::Result::Err(::sqlx::Error::Decode(::std::string::String::new())),
                }
            }
        }

        fn rs_ty_compound_request(optional: bool) -> Type {
            if optional {
                syn::parse_quote!(Option<u64>)
            } else {
                syn::parse_quote!(u64)
            }
        }

        fn response_column_getter(
            name_extern: &str,
            optional: bool,
            foreign: bool,
        ) -> proc_macro2::TokenStream {
            let name_extern = from_str_to_rs_ident(name_extern);
            let field_access = quote! { response.#name_extern };

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
                            ::core::option::Option::None => break 'response_block ::core::option::Option::None,
                        }
                    }
                }
            } else {
                field_access
            };

            if optional {
                quote! { ::core::option::Option::map(#field_access, ::laraxum::Decode::decode) }
            } else {
                quote! { ::laraxum::Decode::decode(#field_access) }
            }
        }
        fn response_getter<'columns>(
            table_ty: &Ident,
            columns: impl IntoIterator<Item = &'columns ResponseGetterColumn<'columns>>,
            optional: bool,
            foreign: bool,
        ) -> proc_macro2::TokenStream {
            let columns = columns.into_iter().map(|column| match column {
                ResponseGetterColumn::Element(element) => {
                    let ResponseGetterColumnElement {
                        name_extern,
                        optional,
                        rs_name,
                        ..
                    } = element;
                    let getter = response_column_getter(name_extern, *optional, foreign);
                    quote! {
                        #rs_name: #getter
                    }
                }
                ResponseGetterColumn::Compound(compound) => {
                    let ResponseGetterColumnCompound {
                        optional,
                        rs_name,
                        rs_ty_name,
                        columns,
                        ..
                    } = compound;
                    let getter = response_getter(rs_ty_name, columns.iter(), *optional, true);
                    quote! {
                        #rs_name: #getter
                    }
                }
            });
            let getter = quote! { #table_ty { #( #columns ),* } };
            if optional {
                #[cfg(feature = "try_blocks")]
                quote! {
                    try { #getter }
                }
                #[cfg(not(feature = "try_blocks"))]
                quote! {
                    'response_block: { ::core::option::Option::Some(#getter) }
                }
            } else {
                getter
            }
        }

        fn flatten_internal<'columns>(
            response_getter_columns: impl Iterator<Item = &'columns ResponseGetterColumn<'columns>>,
            response_getter_column_elements: &mut Vec<
                &'columns ResponseGetterColumnElement<'columns>,
            >,
            response_getter_column_compounds: &mut Vec<
                &'columns ResponseGetterColumnCompound<'columns>,
            >,
        ) {
            for response_getter_column in response_getter_columns {
                match response_getter_column {
                    ResponseGetterColumn::Element(element) => {
                        response_getter_column_elements.push(element);
                    }
                    ResponseGetterColumn::Compound(compound) => {
                        response_getter_column_compounds.push(compound);
                        flatten_internal(
                            compound.columns.iter(),
                            response_getter_column_elements,
                            response_getter_column_compounds,
                        );
                    }
                }
            }
        }
        fn flatten<'columns>(
            response_getter_columns: impl Iterator<Item = &'columns ResponseGetterColumn<'columns>>,
        ) -> (
            Vec<&'columns ResponseGetterColumnElement<'columns>>,
            Vec<&'columns ResponseGetterColumnCompound<'columns>>,
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

        let response_column_fields = table.columns.iter().map(|column| {
            let ResponseColumn {
                rs_name,
                rs_ty,
                attr,
                rs_attrs,
            } = column;

            let serde_skip = attr.skip.then(serde_skip);
            let serde_name = attr.name.as_deref().map(serde_name);

            quote! {
                #(#rs_attrs)* #serde_skip #serde_name
                pub #rs_name: #rs_ty
            }
        });

        let (response_getter_column_elements, response_getter_column_compounds) =
            flatten(table.columns.iter().map(|column| &column.response_getter));
        let response_getter = response_getter(
            &table.rs_name,
            table.columns.iter().map(|column| &column.response_getter),
            true,
            false,
        );
        let response_getter = map_option_to_result(response_getter);

        let request_column_fields =
            table
                .columns
                .iter()
                .flat_map(|column| &column.request)
                .map(|column| {
                    let stage3::RequestColumn {
                        rs_name,
                        rs_ty,
                        attr,
                        rs_attrs,
                        ..
                    } = column;

                    let serde_skip = attr.skip.then(serde_skip);
                    let serde_name = attr.name.as_deref().map(serde_name);

                    quote! {
                        #(#rs_attrs)* #serde_skip #serde_name
                        pub #rs_name: #rs_ty
                    }
                });

        let request_columns_setters =
            table
                .columns
                .iter()
                .flat_map(|column| &column.request)
                .map(|column| {
                    let stage3::RequestColumn {
                        optional, rs_name, ..
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
        let request_columns_setters = quote! { #(#request_columns_setters,)* };

        let create_table = create_table(
            &table_name_intern,
            table.columns.iter().map(|column| &column.create),
        );
        let delete_table = delete_table(&table_name_intern);

        let get_all = get_all(
            &table_name_intern,
            &table_name_extern,
            response_getter_column_elements.iter().copied(),
            response_getter_column_compounds.iter().copied(),
        );
        let create_one = create_one(
            &table_name_intern,
            table.columns.iter().flat_map(|column| &column.request),
        );

        let table_rs_name = &table.rs_name;
        let table_rs_name_request = quote::format_ident!("{}Request", table.rs_name);
        let table_rs_attrs = &*table.rs_attrs;
        let db_rs_name = &db.rs_name;
        let doc = fmt2::fmt! { { str } => "`" {table_name_intern} "`"};
        let table_token_stream = quote! {
            #[doc = #doc]
            #[derive(::serde::Serialize)]
            #(#table_rs_attrs)*
            pub struct #table_rs_name {
                #(#response_column_fields),*
            }

            #[derive(::serde::Deserialize)]
            pub struct #table_rs_name_request {
                #(#request_column_fields),*
            }
        };

        let collection_token_stream = table.columns.is_collection().then(|| quote! {
            impl ::laraxum::Db<#table_rs_name> for #db_rs_name {}

            impl ::laraxum::Table for #table_rs_name {
                type Db = #db_rs_name;
                type Response = #table_rs_name;
            }

            impl ::laraxum::Collection for #table_rs_name {
                type GetAllRequestQuery = ();
                type CreateRequest = #table_rs_name_request;
                type CreateRequestError = ();

                /// `get_all`
                ///
                /// ```sql
                #[doc = #get_all]
                /// ```
                async fn get_all(db: &Self::Db) -> ::core::result::Result<::std::vec::Vec<Self::Response>, ::laraxum::Error> {
                    let response = ::sqlx::query!(#get_all)
                        .try_map(|response| #response_getter)
                        .fetch_all(&db.pool)
                        .await?;
                    ::core::result::Result::Ok(response)
                }
                async fn create_one(
                    db: &Self::Db,
                    request: Self::CreateRequest,
                ) -> ::core::result::Result<(), ::laraxum::ModelError<Self::CreateRequestError>> {
                    let response = ::sqlx::query!(
                            #create_one,
                            #request_columns_setters
                        )
                        .execute(&db.pool)
                        .await?;
                    ::core::result::Result::Ok(())
                }
            }
        });

        let model_token_stream = table.columns.model().map(|table_id| {
            let id_name_intern = name_intern((&*table_name_extern, &table_id.name));
            let get_one = get_one(
                &table_name_intern,
                &table_name_extern,
                &id_name_intern,
                response_getter_column_elements.iter().copied(),
                response_getter_column_compounds.iter().copied(),
            );
            let update_one = update_one(&table_name_intern, &table_id.name, table.columns.iter().flat_map(|column|&column.request));
            let delete_one = delete_one(&table_name_intern, &table_id.name);

            quote! {
                impl ::laraxum::Model for #table_rs_name {
                    type Id = u64;
                    type UpdateRequest = #table_rs_name_request;
                    type UpdateRequestError = ();

                    /// `get_one`
                    ///
                    /// ```sql
                    #[doc = #get_one]
                    /// ```
                    async fn get_one(db: &Self::Db, id: Self::Id) -> ::core::result::Result<Self::Response, ::laraxum::Error>{
                        let response = ::sqlx::query!(#get_one, id)
                            .try_map(|response| #response_getter)
                            .fetch_one(&db.pool)
                            .await?;
                        ::core::result::Result::Ok(response)
                    }
                    async fn create_get_one(
                        db: &Self::Db,
                        request: Self::CreateRequest,
                    ) -> ::core::result::Result<Self::Response, ::laraxum::ModelError<Self::CreateRequestError>> {
                        let response = ::sqlx::query!(#create_one, #request_columns_setters)
                            .execute(&db.pool)
                            .await?;
                        let response = Self::get_one(db, response.last_insert_id()).await?;
                        ::core::result::Result::Ok(response)
                    }
                    async fn update_one(
                        db: &Self::Db,
                        request: Self::UpdateRequest,
                        id: Self::Id,
                    ) -> ::core::result::Result<(), ::laraxum::ModelError<Self::UpdateRequestError>> {
                        ::sqlx::query!(#update_one, #request_columns_setters id)
                            .execute(&db.pool)
                            .await?;
                        ::core::result::Result::Ok(())
                    }
                    async fn update_get_one(
                        db: &Self::Db,
                        request: Self::UpdateRequest,
                        id: Self::Id,
                    ) -> ::core::result::Result<Self::Response, ::laraxum::ModelError<Self::UpdateRequestError>> {
                        Self::update_one(db, request, id).await?;
                        let response = Self::get_one(db, id).await?;
                        ::core::result::Result::Ok(response)
                    }
                    async fn delete_one(db: &Self::Db, id: Self::Id) -> ::core::result::Result<(), ::laraxum::Error> {
                        ::sqlx::query!(#delete_one, id)
                            .execute(&db.pool)
                            .await?;
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

        let many_model_token_stream = table.columns.many_model().map(|(_a, _b)| {
            // TODO:
            quote! {
                impl ManyModel<OneResponse>: Table {
                    type OneRequest;
                    type ManyRequest;
                    type ManyResponse;

                    async fn get_many(
                        db: &Self::Db,
                        one: Self::OneRequest,
                    ) -> Result<Vec<Self::ManyResponse>, Error>;
                    async fn create_many(
                        db: &Self::Db,
                        one: Self::OneRequest,
                        many: &[Self::ManyRequest],
                    ) -> Result<(), Error>;
                    async fn update_many(
                        db: &Self::Db,
                        one: Self::OneRequest,
                        many: &[Self::ManyRequest],
                    ) -> Result<(), Error>;
                    async fn delete_many(db: &Self::Db, one: Self::OneRequest) -> Result<(), Error>;
                }
            }
        });

        let table_token_stream = quote! {
            #table_token_stream
            #collection_token_stream
            #model_token_stream
            #controller_token_stream
            #many_model_token_stream
        };

        Ok(Self {
            token_stream: table_token_stream,
            migration_up: create_table,
            migration_down: delete_table,
        })
    }
}

pub use proc_macro2::TokenStream as Db;

impl TryFrom<stage2::Db> for Db {
    type Error = syn::Error;
    fn try_from(db: stage2::Db) -> Result<Self, Self::Error> {
        let tables: Result<Vec<Table>, syn::Error> = db
            .tables
            .iter()
            .map(|table| Table::try_new(table, &db))
            .try_collect_all_default();
        let tables = tables?;

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

        Ok(quote! {
            /// ```sql
            #[doc = #migration_up_full]
            /// ```
            pub struct #db_ident {
                pool: ::sqlx::Pool<#db_pool_type>,
            }

            impl ::laraxum::AnyDb for #db_ident {
                type Db = Self;
                async fn connect_with_str(s: &str) -> ::core::result::Result::<Self, ::sqlx::Error> {
                    ::core::result::Result::Ok(Self {
                        pool: ::sqlx::Pool::<#db_pool_type>::connect(s).await?,
                    })
                }
                fn db(&self) -> &Self::Db {
                    self
                }
            }

            #(#tables_token_stream)*
        })
    }
}
