use std::borrow::Cow;

use quote::quote;
use syn::{Ident, Type};

use super::stage2;

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
                Self::Varchar(len) => Cow::Owned(fmt2::fmt! { { str } => "VARCHAR(" {len} ")" }),
                Self::Char(len) => Cow::Owned(fmt2::fmt! { { str } => "CHAR(" {len} ")" }),
                Self::Text => Cow::Borrowed("TEXT"),
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

                Self::TimePrimitiveDateTime => Cow::Borrowed("DATETIME"),
                Self::TimeOffsetDateTime => Cow::Borrowed("TIMESTAMP"),
                Self::TimeDate => Cow::Borrowed("DATE"),
                Self::TimeTime => Cow::Borrowed("TIME"),
                Self::TimeDuration => Cow::Borrowed("TIME"),

                Self::ChronoDateTimeUtc => Cow::Borrowed("TIMESTAMP"),
                Self::ChronoDateTimeLocal => Cow::Borrowed("TIMESTAMP"),
                Self::ChronoNaiveDateTime => Cow::Borrowed("DATETIME"),
                Self::ChronoNaiveDate => Cow::Borrowed("DATE"),
                Self::ChronoNaiveTime => Cow::Borrowed("TIME"),
                Self::ChronoTimeDelta => Cow::Borrowed("TIME"),
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

struct RequestColumn {
    name: Ident,
    ty: Type,
}

