use super::stage2;

use quote::quote;
use syn::{Ident, Type};

use std::borrow::Cow;

impl stage2::StringScalarTy {
    fn sql_ty(self) -> Cow<'static, str> {
        #[cfg(feature = "mysql")]
        {
            match self {
                Self::Varchar(len) => Cow::Owned(fmt2::fmt! { { str } => "VARCHAR(" {len} ")" }),
                Self::Char(len) => Cow::Owned(fmt2::fmt! { { str } => "CHAR(" {len} ")" }),
                Self::Text => Cow::Borrowed("TEXT"),
            }
        }
    }
}

struct TimeScalarSqlTy {
    ty: &'static str,
    current_time: &'static str,
}

impl stage2::TimeScalarTy {
    fn sql_ty(self) -> &'static str {
        #[cfg(feature = "mysql")]
        {
            match self {
                Self::TimeDateTime => "DATETIME",
                Self::TimeOffsetDateTime => "TIMESTAMP",
                Self::TimeDate => "DATE",
                Self::TimeTime => "TIME",
                Self::TimeDuration => "TIME",

                Self::ChronoDateTimeUtc => "TIMESTAMP",
                Self::ChronoDateTimeLocal => "TIMESTAMP",
                Self::ChronoNaiveDateTime => "DATETIME",
                Self::ChronoNaiveDate => "DATE",
                Self::ChronoNaiveTime => "TIME",
                Self::ChronoTimeDelta => "TIME",
            }
        }
    }

    fn current_time_function(self) -> &'static str {
        #[cfg(feature = "mysql")]
        {
            match self {
                Self::TimeDateTime => "DATETIME",
                Self::TimeOffsetDateTime => "TIMESTAMP",
                Self::TimeDate => "DATE",
                Self::TimeTime => "TIME",
                Self::TimeDuration => "TIME",

                Self::ChronoDateTimeUtc => "TIMESTAMP",
                Self::ChronoDateTimeLocal => "TIMESTAMP",
                Self::ChronoNaiveDateTime => "DATETIME",
                Self::ChronoNaiveDate => "DATE",
                Self::ChronoNaiveTime => "TIME",
                Self::ChronoTimeDelta => "TIME",
            }
        }
    }
}

impl stage2::ScalarTy {
    const SQL_TY_ID: &str = {
        #[cfg(feature = "mysql")]
        {
            "BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT"
        }
    };

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
        if self.optional {
            Cow::Owned(fmt2::fmt! { { str } => {sql_ty} " NOT NULL" })
        } else {
            sql_ty
        }
    }
}

struct Table {
    token_stream: proc_macro2::TokenStream,
    migration_up: String,
    migration_down: String,
}

impl Table {
    fn from_table_and_db(table: &stage2::Table, db: &stage2::Db) -> Self {
        fn column_alias_sql((table_alias_sql, column_name_sql): (&str, &str)) -> Ident {
            quote::format_ident!("{table_alias_sql}__{column_name_sql}")
        }

        fn request_column_rs(request_ident: &Ident, rs_ty: &Type) -> proc_macro2::TokenStream {
            quote! {
                pub #request_ident: #rs_ty
            }
        }
        fn request_column_setter(request_ident: &Ident) -> proc_macro2::TokenStream {
            quote! {
                request.#request_ident
            }
        }

        fn response_column_rs(response_ident: &Ident, rs_ty: &Type) -> proc_macro2::TokenStream {
            quote! {
                pub #response_ident: #rs_ty
            }
        }
        fn response_column_getter(
            response_ident: &Ident,
            column_alias_sql: &Ident,
        ) -> proc_macro2::TokenStream {
            quote! {
                #response_ident: response.#column_alias_sql
            }
        }
        fn response_table_getter(
            table_ident: &Ident,
            columns: &[proc_macro2::TokenStream],
        ) -> proc_macro2::TokenStream {
            quote! {
                #table_ident {
                    #(#columns),*
                }
            }
        }

        let table_request_struct_ident = quote::format_ident!("{}Request", table.ident);

        let table_ident_sql = fmt2::fmt! { { str } => {db.name} "." {table.name} };
        let table_alias_sql = fmt2::fmt! { { str } => "__" {db.name} "__" {table.name} };

        let mut create_columns: Vec<(&str, Cow<str>)> = vec![];

