use crate::utils::syn::from_str_to_rs_ident;

use super::stage2;

use quote::quote;
use syn::{Ident, Type};

use std::borrow::Cow;

fn alias((parent, child): (&str, &str)) -> String {
    fmt2::fmt! { { str } => {parent} "__" {child} }
}
fn alias_rs_ident(ident: (&str, &str)) -> Ident {
    from_str_to_rs_ident(&*alias(ident))
}
fn ident((parent, child): (&str, &str)) -> String {
    fmt2::fmt! { { str } => {parent} "." {child} }
}
fn ident_and_alias(parent_child: (&str, &str)) -> (String, String) {
    (ident(parent_child), alias(parent_child))
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

impl stage2::StringScalarTy {
    fn sql_ty(self) -> Cow<'static, str> {
        #[cfg(feature = "mysql")]
        match self {
            Self::Varchar(len) => Cow::Owned(fmt2::fmt! { { str } => "VARCHAR(" {len} ")" }),
            Self::Char(len) => Cow::Owned(fmt2::fmt! { { str } => "CHAR(" {len} ")" }),
            Self::Text => Cow::Borrowed("TEXT"),
        }
    }
}

impl stage2::TimeScalarTy {
    fn sql_ty(self) -> &'static str {
        TimeScalarTy::from(self).sql_ty
    }
}

struct TimeScalarTy {
    sql_ty: &'static str,
    sql_current_time_func: &'static str,
}
impl From<stage2::TimeScalarTy> for TimeScalarTy {
    fn from(stage2_time_scalar_ty: stage2::TimeScalarTy) -> Self {
        #[cfg(feature = "mysql")]
        match stage2_time_scalar_ty {
            stage2::TimeScalarTy::ChronoDateTimeUtc => Self {
                sql_ty: "TIMESTAMP",
                sql_current_time_func: "UTC_TIMESTAMP()",
            },
            stage2::TimeScalarTy::ChronoDateTimeLocal
            | stage2::TimeScalarTy::TimeOffsetDateTime => Self {
                sql_ty: "TIMESTAMP",
                sql_current_time_func: "CURRENT_TIMESTAMP()",
            },
            stage2::TimeScalarTy::ChronoNaiveDateTime
            | stage2::TimeScalarTy::TimePrimitiveDateTime => Self {
                sql_ty: "DATETIME",
                sql_current_time_func: "CURRENT_TIMESTAMP()",
            },
            stage2::TimeScalarTy::ChronoNaiveDate | stage2::TimeScalarTy::TimeDate => Self {
                sql_ty: "DATE",
                sql_current_time_func: "CURRENT_DATE()",
            },
            stage2::TimeScalarTy::ChronoNaiveTime
            | stage2::TimeScalarTy::TimeTime
            | stage2::TimeScalarTy::ChronoTimeDelta
            | stage2::TimeScalarTy::TimeDuration => Self {
                sql_ty: "TIME",
                sql_current_time_func: "CURRENT_TIME()",
            },
        }
    }
}

impl stage2::ScalarTy {
    fn sql_ty(self) -> Cow<'static, str> {
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

impl stage2::RealTy {
    fn sql_ty(self) -> Cow<'static, str> {
        let sql_ty = self.ty.sql_ty();
        make_maybe_optional(sql_ty, self.optional)
    }
}

impl stage2::AutoTimeTy {
    fn sql_ty(self) -> String {
        let time_scalar_ty = TimeScalarTy::from(self.ty);
        let mut auto_time_ty = make_not_optional(time_scalar_ty.sql_ty);
        fmt2::fmt! { (auto_time_ty) =>
            " DEFAULT " {time_scalar_ty.sql_current_time_func}
        }
        if matches!(self.event, stage2::AutoTimeEvent::OnUpdate) {
            fmt2::fmt! { (auto_time_ty) =>
                " ON UPDATE " {time_scalar_ty.sql_current_time_func}
            }
        }
        auto_time_ty
    }
}

const SQL_TY_ID: &str = {
    #[cfg(feature = "mysql")]
    {
        "BIGINT UNSIGNED NOT NULL UNIQUE PRIMARY KEY AUTO_INCREMENT"
    }
};

impl stage2::ForeignTy {
    fn sql_ty(&self, sql_column_alias: &str, id_ident: &str) -> String {
        #[cfg(feature = "mysql")]
        {
            let sql_ty = make_maybe_optional(Cow::Borrowed("BIGINT UNSIGNED"), self.optional);
            let mut sql_ty = sql_ty.into_owned();
            fmt2::fmt! { (sql_ty) => " FOREIGN KEY REFERENCES " {sql_column_alias} "(" {id_ident} ")" }
            sql_ty
        }
    }
}

struct RequestColumn<'sql_name> {
    field: proc_macro2::TokenStream,
    rs_setter: proc_macro2::TokenStream,
    sql_setter: &'sql_name str,
}

struct ResponseColumn<'ident> {
    field: proc_macro2::TokenStream,
    rs_getter: (&'ident Ident, proc_macro2::TokenStream),
}

