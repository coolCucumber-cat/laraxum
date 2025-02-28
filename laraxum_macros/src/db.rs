use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, punctuated::Punctuated, Ident, LitStr, Token, Type};

use crate::utils::{
    maybe_optional, maybe_reference, try_from_path_to_ident, try_from_type_to_sql_type,
};

enum ColumnType {
    NotForeign(Type),
    Foreign(Ident),
}

struct Column {
    /// the name for the column
    response_name: Ident,
    /// the name for the column in the request
    name: Ident,
    /// the type the column has
    self_type: ColumnType,
    /// is nullable
    nullable: bool,
}

impl Parse for Column {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        let response_name = if input.parse::<Token![|]>().is_ok() {
            input.parse::<Ident>()?
        } else {
            name.clone()
        };
        input.parse::<Token![:]>()?;
        let self_type = input.parse::<Type>()?;
        let (self_type, foreign) = maybe_reference(self_type);
        let (self_type, nullable) = maybe_optional(&self_type);
        let self_type = if foreign {
            let Type::Path(path) = self_type else {
                panic!("invalid type for column");
            };
            let Some(ident) = try_from_path_to_ident(path) else {
                panic!("invalid type for column");
            };
            ColumnType::Foreign(ident.clone())
        } else {
            ColumnType::NotForeign(self_type.clone())
        };
        Ok(Self {
            response_name,
            name,
            self_type,
            nullable,
        })
    }
}

struct Table {
    /// the name for the table struct, for example `Customer`
    self_type: Ident,
    /// the name for the sql table, for example `customers`
    name: String,
    /// the name for the id of the table, for example `CustomerId`
    id_name: Ident,
    /// automatically implement the controller as well as the model, using the db as the state
    auto_impl_controller: bool,
    /// the columns in the database
    columns: Vec<Column>,
}

impl Parse for Table {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let auto_impl_controller = input.parse::<Token![auto]>().is_ok();
        let self_type = input.parse::<Ident>()?;
        let name = if input.parse::<Token![as]>().is_ok() {
            input.parse::<LitStr>()?.value()
        } else {
            self_type.to_string()
        };
        let id_name = quote::format_ident!("id");
        let content;
        syn::braced!(content in input);
        let columns = Punctuated::<Column, Token![,]>::parse_terminated(&content)?;
        let columns = columns.into_iter().collect();
        Ok(Self {
            self_type,
            name,
            columns,
            auto_impl_controller,
            id_name,
        })
    }
}

pub struct Db {
    /// the name for the database struct, for example `AppDb`
    self_type: Ident,
    /// the name of the database
    name: String,
    /// the type for the sql pool, for example `sqlx::MySqlPool`
    pool_type: Type,
    /// the tables in the database
    tables: Vec<Table>,
}

impl Parse for Db {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let self_type = input.parse::<Ident>()?;
        let name = if input.parse::<Token![as]>().is_ok() {
            input.parse::<LitStr>()?.value()
        } else {
            self_type.to_string()
        };
        input.parse::<Token![:]>()?;
        let pool_type = input.parse::<Type>()?;
        let content;
        syn::braced!(content in input);
        let tables = Punctuated::<Table, Token![,]>::parse_terminated(&content)?;
        let tables = tables.into_iter().collect();
        Ok(Self {
            self_type,
            name,
            pool_type,
            tables,
        })
    }
}

