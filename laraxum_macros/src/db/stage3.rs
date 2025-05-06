use super::stage2;

use crate::utils::syn::from_str_to_rs_ident;

use std::borrow::Cow;

use quote::quote;
use syn::{Attribute, Ident, Type};

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

fn make_not_optional(sql_ty: impl Into<String>) -> String {
    let mut sql_ty = sql_ty.into();
    fmt2::fmt! { (sql_ty) => " NOT NULL" };
    sql_ty
}
fn make_maybe_optional(sql_ty: Cow<str>, optional: bool) -> Cow<str> {
    if optional {
        sql_ty
    } else {
        Cow::Owned(make_not_optional(sql_ty.into_owned()))
    }
}

fn rs_ty_foreign_id(optional: bool) -> Type {
    if optional {
        syn::parse_quote!(Option<u64>)
    } else {
        syn::parse_quote!(u64)
    }
}

impl stage2::AtomicTyString {
    fn sql_ty(&self) -> Cow<'static, str> {
        #[cfg(feature = "mysql")]
        match self {
            Self::Varchar(len) => Cow::Owned(fmt2::fmt! { { str } => "VARCHAR(" {len} ")" }),
            Self::Char(len) => Cow::Owned(fmt2::fmt! { { str } => "CHAR(" {len} ")" }),
            Self::Text => Cow::Borrowed("TEXT"),
        }
    }
}

impl stage2::AtomicTyTime {
    fn sql_ty(&self) -> &'static str {
        AtomicTyTime::from(self).sql_ty
    }
}

struct AtomicTyTime {
    sql_ty: &'static str,
    sql_current_time_func: &'static str,
}
impl From<&stage2::AtomicTyTime> for AtomicTyTime {
    fn from(ty: &stage2::AtomicTyTime) -> Self {
        #[cfg(feature = "mysql")]
        match ty {
            stage2::AtomicTyTime::ChronoDateTimeUtc => Self {
                sql_ty: "TIMESTAMP",
                sql_current_time_func: "UTC_TIMESTAMP()",
            },
            stage2::AtomicTyTime::ChronoDateTimeLocal
            | stage2::AtomicTyTime::TimeOffsetDateTime => Self {
                sql_ty: "TIMESTAMP",
                sql_current_time_func: "CURRENT_TIMESTAMP()",
            },
            stage2::AtomicTyTime::ChronoNaiveDateTime
            | stage2::AtomicTyTime::TimePrimitiveDateTime => Self {
                sql_ty: "DATETIME",
                sql_current_time_func: "CURRENT_TIMESTAMP()",
            },
            stage2::AtomicTyTime::ChronoNaiveDate | stage2::AtomicTyTime::TimeDate => Self {
                sql_ty: "DATE",
                sql_current_time_func: "CURRENT_DATE()",
            },
            stage2::AtomicTyTime::ChronoNaiveTime
            | stage2::AtomicTyTime::TimeTime
            | stage2::AtomicTyTime::ChronoTimeDelta
            | stage2::AtomicTyTime::TimeDuration => Self {
                sql_ty: "TIME",
                sql_current_time_func: "CURRENT_TIME()",
            },
        }
    }
}