impl RequestColumn {
    fn request_setter(
        request_columns: &[Self],
    ) -> impl Iterator<Item = proc_macro2::TokenStream> + Clone {
        request_columns.iter().map(|rc| {
            let name = &rc.name;
            quote! { request.#name }
        })
    }
}

struct ResponseColumn {
    name: Ident,
    ty: Type,
    from_expanded_response: proc_macro2::TokenStream,
}

impl ResponseColumn {
    fn response_getter(response_columns: &[Self], table_name: &Ident) -> proc_macro2::TokenStream {
        let response_columns = response_columns.iter().map(|rc| {
            let name = &rc.name;
            let from_expanded_response = &rc.from_expanded_response;
            quote! { #name: #from_expanded_response }
        });
        quote! {
            #table_name {
                #(#response_columns,)*
            }
        }
    }
}

struct ExpandedReponseColumn {
    inner_name: Ident,
    table_name: String,
}

impl ExpandedReponseColumn {
    fn name(&self) -> Ident {
        quote::format_ident!("{}__{}", self.table_name, self.inner_name)
    }
    fn get_query(&self) -> String {
        fmt2::fmt! { { str } =>
            {self.table_name} "." {self.inner_name;std} " AS " {self.name();std}
        }
    }
    fn to_response_column(&self, ty: Type) -> ResponseColumn {
        let name = self.name();
        ResponseColumn {
            name: self.inner_name.clone(),
            ty,
            from_expanded_response: quote! { response.#name },
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
        let table_request_ty = quote::format_ident!("{}Request", table.ident);
        let query_table_name = fmt2::fmt! { { str } => "__" {db.name} "__" {table.name} };

        let mut request_columns: Vec<RequestColumn> = vec![];
        let mut expanded_response_columns: Vec<ExpandedReponseColumn> = vec![];
        let mut response_columns: Vec<ResponseColumn> = vec![];
        let mut create_columns: Vec<String> = vec![];
        let joins: Vec<String> = vec![
            fmt2::fmt! { { str } => "FROM " {db.name} "." {table.name} " AS " {query_table_name} },
        ];

        for column in &table.columns {
            match &column.virtual_ty {
                stage2::VirtualTy::Real(real_ty) => {
                    let sql_ty = real_ty.sql_ty();
                    create_columns.push(fmt2::fmt! { { str } =>
                        {column.request_ident;std} " " {sql_ty}
                    });

                    let expanded_response_column = ExpandedReponseColumn {
                        inner_name: column.response_ident.clone(),
                        table_name: query_table_name.clone(),
                    };
                    let response_column =
                        expanded_response_column.to_response_column(column.rs_ty.clone());
                    expanded_response_columns.push(expanded_response_column);
                    response_columns.push(response_column);

                    request_columns.push(RequestColumn {
                        name: column.request_ident.clone(),
                        ty: column.rs_ty.clone(),
                    });
                }
                stage2::VirtualTy::Id => {
                    let sql_ty = stage2::ScalarTy::SQL_TY_ID;
                    create_columns.push(fmt2::fmt! { { str } =>
                        {column.request_ident;std} " " {sql_ty}
                    });

                    let expanded_response_column = ExpandedReponseColumn {
                        inner_name: column.response_ident.clone(),
                        table_name: query_table_name.clone(),
                    };
                    let response_column =
                        expanded_response_column.to_response_column(column.rs_ty.clone());
                    expanded_response_columns.push(expanded_response_column);
                    response_columns.push(response_column);
                }
                stage2::VirtualTy::OnCreate(time_ty) => {
                    let sql_ty = time_ty.sql_ty();
                    create_columns.push(fmt2::fmt! { { str } =>
                        {column.request_ident;std} " " {sql_ty}
                    });

                    let expanded_response_column = ExpandedReponseColumn {
                        inner_name: column.response_ident.clone(),
                        table_name: query_table_name.clone(),
                    };
                    let response_column =
                        expanded_response_column.to_response_column(column.rs_ty.clone());
                    expanded_response_columns.push(expanded_response_column);
                    response_columns.push(response_column);
                }
                stage2::VirtualTy::OnUpdate(_x) => {}
                stage2::VirtualTy::Foreign(_foreign_table_ty) => {
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

                    // sql_response_columns.push(fmt2::fmt! { { str } =>
                    //     {self.name;std}"."{column_response_name;std} " as " "_"{self.name;std}"_"{column_response_name;std}"_"
                    // });
                }
            }
        }

        let migration_up = fmt2::fmt! { { str } =>
            "CREATE TABLE IF NOT EXISTS " {db.name} "." {table.name} " ("
                @..join(create_columns => "," => |c| {c})
            ");"
        };
        let migration_down = fmt2::fmt! { { str } =>
            "DROP TABLE " {db.name} "." {table.name} ";"
        };

        let get_all = fmt2::fmt! { { str } =>
            "SELECT "
            @..join(expanded_response_columns => "," => |c| {c.get_query()})
            @..(joins => |join| " " {join})
        };
        let get_one = fmt2::fmt! { { str } =>
            {get_all} " WHERE " {query_table_name} "." {table.id_ident;std} " = ?"
        };
        let create_one = fmt2::fmt! { { str } =>
            "INSERT INTO " {db.name} "." {table.name} " ("
                @..join(request_columns.iter() => "," => |c| {c.name;std})
            ") VALUES ("
                @..join(request_columns.iter() => "," => |_c| "?")
            ")"
        };
        let update_one = fmt2::fmt! { { str } =>
            "UPDATE " {db.name} "." {table.name} " SET "
            @..join(request_columns.iter() => "," => |c| {c.name;std} " = ?")
            " WHERE " {table.id_ident;std} " = ?"
        };
        let delete_one = fmt2::fmt! { { str } =>
            "DELETE FROM " {db.name} "." {table.name}
            " WHERE " {table.id_ident;std} " = ?"
        };

        let table_columns = response_columns.iter().map(|c| {
            let name = &c.name;
            let ty = &c.ty;
            quote! { #name: #ty }
        });
        let request_table_columns = request_columns.iter().map(|c| {
            let name = &c.name;
            let ty = &c.ty;
            quote! { #name: #ty }
        });

        let request_setter_create = RequestColumn::request_setter(&request_columns);
        let request_setter_update = request_setter_create.clone();
        let response_getter = ResponseColumn::response_getter(&response_columns, &table.ident);
        let table_ty = &table.ident;
        let db_ty = &db.ident;

        let controller_ts = if table.auto_impl_controller {
            quote! {
                impl ::laraxum::Controller for #table_ty {
                    type State = #db_ty;
                }
            }
        } else {
            quote! {}
        };

        let doc = fmt2::fmt! { { str } => "`" {db.name} "." {table.name} "`"};

        let table_ts = quote! {
            #[doc = #doc]
            #[derive(::serde::Serialize)]
            pub struct #table_ty {
                #(pub #table_columns),*
            }

            #[derive(::serde::Deserialize)]
            pub struct #table_request_ty {
                #(pub #request_table_columns),*
            }

            impl ::laraxum::Db<#table_ty> for #db_ty {}

            impl ::laraxum::Table for #table_ty {
                type Db = #db_ty;
                type Response = #table_ty;
                type Request = #table_request_ty;
                type RequestError = ();
                type RequestQuery = ();
            }

            impl ::laraxum::Model for #table_ty {
                /// `get_all`
                ///
                /// ```sql
                #[doc = #get_all]
                /// ```
                async fn get_all(db: &Self::Db)
                    -> ::core::result::Result::<::std::vec::Vec<Self::Response>, ::laraxum::Error>
                {
                    let response = ::sqlx::query!(#get_all)
                        .map(|response| #response_getter)
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
                        .map(|response| #response_getter)
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
                        #(#request_setter_create,)*
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
                        #(#request_setter_update,)*
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
            token_stream: table_ts,
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

        let tables_ts = tables.iter().map(|table| &table.token_stream);

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

            #(#tables_ts)*

            use ::laraxum_macros::{id, foreign, on_update, on_create};
        }
    }
}