struct ExpandedResponseColumn {
    sql_getter_ident: String,
    sql_getter_alias: String,
}

struct Table {
    token_stream: proc_macro2::TokenStream,
    migration_up: String,
    migration_down: String,
}

impl Table {
    fn from_table_and_db(table: &stage2::Table, db: &stage2::Db) -> Self {
        fn request_field(request_ident: &Ident, rs_ty: &Type) -> proc_macro2::TokenStream {
            quote! {
                pub #request_ident: #rs_ty
            }
        }
        fn request_rs_setter(request_ident: &Ident) -> proc_macro2::TokenStream {
            quote! { request.#request_ident }
        }

        fn response_field(response_ident: &Ident, rs_ty: &Type) -> proc_macro2::TokenStream {
            quote! {
                pub #response_ident: #rs_ty
            }
        }
        fn response_rs_getter(column_alias: &str) -> proc_macro2::TokenStream {
            let column_alias = from_str_to_rs_ident(column_alias);
            quote! { response.#column_alias }
        }
        fn response_table_rs_getter<'columns, 'ident: 'columns>(
            table_ident: &Ident,
            columns: impl Iterator<Item = &'columns (&'ident Ident, proc_macro2::TokenStream)>,
        ) -> proc_macro2::TokenStream {
            let columns = columns.map(|(ident, rs_getter)| {
                quote! {
                    #ident: #rs_getter
                }
            });
            quote! { #table_ident { #( #columns ),* } }
        }

        fn traverse_columns<'columns>(
            table_columns: &'columns [stage2::Column],
            table_alias: &str,
            db_tables: &[stage2::Table],
            expanded_response_columns: &mut Vec<ExpandedResponseColumn>,
        ) -> impl Iterator<Item = (&'columns Ident, proc_macro2::TokenStream)> {
            table_columns.iter().map(|column| {
                let stage2::Column {
                    response_ident,
                    request_ident: _,
                    sql_name,
                    ty:
                        stage2::ColumnTy {
                            virtual_ty,
                            rs_ty: _,
                        },
                } = column;
                let sql_name = &**sql_name;
                let (sql_column_ident, column_alias) = ident_and_alias((table_alias, sql_name));
                let response_rs_getter = match virtual_ty {
                    stage2::VirtualTy::Inner(_virtual_ty_inner) => {
                        let response_rs_getter = response_rs_getter(&column_alias);
                        expanded_response_columns.push(ExpandedResponseColumn {
                            sql_getter_ident: sql_column_ident,
                            sql_getter_alias: column_alias,
                        });
                        response_rs_getter
                    }
                    stage2::VirtualTy::Foreign(foreign_ty) => {
                        let foreign_table = stage2::find_table(db_tables, &foreign_ty.ident)
                            .expect("table does not exist");
                        let foreign_table_alias = alias((table_alias, &foreign_table.name));
                        let rs_getter_columns = traverse_columns(
                            &foreign_table.columns,
                            &foreign_table_alias,
                            db_tables,
                            expanded_response_columns,
                        );
                        // let rs_getter_columns=rs_getter_columns;

                        response_table_rs_getter(&foreign_table.ident, &rs_getter_columns)
                    }
                };
                (response_ident, response_rs_getter)
            })
        }

        let (sql_table_ident, table_alias) = ident_and_alias((&*db.name, &*table.name));

        let mut create_columns: Vec<(&str, Cow<str>)> = vec![];
        let mut request_columns: Vec<RequestColumn> = vec![];
        let mut response_columns: Vec<ResponseColumn> = vec![];
        let mut expanded_response_columns: Vec<ExpandedResponseColumn> = vec![];
        let mut joins: Vec<String> = vec![];

        for column in &table.columns {
            let stage2::Column {
                response_ident,
                request_ident,
                sql_name,
                ty: stage2::ColumnTy { virtual_ty, rs_ty },
            } = column;
            let sql_name = &**sql_name;

            match virtual_ty {
                stage2::VirtualTy::Inner(virtual_ty_inner) => {
                    match virtual_ty_inner {
                        stage2::VirtualTyInner::Real(real_ty) => {
                            create_columns.push((sql_name, real_ty.sql_ty()));

                            request_columns.push(RequestColumn {
                                field: request_field(request_ident, rs_ty),
                                rs_setter: request_rs_setter(request_ident),
                                sql_setter: sql_name,
                            });
                        }
                        stage2::VirtualTyInner::Id => {
                            create_columns.push((sql_name, Cow::Borrowed(SQL_TY_ID)));
                        }
                        stage2::VirtualTyInner::AutoTime(auto_time_ty) => {
                            create_columns.push((sql_name, Cow::Owned(auto_time_ty.sql_ty())));
                        }
                    }

                    let (sql_column_ident, column_alias) =
                        ident_and_alias((&*table_alias, sql_name));
                    response_columns.push(ResponseColumn {
                        field: response_field(response_ident, rs_ty),
                        rs_getter: (response_ident, response_rs_getter(&*column_alias)),
                    });
                    expanded_response_columns.push(ExpandedResponseColumn {
                        sql_getter_ident: sql_column_ident,
                        sql_getter_alias: column_alias,
                    });
                }
                stage2::VirtualTy::Foreign(foreign_ty) => {
                    let foreign_table = stage2::find_table(&db.tables, &foreign_ty.ident)
                        .expect("table does not exist");

                    let sql_ty = foreign_ty.sql_ty(&*foreign_table.name, &*foreign_table.id_ident);
                    create_columns.push((sql_name, Cow::Owned(sql_ty)));

                    request_columns.push(RequestColumn {
                        field: request_field(request_ident, &rs_ty_foreign_id(foreign_ty.optional)),
                        rs_setter: request_rs_setter(request_ident),
                        sql_setter: sql_name,
                    });

                    let rs_getter_columns = traverse_columns(
                        &foreign_table.columns,
                        &table_alias,
                        &db.tables,
                        &mut expanded_response_columns,
                    );
                    let rs_getter =
                        response_table_rs_getter(&foreign_table.ident, rs_getter_columns);
                    response_columns.push(ResponseColumn {
                        field: response_field(response_ident, rs_ty),
                        rs_getter: (response_ident, rs_getter),
                    });
                }
            }
        }

        let response_columns_rs_getters = response_columns.iter().map(|column| &column.rs_getter);
        let response_table_rs_getter =
            response_table_rs_getter(&table.ident, response_columns_rs_getters);

        let sql_table_id_ident = ident((&*table_alias, &*table.id_ident));

        let migration_up = fmt2::fmt! { { str } =>
            "CREATE TABLE IF NOT EXISTS " {sql_table_ident} " ("
                @..join(create_columns => "," => |c| {c.0} " " {c.1})
            ");"
        };
        let migration_down = fmt2::fmt! { { str } =>
            "DROP TABLE " {sql_table_ident} ";"
        };

        let get_all = fmt2::fmt! { { str } =>
            "SELECT "
            @..join(expanded_response_columns => "," => |c| {c.sql_getter_ident} " AS " {c.sql_getter_alias})
            " FROM " {sql_table_ident} " AS " {table_alias}
            @..(joins => |join| " " {join})
        };
        let get_one = fmt2::fmt! { { str } =>
            {get_all} " WHERE " {sql_table_id_ident} "=?"
        };

        let create_one = fmt2::fmt! { { str } =>
            "INSERT INTO " {sql_table_ident} " ("
                @..join(&request_columns => "," => |c| {c.sql_setter})
            ") VALUES ("
                @..join(&request_columns => "," => |_c| "?")
            ")"
        };
        let update_one = fmt2::fmt! { { str } =>
            "UPDATE " {sql_table_ident} " SET "
            @..join(&request_columns => "," => |c| {c.sql_setter} "=?")
            " WHERE " {table.id_ident} "=?"
        };
        let delete_one = fmt2::fmt! { { str } =>
            "DELETE FROM " {sql_table_ident}
            " WHERE " {table.id_ident} "=?"
        };

        let doc = fmt2::fmt! { { str } => "`" {sql_table_ident} "`"};

        let table_ident = &table.ident;
        let db_ident = &db.ident;
        let response_columns_fields = response_columns.iter().map(|c| &c.field);
        let request_columns_fields = request_columns.iter().map(|c| &c.field);
        let request_columns_setters = request_columns.iter().map(|c| &c.rs_setter);

        let controller_ts = if table.auto_impl_controller {
            quote! {
                impl ::laraxum::Controller for #table_ident {
                    type State = #db_ident;
                }
            }
        } else {
            quote! {}
        };
        let table_request_struct_ident = quote::format_ident!("{}Request", table.ident);
        let table_token_stream = quote! {
            #[doc = #doc]
            #[derive(::serde::Serialize)]
            pub struct #table_ident {
                #(#response_columns_fields),*
            }

            #[derive(::serde::Deserialize)]
            pub struct #table_request_struct_ident {
                #(#request_columns_fields),*
            }

            impl ::laraxum::Db<#table_ident> for #db_ident {}

            impl ::laraxum::Table for #table_ident {
                type Db = #db_ident;
                type Response = #table_ident;
                type Request = #table_request_struct_ident;
                type RequestError = ();
                type RequestQuery = ();
            }

            impl ::laraxum::Model for #table_ident {
                /// `get_all`
                ///
                /// ```sql
                #[doc = #get_all]
                /// ```
                async fn get_all(db: &Self::Db)
                    -> ::core::result::Result::<::std::vec::Vec<Self::Response>, ::laraxum::Error>
                {
                    let response = ::sqlx::query!(#get_all)
                        .map(|response| #response_table_rs_getter)
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
                        .map(|response| #response_table_rs_getter)
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
                        #(#request_columns_setters,)*
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
                        #(#request_columns_setters,)*
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

            #controller_ts
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
            .map(|table| Table::from_table_and_db(table, &db))
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

        let db_ident = &db.ident;
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