        let mut request_columns_rs: Vec<proc_macro2::TokenStream> = vec![];
        let mut request_columns_setter: Vec<proc_macro2::TokenStream> = vec![];
        let mut request_columns_sql_create: Vec<(&str, &'static str)> = vec![];
        let mut request_columns_sql_update: Vec<(&str, &'static str)> = vec![];

        let mut response_columns_rs: Vec<proc_macro2::TokenStream> = vec![];
        let mut response_columns_getter: Vec<proc_macro2::TokenStream> = vec![];
        let mut response_columns_sql: Vec<(&str, &str)> = vec![];

        let mut joins: Vec<String> = vec![];

        for column in &table.columns {
            let stage2::Column {
                response_ident,
                request_ident,
                sql_name,
                ty: stage2::ColumnTy { virtual_ty, rs_ty },
            } = column;
            let sql_name = &**sql_name;
            let column_ident_sql = (&*table_alias_sql, sql_name);
            let column_alias_sql = column_alias_sql(column_ident_sql);

            match virtual_ty {
                stage2::VirtualTy::Real(real_ty) => {
                    let sql_ty = real_ty.sql_ty();
                    create_columns.push((sql_name, sql_ty));

                    request_columns_rs.push(request_column_rs(request_ident, rs_ty));
                    request_columns_setter.push(request_column_setter(request_ident));
                    let request_column_sql = (sql_name, "?");
                    request_columns_sql_create.push(request_column_sql);
                    request_columns_sql_update.push(request_column_sql);

                    response_columns_rs.push(response_column_rs(response_ident, rs_ty));
                    response_columns_getter
                        .push(response_column_getter(response_ident, &column_alias_sql));
                    response_columns_sql.push(column_ident_sql);
                }
                stage2::VirtualTy::Id => {
                    let sql_ty = stage2::ScalarTy::SQL_TY_ID;
                    create_columns.push((sql_name, sql_ty.into()));

                    response_columns_rs.push(response_column_rs(response_ident, rs_ty));
                    response_columns_getter
                        .push(response_column_getter(response_ident, &column_alias_sql));
                    response_columns_sql.push(column_ident_sql);
                }
                stage2::VirtualTy::OnCreate(time_ty) => {}
                stage2::VirtualTy::OnUpdate(time_ty) => {}
                stage2::VirtualTy::Foreign(table_ty) => {
                    // let foreign_table = tables
                    //     .iter()
                    //     .find(|&ft| &ft.self_type == foreign_table_type)
                    //     .expect("table does not exist");
                    // let foreign_table_name = &*foreign_table.name;
                    // let foreign_table_id_name = &foreign_table.id_name;
                    // if column_nullable {
                    //     column_responses.push(quote! {
                    //         #column_response_name: ::core::option::Option<#foreign_table_type>
                    //     });
                    //     column_requests.push(quote! {
                    //         #column_name: ::core::option::Option<::laraxum::Id>
                    //     });
                    //     sql_create.push(fmt2::fmt! { { str } =>
                    //         {column_response_name;std} " BIGINT FOREIGN KEY REFERENCES " {foreign_table_name;std} "(" {foreign_table_id_name;std} ")"
                    //     });
                    // } else {
                    //     column_responses.push(quote! {
                    //         #column_response_name: #foreign_table_type
                    //     });
                    //     column_requests.push(quote! {
                    //         #column_name: ::laraxum::Id
                    //     });
                    //     sql_create.push(fmt2::fmt! { { str } =>
                    //         {column_response_name;std} " BIGINT NOT NULL FOREIGN KEY REFERENCES " {foreign_table_name;std} "(" {foreign_table_id_name;std} ")"
                    //     });
                    // }
                    // for foreign_column in &foreign_table.columns {
                    //     sql_response_columns.push(fmt2::fmt! { { str } =>
                    //         {foreign_table_name;std}"."{foreign_column.name;std}
                    //         " as __"{self.name;std}"__"{foreign_table_name;std}"__"{foreign_column.name;std}
                    //     });
                    //     sql_joins.push(fmt2::fmt! { { str } =>
                    //         "LEFT JOIN " {foreign_table_name}
                    //     });
                    // }
                }
            }
        }

        let migration_up = fmt2::fmt! { { str } =>
            "CREATE TABLE IF NOT EXISTS " {table_ident_sql} " ("
                @..join(create_columns => "," => |c| {c.0} " " {c.1})
            ");"
        };
        let migration_down = fmt2::fmt! { { str } =>
            "DROP TABLE " {table_ident_sql} ";"
        };

        let get_all = fmt2::fmt! { { str } =>
            "SELECT "
            @..join(response_columns_sql => "," => |c| {c.0} "." {c.1} " AS " {c.0} "__" {c.1})
            " FROM " {table_ident_sql} " AS " {table_alias_sql}
            @..(joins => |join| " " {join})
        };
        let get_one = fmt2::fmt! { { str } =>
            {get_all} " WHERE " {table_alias_sql} "." {table.id_ident;std} "=?"
        };

        let create_one = fmt2::fmt! { { str } =>
            "INSERT INTO " {db.name} "." {table.name} " ("
                @..join(&request_columns_sql_create => "," => |c| {c.0})
            ") VALUES ("
                @..join(&request_columns_sql_create => "," => |c| {c.1})
            ")"
        };
        let update_one = fmt2::fmt! { { str } =>
            "UPDATE " {table_ident_sql} " SET "
            @..join(&request_columns_sql_update => "," => |c| {c.0} "=" {c.1})
            " WHERE " {table.id_ident;std} "=?"
        };
        let delete_one = fmt2::fmt! { { str } =>
            "DELETE FROM " {table_ident_sql}
            " WHERE " {table.id_ident;std} "=?"
        };

        let response_table_getter = response_table_getter(&table.ident, &response_columns_getter);

        let doc = fmt2::fmt! { { str } => "`" {db.name} "." {table.name} "`"};

        let table_ident = &table.ident;
        let db_ident = &db.ident;

        let controller_ts = if table.auto_impl_controller {
            quote! {
                impl ::laraxum::Controller for #table_ident {
                    type State = #db_ident;
                }
            }
        } else {
            quote! {}
        };
        let table_token_stream = quote! {
            #[doc = #doc]
            #[derive(::serde::Serialize)]
            pub struct #table_ident {
                #(#response_columns_rs),*
            }

            #[derive(::serde::Deserialize)]
            pub struct #table_request_struct_ident {
                #(#request_columns_rs),*
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
                        .map(|response| #response_table_getter)
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
                        .map(|response| #response_table_getter)
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
                        #(#request_columns_setter,)*
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
                ) -> ::core::result::Result::<(), ::laraxum::Error> {
                    ::sqlx::query!(
                        #update_one,
                        #(#request_columns_setter,)*
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