impl stage2::AtomicTy {
    fn sql_ty(&self) -> Cow<'static, str> {
        #[cfg(feature = "mysql")]
        {
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

                Self::String(string) => string.sql_ty(),
                Self::Time(time) => Cow::Borrowed(time.sql_ty()),
            }
        }

        // #[cfg(feature = "postgres")]
        // ty_enum! {
        //     enum ColumnTyPrimitiveInner {
        //         Id(Id) => u64 => "SERIAL PRIMARY KEY",
        //         String(String) => ::std::string::String => "VARCHAR(255)",
        //         bool(bool) => bool => "BOOL",
        //         i8(i8) => i8 => "CHAR",  // TINYINT
        //         i16(i16) => i16 => "INT2", // SMALLINT
        //         i32(i32) => i32 => "INT4", // INT
        //         i64(i64) => i64 => "INT8", // BIGINT
        //         f32(f32) => f32 => "FLOAT4", // FLOAT
        //         f64(f64) => f64 => "FLOAT8", // DOUBLE
        //     }
        // }
        //
        // #[cfg(feature = "sqlite")]
        // ty_enum! {
        //     enum ColumnTyPrimitiveInner {
        //         Id(Id) => u64 => "INTEGER PRIMARY KEY AUTOINCREMENT",
        //         String(String) => ::std::string::String => "TEXT",
        //         bool(bool) => bool => "BOOLEAN",
        //         u8(u8) => u8 => "INTEGER",
        //         i8(i8) => i8 => "INTEGER",
        //         u16(u16) => u16 => "INTEGER",
        //         i16(i16) => i16 => "INTEGER",
        //         u32(u32) => u32 => "INTEGER",
        //         i32(i32) => i32 => "INTEGER",
        //         u64(u64) => u64 => "INTEGER",
        //         i64(i64) => i64 => "BIGINT",
        //         f32(f32) => f32 => "FLOAT",
        //         f64(f64) => f64 => "DOUBLE",
        //     }
        // }
    }
}

impl stage2::TyElementValue {
    fn sql_ty(&self) -> Cow<'static, str> {
        let sql_ty = self.ty.sql_ty();
        make_maybe_optional(sql_ty, self.optional)
    }
}

impl stage2::TyElementAutoTime {
    fn sql_ty(&self) -> String {
        let atomic_ty_time = AtomicTyTime::from(&self.ty);
        let mut sql_ty = make_not_optional(atomic_ty_time.sql_ty);
        fmt2::fmt! { (sql_ty) =>
            " DEFAULT " {atomic_ty_time.sql_current_time_func}
        }
        if matches!(self.event, stage2::AutoTimeEvent::OnUpdate) {
            fmt2::fmt! { (sql_ty) =>
                " ON UPDATE " {atomic_ty_time.sql_current_time_func}
            }
        }
        sql_ty
    }
}

impl stage2::TyElement {
    const SQL_TY_ID: &str = {
        #[cfg(feature = "mysql")]
        {
            "BIGINT UNSIGNED NOT NULL UNIQUE PRIMARY KEY AUTO_INCREMENT"
        }
    };

    fn sql_ty(&self) -> Cow<str> {
        match self {
            Self::Value(value) => value.sql_ty(),
            Self::Id => Cow::Borrowed(Self::SQL_TY_ID),
            Self::AutoTime(auto_time) => Cow::Owned(auto_time.sql_ty()),
        }
    }
}

impl stage2::TyCompound {
    fn sql_ty(&self, table_name: &str, table_id_name: &str) -> String {
        #[cfg(feature = "mysql")]
        {
            let sql_ty = make_maybe_optional(
                Cow::Borrowed("BIGINT UNSIGNED"),
                self.multiplicity.optional(),
            );
            let mut sql_ty = sql_ty.into_owned();
            fmt2::fmt! { (sql_ty) => " FOREIGN KEY REFERENCES " {table_name} "(" {table_id_name} ")" }
            sql_ty
        }
    }
}

// impl stage2::TyElement {
//     fn transform_response(&self, expr: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
//         type TransformFn = fn(&proc_macro2::TokenStream) -> proc_macro2::TokenStream;
//
//         #[allow(clippy::match_single_binding)]
//         let transform: Option<TransformFn> = match self {
//             #[cfg(feature = "mysql")]
//             Self::Value(stage2::TyElementValue {
//                 ty: stage2::AtomicTy::bool,
//                 optional: _,
//             }) => Some(|ts| quote! { #ts != 0 }),
//             _ => None,
//         };
//
//         if let Some(transform) = transform {
//             if self.optional() {
//                 let val_name = quote! { val };
//                 let transformed = transform(&val_name);
//                 quote! { ::core::option::Option::map(#expr, |#val_name| #transformed) }
//             } else {
//                 transform(&expr)
//             }
//         } else {
//             expr
//         }
//     }
// }