impl From<Db> for TokenStream {
    fn from(db: Db) -> Self {
        let Db {
            self_type: db_type,
            name: db_name,
            pool_type: db_pool_type,
            tables,
        } = db;

        let transform_table = |table: &Table| {
            let Table {
                self_type: table_type,
                name: table_name,
                columns,
                id_name: table_id_name,
                auto_impl_controller: table_auto_impl_controller,
            } = table;
            let table_auto_impl_controller = *table_auto_impl_controller;
            let table_request_type = quote::format_ident!("{table_type}Request");

            let column_request_names = columns.iter().map(|column| &column.name);
            let column_request_names_1 = column_request_names.clone();
            let column_request_names_2 = column_request_names.clone();
            let column_response_names = columns.iter().map(|column| &column.response_name);

            let mut column_types = vec![];

            let mut sql_create = vec![];
            let mut sql_response_columns = vec![];
            // let mut sql_joins = vec![];

            for column in columns {
                let column_response_name = &column.response_name;
                let column_name = &column.name;
                let column_nullable = column.nullable;

                match &column.self_type {
                    ColumnType::Foreign(foreign_table_type) => {
                        unimplemented!();
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
                        //         " as __"{table_name;std}"__"{foreign_table_name;std}"__"{foreign_column.name;std}
                        //     });
                        //     sql_joins.push(fmt2::fmt! { { str } =>
                        //         "LEFT JOIN " {foreign_table_name}
                        //     });
                        // }

                        // sql_response_columns.push(fmt2::fmt! { { str } =>
                        //     {table_name;std}"."{column_response_name;std} " as " "_"{table_name;std}"_"{column_response_name;std}"_"
                        // });
                    }
                    ColumnType::NotForeign(column_type) => {
                        let sql_column_type =
                            try_from_type_to_sql_type(column_type).expect("invalid type");
                        if column_nullable {
                            let column_type = quote! { ::core::option::Option<#column_type> };
                            column_types.push((column_type.clone(), column_type));

                            sql_create.push(fmt2::fmt! { { str } =>
                                {column_name;std} " " {sql_column_type}
                            });
                        } else {
                            let column_type = quote! { #column_type };
                            column_types.push((column_type.clone(), column_type));

                            sql_create.push(fmt2::fmt! { { str } =>
                                {column_name;std} " " {sql_column_type} " NOT NULL"
                            });
                        }
                        sql_response_columns.push(fmt2::fmt! { { str } =>
                            "__" {table_name} "." {column_name;std} " as __" {table_name} "__" {column_name;std} 
                        });
                    }
                }
            }

            let column_request_types = column_types.iter().map(|column| &column.0);
            let column_response_types = column_types.iter().map(|column| &column.1);

            let migration_up = fmt2::fmt! { { str } =>
                "CREATE TABLE IF NOT EXISTS " {db_name} "." {table_name} " ("
                    {table_id_name;std} " BIGINT PRIMARY KEY AUTO_INCREMENT"
                    @..(sql_create => |c| "," {c})
                ");"
            };
            let migration_down = fmt2::fmt! { { str } =>
                "DROP TABLE " {db_name} "." {table_name} ";" ln
            };

            let get_all = fmt2::fmt! { { str } =>
                "SELECT __"
                {table_name} "." {table_id_name;std}
                @..(column_response_names.clone() => |c| ", __" {table_name} "." {c;std})
                " FROM " {db_name} "." {table_name} " __" {table_name}
            };
            let get_one = fmt2::fmt! { { str } =>
                {get_all} " WHERE __" {table_name} "." {table_id_name;std} " = ?"
            };
            let create_one = fmt2::fmt! { { str } =>
                "INSERT INTO " {db_name} "." {table_name} " ("
                    @..join(column_request_names.clone() => ", " => |c| {c;std})
                ") VALUES ("
                    @..join(column_request_names.clone() => ", " => |_c| "?")
                ")"
            };
            let update_one = fmt2::fmt! { { str } =>
                "UPDATE " {db_name} "." {table_name} " SET "
                @..join(column_request_names.clone() => ", " => |c| {c;std} " = ?")
                " WHERE " {table_id_name;std} " = ?"
            };
            let delete_one = fmt2::fmt! { { str } =>
                "DELETE FROM " {db_name} "." {table_name}
                " WHERE " {table_id_name;std} " = ?"
            };

            let controller_ts = if table_auto_impl_controller {
                quote! {
                    impl ::laraxum::Controller for #table_type {
                        type State = #db_type;
                    }
                }
            } else {
                quote! {}
            };

            let table_ts = quote! {
                #[doc = "`"]
                #[doc = #db_name]
                #[doc = "."]
                #[doc = #table_name]
                #[doc = "`"]
                #[derive(::serde::Serialize)]
                pub struct #table_type {
                    pub #table_id_name: ::laraxum::Id,
                    pub #(#column_response_names: #column_response_types),*
                }

                #[derive(::serde::Deserialize)]
                pub struct #table_request_type {
                    pub #(#column_request_names: #column_request_types),*
                }

                impl ::laraxum::Db<#table_type> for #db_type {}

                impl ::laraxum::Table for #table_type {
                    type Db = #db_type;
                    type Response = #table_type;
                    type Request = #table_request_type;
                }

                impl ::laraxum::Model for #table_type {
                    type RequestQuery = ();

                    /// `get_all`
                    ///
                    /// ```sql
                    #[doc = #get_all]
                    /// ```
                    async fn get_all(db: &Self::Db) -> ::core::result::Result<::std::vec::Vec<Self::Response>, ::sqlx::Error> {
                        ::sqlx::query_as!(Self::Response, #get_all).fetch_all(&db.pool).await
                    }
                    /// `get_one`
                    ///
                    /// ```sql
                    #[doc = #get_one]
                    /// ```
                    async fn get_one(db: &Self::Db, id: ::laraxum::Id) -> ::core::result::Result<::core::option::Option<Self::Response>, ::sqlx::Error> {
                        ::sqlx::query_as!(Self::Response, #get_one, id).fetch_optional(&db.pool).await
                    }
                    /// `get_one_exact`
                    ///
                    /// ```sql
                    #[doc = #get_one]
                    /// ```
                    async fn get_one_exact(db: &Self::Db, id: ::laraxum::Id) -> ::core::result::Result<Self::Response, ::sqlx::Error> {
                        ::sqlx::query_as!(Self::Response, #get_one, id).fetch_one(&db.pool).await
                    }
                    /// `create_one`
                    ///
                    /// ```sql
                    #[doc = #create_one]
                    /// ```
                    async fn create_one(db: &Self::Db, r: Self::Request) -> ::core::result::Result<::laraxum::Id, ::sqlx::Error> {
                        ::core::result::Result::map(
                            ::sqlx::query!(
                                #create_one,
                                #(r.#column_request_names_1,)*
                            )
                            .execute(&db.pool)
                            .await,
                            |r| r.last_insert_id(),
                        )
                    }
                    /// `update_one`
                    ///
                    /// ```sql
                    #[doc = #update_one]
                    /// ```
                    async fn update_one(
                        db: &Self::Db,
                        r: Self::Request,
                        id: ::laraxum::Id,
                    ) -> ::core::result::Result<(), ::sqlx::Error> {
                        ::core::result::Result::map(
                            ::sqlx::query!(
                                #update_one,
                                #(r.#column_request_names_2,)*
                                id,
                            )
                            .execute(&db.pool)
                            .await,
                            |_| (),
                        )
                    }
                    /// `delete_one`
                    ///
                    /// ```sql
                    #[doc = #delete_one]
                    /// ```
                    async fn delete_one(db: &Self::Db, id: ::laraxum::Id) -> ::core::result::Result<(), ::sqlx::Error> {
                        ::core::result::Result::map(
                            ::sqlx::query!(#delete_one, id)
                            .execute(&db.pool)
                            .await,
                            |_| (),
                        )
                    }
                }

                #controller_ts
            };
            (table_ts, migration_up, migration_down)
        };
        let tables = tables.iter().map(transform_table).collect::<Vec<_>>();

        let migration_up = fmt2::fmt! { { str } =>
            "BEGIN TRANSACTION;"
            @..(&tables => |table| {table.1})
            "COMMIT;"
        };
        let migration_down = fmt2::fmt! { { str } =>
            "BEGIN TRANSACTION;"
            @..(tables.iter().rev() => |table| {table.2})
            "COMMIT;"
        };
        let migration_up_full = fmt2::fmt! { { str } =>
            "CREATE DATABASE IF NOT EXISTS " {db_name} ";"
            {migration_up}
        };
        let migration_down_full = fmt2::fmt! { { str } =>
            {migration_down}
            "DROP DATABASE " {db_name} ";"
        };

        let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let root = root.join("laraxum");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("migration_up.sql"), &migration_up).unwrap();
        std::fs::write(root.join("migration_down.sql"), &migration_down).unwrap();
        std::fs::write(root.join("migration_up_full.sql"), &migration_up_full).unwrap();
        std::fs::write(root.join("migration_down_full.sql"), &migration_down_full).unwrap();

        let tables_ts = tables.iter().map(|table| &table.0);
        quote! {
            #[doc = #migration_up_full]
            pub struct #db_type {
                pool: ::sqlx::Pool<#db_pool_type>,
            }

            impl ::laraxum::AnyDb for #db_type {
                type Db = Self;
                async fn connect_with_str(s: &str) -> ::core::result::Result<Self, ::sqlx::Error> {
                    ::core::result::Result::Ok(Self {
                        pool: ::sqlx::Pool::<#db_pool_type>::connect(s).await?,
                    })
                }
                fn db(&self) -> &Self::Db {
                    self
                }
            }

            #(#tables_ts)*
        }
        .into()
    }
}