struct RequestColumn<'name> {
    name: &'name str,
    field: proc_macro2::TokenStream,
    setter: proc_macro2::TokenStream,
}

struct ResponseColumnName {
    name_intern: String,
    name_extern: String,
}

struct Join {
    foreign_table_name_intern: String,
    foreign_table_name_extern: String,
    foreign_table_id_name_intern: String,
    column_name_intern: String,
}

struct Table {
    token_stream: proc_macro2::TokenStream,
    migration_up: String,
    migration_down: String,
}

impl Table {
    fn new(table: &stage2::Table, db: &stage2::Db) -> Self {
        fn serde_skip() -> proc_macro2::TokenStream {
            quote! { #[serde(skip)] }
        }
        fn serde_name(serde_name: &str) -> proc_macro2::TokenStream {
            quote! { #[serde(rename = #serde_name)] }
        }

        fn request_column_field(
            rs_name: &Ident,
            rs_ty: &Type,
            attr: &stage2::ColumnAttrRequest,
            rs_attrs: &[Attribute],
        ) -> proc_macro2::TokenStream {
            let serde_skip = attr.skip.then(serde_skip);
            let serde_name = attr.name.as_deref().map(serde_name);
            quote! {
                #(#rs_attrs)* #serde_skip #serde_name
                pub #rs_name: #rs_ty
            }
        }
        fn request_column_setter(rs_name: &Ident, optional: bool) -> proc_macro2::TokenStream {
            if optional {
                quote! { ::core::option::Option::map(request.#rs_name, ::laraxum::Encode::encode) }
            } else {
                quote! { ::laraxum::Encode::encode(request.#rs_name) }
            }
        }

        fn response_column_field(
            rs_name: &Ident,
            rs_ty: &Type,
            attr: &stage2::ColumnAttrResponse,
            rs_attrs: &[Attribute],
        ) -> proc_macro2::TokenStream {
            let serde_skip = attr.skip.then(serde_skip);
            let serde_name = attr.name.as_deref().map(serde_name);
            quote! {
                #(#rs_attrs)* #serde_skip #serde_name
                pub #rs_name: #rs_ty
            }
        }
        fn response_column_getter_field_access(name_extern: &str) -> proc_macro2::TokenStream {
            let name_extern = from_str_to_rs_ident(name_extern);
            quote! { response.#name_extern }
        }
        fn response_column_getter_decode(
            field_access: proc_macro2::TokenStream,
            optional: bool,
        ) -> proc_macro2::TokenStream {
            if optional {
                quote! { ::core::option::Option::map(#field_access, ::laraxum::Decode::decode) }
            } else {
                quote! { ::laraxum::Decode::decode(#field_access) }
            }
        }
        fn response_column_getter(name_extern: &str, optional: bool) -> proc_macro2::TokenStream {
            let field_access = response_column_getter_field_access(name_extern);
            response_column_getter_decode(field_access, optional)
        }
        fn response_column_getter_foreign(
            name_extern: &str,
            optional: bool,
        ) -> proc_macro2::TokenStream {
            let field_access = response_column_getter_field_access(name_extern);
            let field_access = if optional {
                field_access
            } else {
                #[cfg(feature = "try_blocks")]
                {
                    quote! { #field_access? }
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
            };
            response_column_getter_decode(field_access, optional)
        }
        fn response_table_getter<'response_name>(
            table_ty: &Ident,
            columns: impl IntoIterator<Item = (&'response_name Ident, proc_macro2::TokenStream)>,
            optional: bool,
        ) -> proc_macro2::TokenStream {
            let columns = columns.into_iter().map(|(field_name, getter)| {
                quote! {
                    #field_name: #getter
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
                    'response_block: {
                        ::core::option::Option::Some(#getter)
                    }
                }
            } else {
                getter
            }
        }

        fn map_option_to_result(
            token_stream: proc_macro2::TokenStream,
        ) -> proc_macro2::TokenStream {
            quote! {
                match #token_stream {
                    ::core::option::Option::Some(val) => ::core::result::Result::Ok(val),
                    ::core::option::Option::None => ::core::result::Result::Err(::sqlx::Error::Decode("".into())),
                }
            }
        }

        fn traverse<'columns>(
            table_columns: &'columns [stage2::Column],
            table_name_extern: &str,
            db_tables: &[stage2::Table],
            db_name: &str,
            response_columns_names: &mut Vec<ResponseColumnName>,
            joins: &mut Vec<Join>,
        ) -> impl Iterator<Item = (&'columns Ident, proc_macro2::TokenStream)> {
            table_columns.iter().map(move |column| {
                let stage2::Column {
                    name: column_name,
                    rs_name,
                    ty,
                    ..
                } = column;
                let column_name = &**column_name;
                let (column_name_intern, column_name_extern) =
                    name_intern_extern((table_name_extern, column_name));

                let response_column_getter = match ty {
                    stage2::Ty::Element(ty_element) => {
                        let response_column_getter = response_column_getter_foreign(
                            &column_name_extern,
                            ty_element.optional(),
                        );

                        response_columns_names.push(ResponseColumnName {
                            name_intern: column_name_intern,
                            name_extern: column_name_extern,
                        });

                        response_column_getter
                    }
                    stage2::Ty::Compund(ty_compound) => {
                        let foreign_table = stage2::find_table(db_tables, &ty_compound.ty)
                            .expect("table does not exist");

                        let foreign_table_name_intern = name_intern((db_name, &foreign_table.name));
                        let foreign_table_name_extern = name_extern_triple((
                            table_name_extern,
                            &foreign_table.name,
                            column_name,
                        ));
                        let foreign_table_id_name_intern =
                            name_intern((&*foreign_table_name_extern, &foreign_table.id_name));

                        let response_columns_getters = traverse(
                            &foreign_table.columns,
                            &foreign_table_name_extern,
                            db_tables,
                            db_name,
                            response_columns_names,
                            joins,
                        );
                        let response_table_getter = response_table_getter(
                            &foreign_table.rs_name,
                            response_columns_getters,
                            ty_compound.multiplicity.optional(),
                        );

                        joins.push(Join {
                            foreign_table_name_intern,
                            foreign_table_name_extern,
                            foreign_table_id_name_intern,
                            column_name_intern,
                        });

                        response_table_getter
                    }
                };
                (rs_name, response_column_getter)
            })
        }

        let (table_name_intern, table_name_extern) = name_intern_extern((&*db.name, &*table.name));

        let mut create_columns: Vec<(&str, Cow<str>)> = vec![];
        let mut request_columns: Vec<RequestColumn> = vec![];
        let mut response_columns_names: Vec<ResponseColumnName> = vec![];
        let mut response_columns_fields: Vec<proc_macro2::TokenStream> = vec![];
        let mut response_columns_getters: Vec<(&Ident, proc_macro2::TokenStream)> = vec![];
        let mut joins: Vec<Join> = vec![];

        for column in &table.columns {
            let stage2::Column {
                name: column_name,
                rs_name,
                ty,
                rs_ty,
                attr_response: response,
                attr_request: request,
                rs_attrs: attrs,
            } = column;
            let column_name = &**column_name;
            let attrs = &**attrs;
            let (column_name_intern, column_name_extern) =
                name_intern_extern((&*table_name_extern, column_name));

            match ty {
                stage2::Ty::Element(ty_element) => {
                    let sql_ty = ty_element.sql_ty();
                    create_columns.push((column_name, sql_ty));

                    if let stage2::TyElement::Value(_) = ty_element {
                        request_columns.push(RequestColumn {
                            field: request_column_field(rs_name, rs_ty, request, attrs),
                            setter: request_column_setter(rs_name, ty_element.optional()),
                            name: column_name,
                        });
                    }

                    response_columns_fields
                        .push(response_column_field(rs_name, rs_ty, response, attrs));
                    response_columns_getters.push((
                        rs_name,
                        response_column_getter(&column_name_extern, ty_element.optional()),
                    ));
                    response_columns_names.push(ResponseColumnName {
                        name_intern: column_name_intern,
                        name_extern: column_name_extern,
                    });
                }
                stage2::Ty::Compund(ty_compound) => {
                    // let Some(foreign_table) = stage2::find_table(&db.tables, &ty_compound.ident)
                    // else {
                    //     continue;
                    // };
                    let foreign_table = stage2::find_table(&db.tables, &ty_compound.ty)
                        .expect("table does not exist");

                    let sql_ty = ty_compound.sql_ty(&foreign_table.name, &foreign_table.id_name);
                    create_columns.push((column_name, Cow::Owned(sql_ty)));

                    request_columns.push(RequestColumn {
                        field: request_column_field(
                            rs_name,
                            &rs_ty_foreign_id(ty_compound.multiplicity.optional()),
                            request,
                            attrs,
                        ),
                        setter: request_column_setter(rs_name, ty_compound.multiplicity.optional()),
                        name: column_name,
                    });

                    let foreign_table_name_intern = name_intern((&db.name, &foreign_table.name));
                    let foreign_table_name_extern =
                        name_extern_triple((&table_name_extern, &foreign_table.name, column_name));
                    let foreign_table_id_name_intern =
                        name_intern((&*foreign_table_name_extern, &foreign_table.id_name));

                    let response_columns_getter = traverse(
                        &foreign_table.columns,
                        &foreign_table_name_extern,
                        &db.tables,
                        &db.name,
                        &mut response_columns_names,
                        &mut joins,
                    );
                    let response_table_getter = response_table_getter(
                        &foreign_table.rs_name,
                        response_columns_getter,
                        ty_compound.multiplicity.optional(),
                    );
                    response_columns_getters.push((rs_name, response_table_getter));

                    joins.push(Join {
                        foreign_table_name_intern,
                        foreign_table_name_extern,
                        foreign_table_id_name_intern,
                        column_name_intern,
                    });

                    response_columns_fields
                        .push(response_column_field(rs_name, rs_ty, response, attrs));
                }
            }
        }

        let response_table_getter =
            response_table_getter(&table.rs_name, response_columns_getters, true);
        let response_table_getter = map_option_to_result(response_table_getter);

        let id_name_intern = name_intern((&*table_name_extern, &*table.id_name));

        let migration_up = fmt2::fmt! { { str } =>
            "CREATE TABLE IF NOT EXISTS " {table_name_intern} " ("
                @..join(create_columns => "," => |c| {c.0} " " {c.1})
            ");"
        };
        let migration_down = fmt2::fmt! { { str } =>
            "DROP TABLE " {table_name_intern} ";"
        };

        let get_all = fmt2::fmt! { { str } =>
            "SELECT "
            @..join(response_columns_names => "," => |c|
                {c.name_intern}
                " AS "
                // "`"
                {c.name_extern}
                // "`"
            )
            " FROM " {table_name_intern} " AS " {table_name_extern}
            @..(joins.iter().rev() => |join|
                " LEFT JOIN "
                {join.foreign_table_name_intern} " AS " {join.foreign_table_name_extern}
                " ON "
                {join.column_name_intern} "=" {join.foreign_table_id_name_intern}
            )
        };
        let get_one = fmt2::fmt! { { str } =>
            {get_all} " WHERE " {id_name_intern} "=?"
        };

        let create_one = fmt2::fmt! { { str } =>
            "INSERT INTO " {table_name_intern} " ("
                @..join(&request_columns => "," => |c| {c.name})
            ") VALUES ("
                @..join(&request_columns => "," => |_c| "?")
            ")"
        };
        let update_one = fmt2::fmt! { { str } =>
            "UPDATE " {table_name_intern} " SET "
            @..join(&request_columns => "," => |c| {c.name} "=?")
            " WHERE " {table.id_name} "=?"
        };
        let delete_one = fmt2::fmt! { { str } =>
            "DELETE FROM " {table_name_intern}
            " WHERE " {table.id_name} "=?"
        };

        let doc = fmt2::fmt! { { str } => "`" {table_name_intern} "`"};

        let table_rs_name = &table.rs_name;
        let table_rs_attrs = &*table.rs_attrs;
        let db_rs_name = &db.rs_name;
        let request_columns_fields = request_columns.iter().map(|c| &c.field);
        let request_columns_setters = request_columns.iter().map(|c| &c.setter);
        let request_columns_setter = quote! { #(#request_columns_setters,)* };

        let controller_token_stream = if table.controller {
            quote! {
                impl ::laraxum::Controller for #table_rs_name {
                    type State = #db_rs_name;
                }
            }
        } else {
            quote! {}
        };
        let table_rs_name_request = quote::format_ident!("{}Request", table.rs_name);
        let table_token_stream = quote! {
            #[doc = #doc]
            #[derive(::serde::Serialize)]
            #(#table_rs_attrs)*
            pub struct #table_rs_name {
                #(#response_columns_fields),*
            }

            #[derive(::serde::Deserialize)]
            pub struct #table_rs_name_request {
                #(#request_columns_fields),*
            }

            impl ::laraxum::Db<#table_rs_name> for #db_rs_name {}

            impl ::laraxum::Table for #table_rs_name {
                type Db = #db_rs_name;
                type Response = #table_rs_name;
                type Request = #table_rs_name_request;
                type RequestError = ();
                type RequestQuery = ();
            }

            impl ::laraxum::Model for #table_rs_name {
                /// `get_all`
                ///
                /// ```sql
                #[doc = #get_all]
                /// ```
                async fn get_all(db: &Self::Db)
                    -> ::core::result::Result::<::std::vec::Vec<Self::Response>, ::laraxum::Error>
                {
                    let response = ::sqlx::query!(#get_all)
                        .try_map(|response| #response_table_getter)
                        .fetch_all(&db.pool)
                        .await?;
                    ::core::result::Result::Ok(response)
                }
                /// `get_one`
                ///
                /// ```sql
                #[doc = #get_one]
                /// ```
                async fn get_one(db: &Self::Db, id: ::laraxum::Id)
                    -> ::core::result::Result::<Self::Response, ::laraxum::Error>
                {
                    let response = ::sqlx::query!(#get_one, id)
                        .try_map(|response| #response_table_getter)
                        .fetch_one(&db.pool)
                        .await?;
                    ::core::result::Result::Ok(response)
                }
                /// `create_one`
                ///
                /// ```sql
                #[doc = #create_one]
                /// ```
                async fn create_one(db: &Self::Db, request: Self::Request)
                    -> ::core::result::Result::<::laraxum::Id, ::laraxum::Error>
                {
                    let response = ::sqlx::query!(
                            #create_one,
                            #request_columns_setter
                        )
                        .execute(&db.pool)
                        .await?;
                    ::core::result::Result::Ok(response.last_insert_id())
                }
                /// `update_one`
                ///
                /// ```sql
                #[doc = #update_one]
                /// ```
                async fn update_one(
                    db: &Self::Db,
                    request: Self::Request,
                    id: ::laraxum::Id,
                )
                    -> ::core::result::Result::<(), ::laraxum::Error>
                {
                    ::sqlx::query!(
                        #update_one,
                        #request_columns_setter
                        id,
                    )
                        .execute(&db.pool)
                        .await?;
                    ::core::result::Result::Ok(())
                }
                /// `delete_one`
                ///
                /// ```sql
                #[doc = #delete_one]
                /// ```
                async fn delete_one(db: &Self::Db, id: ::laraxum::Id)
                    -> ::core::result::Result::<(), ::laraxum::Error>
                {
                    ::sqlx::query!(#delete_one, id)
                        .execute(&db.pool)
                        .await?;
                    ::core::result::Result::Ok(())
                }
            }

            #controller_token_stream
        };

        Self {
            token_stream: table_token_stream,
            migration_up,
            migration_down,
        }
    }
}

pub use proc_macro2::TokenStream as Db;

impl From<stage2::Db> for Db {
    fn from(db: stage2::Db) -> Self {
        let tables = db
            .tables
            .iter()
            .map(|table| Table::new(table, &db))
            .collect::<Vec<_>>();

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
        }
    }
}
