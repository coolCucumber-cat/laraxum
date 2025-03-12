mod kw {
    syn::custom_keyword! { Option }
    syn::custom_keyword! { Id }
}

use crate::utils::parse_curly_brackets;

use proc_macro2::Span;
use quote::quote;
use syn::{parse::Parse, punctuated::Punctuated, spanned::Spanned, Ident, LitStr, Token, Type};

const TABLE_MUST_HAVE_ID: &str = "table must have an ID";
const UNKNOWN_TYPE: &str = "unknown type";

macro_rules! ty_enum {
    {
        enum $enum:ident {
            $(
                $(#[$meta:meta])*
                $ident:ident($ty:ty) => $rs_ty:ty => $sql_ty:expr
            ),* $(,)?
        }
    } => {
        #[allow(non_camel_case_types)]
        #[derive(Copy, Clone, PartialEq, Eq)]
        enum $enum {
            $(
                $(#[$meta])*
                #[doc = $sql_ty]
                $ident,
            )*
        }

        impl $enum {
            fn sql_ty_not_null(self) -> &'static str {
                match self {
                    $(
                        $(#[$meta])*
                        Self::$ident => $sql_ty,
                    )*
                }
            }
            fn sql_ty_null(self) -> &'static str {
                match self {
                    $(
                        $(#[$meta])*
                        Self::$ident => ::core::concat!($sql_ty, " NOT NULL"),
                    )*
                }
            }

            fn rs_ty_not_null(self, span: ::proc_macro2::Span) -> ::proc_macro2::TokenStream {
                match self {
                    $(
                        $(#[$meta])*
                        Self::$ident => ::quote::quote_spanned! { span => $rs_ty },
                    )*
                }
            }
            fn rs_ty_null(self, span: ::proc_macro2::Span, nullability: kw::Option) -> ::proc_macro2::TokenStream {
                let ident = self.rs_ty_not_null(span);
                ::quote::quote! { ::core::option::#nullability<#ident> }
            }


            fn parse_ty(input: ::syn::Type) -> ::core::option::Option<Self> {
                $(
                    $(#[$meta])*
                    {
                        let ty: ::syn::Type = ::syn::parse_quote! { $ty };
                        if ty == input {
                            return ::core::option::Option::Some(Self::$ident);
                        }
                    }
                )*
                ::core::option::Option::None
            }
        }
    };
}

#[cfg(feature = "mysql")]
ty_enum! {
    enum ColumnTyPrimitiveInner {
        Id(Id) => u64 => "BIGINT PRIMARY KEY AUTO_INCREMENT",
        String(String) => ::std::string::String => "VARCHAR(255)",
        StringText(String<Text>) => ::std::string::String => "TEXT",
        bool(bool) => bool => "BOOL",
        u8(u8) => u8 => "TINYINT UNSIGNED",
        i8(i8) => i8 => "TINYINT",
        u16(u16) => u16 => "SMALLINT UNSIGNED",
        i16(i16) => i16 => "SMALLINT",
        u32(u32) => u32 => "INT UNSIGNED",
        i32(i32) => i32 => "INT",
        u64(u64) => u64 => "BIGINT UNSIGNED",
        i64(i64) => i64 => "BIGINT",
        f32(f32) => f32 => "FLOAT",
        f64(f64) => f64 => "DOUBLE",
    }
}

#[cfg(feature = "postgres")]
ty_enum! {
    enum ColumnTyPrimitiveInner {
        Id(Id) => u64 => "SERIAL PRIMARY KEY",
        String(String) => ::std::string::String => "VARCHAR(255)",
        bool(bool) => bool => "BOOL",
        i8(i8) => i8 => "CHAR",  // TINYINT
        i16(i16) => i16 => "INT2", // SMALLINT
        i32(i32) => i32 => "INT4", // INT
        i64(i64) => i64 => "INT8", // BIGINT
        f32(f32) => f32 => "FLOAT4", // FLOAT
        f64(f64) => f64 => "FLOAT8", // DOUBLE
    }
}

#[cfg(feature = "sqlite")]
ty_enum! {
    enum ColumnTyPrimitiveInner {
        Id(Id) => u64 => "INTEGER PRIMARY KEY AUTOINCREMENT",
        String(String) => ::std::string::String => "TEXT",
        bool(bool) => bool => "BOOLEAN",
        u8(u8) => u8 => "INTEGER",
        i8(i8) => i8 => "INTEGER",
        u16(u16) => u16 => "INTEGER",
        i16(i16) => i16 => "INTEGER",
        u32(u32) => u32 => "INTEGER",
        i32(i32) => i32 => "INTEGER",
        u64(u64) => u64 => "INTEGER",
        i64(i64) => i64 => "BIGINT",
        f32(f32) => f32 => "FLOAT",
        f64(f64) => f64 => "DOUBLE",
    }
}

#[derive(Clone, Copy)]
struct ColumnTyPrimitive {
    ty: ColumnTyPrimitiveInner,
    span: Span,
}

impl ColumnTyPrimitive {
    fn sql_ty(self, nullability: Option<kw::Option>) -> &'static str {
        if nullability.is_some() {
            self.ty.sql_ty_null()
        } else {
            self.ty.sql_ty_not_null()
        }
    }

    fn rs_ty(self, nullability: Option<kw::Option>) -> proc_macro2::TokenStream {
        match nullability {
            Some(nullability) => self.ty.rs_ty_null(self.span, nullability),
            None => self.ty.rs_ty_not_null(self.span),
        }
    }

    fn is_id(self) -> bool {
        matches!(self.ty, ColumnTyPrimitiveInner::Id)
    }
}

impl Parse for ColumnTyPrimitive {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ty_token = input.parse::<Type>()?;
        let span = ty_token.span();
        match ColumnTyPrimitiveInner::parse_ty(ty_token) {
            Some(ty) => Ok(Self { ty, span }),
            None => Err(syn::Error::new(span, UNKNOWN_TYPE)),
        }
    }
}

#[derive(Clone, Copy)]
enum ColumnTyInner {
    Primitive(ColumnTyPrimitive),
    Foreign(Ident),
    // Primary(kw::Id),
}

impl Parse for ColumnTyInner {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // if let Ok(kw_id) = input.parse::<kw::Id>() {
        // Ok(Self::Primary(kw_id))
        // } else
        if input.parse::<Token![&]>().is_ok() {
            let ty = input.parse::<Ident>()?;
            Ok(Self::Foreign(ty))
        } else {
            let ty = input.parse::<ColumnTyPrimitive>()?;
            Ok(Self::Primitive(ty))
        }
    }
}

#[derive(Clone, Copy)]
struct ColumnTy {
    ty_inner: ColumnTyInner,
    nullability: Option<kw::Option>,
}

impl Parse for ColumnTy {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let nullability = input.parse::<kw::Option>().ok();
        let ty_inner = if nullability.is_some() {
            input.parse::<Token![<]>()?;
            let ty_inner = input.parse::<ColumnTyInner>()?;
            input.parse::<Token![>]>()?;
            ty_inner
        } else {
            input.parse::<ColumnTyInner>()?
        };

        match (ty_inner.clone(), nullability) {
            (ColumnTyInner::Primitive(ty), Some(kw::Option { span })) if ty.is_id() => {
                Err(syn::Error::new(span, "ID must not be nullable"))
            }
            _ => Ok(Self {
                ty_inner,
                nullability,
            }),
        }
        // match (ty_inner.clone(), nullability) {
        //     (ColumnTyInner::Primary(_), Some(kw::Option { span })) => {
        //         Err(syn::Error::new(span, "ID must not be nullable"))
        //     }
        //     _ => Ok(Self {
        //         ty_inner,
        //         nullability,
        //     }),
        // }
    }
}

#[derive(Clone)]
struct Column {
    /// the name for the column
    response_name: Ident,
    /// the name for the column in the request
    name: Ident,
    /// the type the column has
    ty: ColumnTy,
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
        let ty = input.parse::<ColumnTy>()?;
        Ok(Self {
            response_name,
            name,
            ty,
        })
    }
}

struct RequestColumn {
    name: Ident,
    ty: proc_macro2::TokenStream,
}

impl RequestColumn {
    fn request_setter(request_columns: &[Self]) -> proc_macro2::TokenStream {
        let names = request_columns.iter().map(|rc| &rc.name);
        quote! { #(request.#names,)* }
    }
}

struct ResponseColumn {
    name: Ident,
    ty: proc_macro2::TokenStream,
    from_expanded_response: proc_macro2::TokenStream,
}

impl ResponseColumn {
    fn response_getter() {}
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
    fn to_response_column(&self, ty: proc_macro2::TokenStream) -> ResponseColumn {
        let name = self.name();
        ResponseColumn {
            name: self.inner_name.clone(),
            ty,
            from_expanded_response: quote! { response.#name },
        }
    }
}

struct Table {
    /// the name for the table struct, for example `Customer`
    ty: Ident,
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
        let ty = input.parse::<Ident>()?;
        let name = if input.parse::<Token![as]>().is_ok() {
            input.parse::<LitStr>()?.value()
        } else {
            ty.to_string()
        };
        let content = parse_curly_brackets(input)?;
        let columns_iter = Punctuated::<Column, Token![,]>::parse_terminated(&content)?;
        let mut id_name = None;
        let mut columns = vec![];
        for column in columns_iter {
            if matches!(&column.ty.ty_inner, ColumnTyInner::Primary(_)) {
                match id_name {
                    Some(_) => {
                        return Err(syn::Error::new(
                            column.name.span(),
                            "table cannot have multiple IDs",
                        ))
                    }
                    None => id_name = Some(column.name.clone()),
                }
            }
            // if matches!(&column.ty.ty_inner, ColumnTyInner::Primary(_)) {
            //     match id_name {
            //         Some(_) => {
            //             return Err(syn::Error::new(
            //                 column.name.span(),
            //                 "table cannot have multiple IDs",
            //             ))
            //         }
            //         None => id_name = Some(column.name.clone()),
            //     }
            // }
            columns.push(column);
        }
        let Some(id_name) = id_name else {
            return Err(syn::Error::new(ty.span(), TABLE_MUST_HAVE_ID));
        };
        Ok(Self {
            ty,
            name,
            columns,
            auto_impl_controller,
            id_name,
        })
    }
}

impl Table {
    fn transform_table(&self, db: &Db) -> (proc_macro2::TokenStream, String, String) {
        let table_request_ty = quote::format_ident!("{}Request", self.ty);
        let query_table_name = fmt2::fmt! { { str } => "__" {db.name} "__" {self.name} };

        let mut request_columns = vec![];
        let mut expanded_response_columns = vec![];
        let mut response_columns = vec![];
        let mut create_columns = vec![];
        let joins = vec![
            fmt2::fmt! { { str } => "FROM " {db.name} "." {self.name} " AS " {query_table_name}},
        ];

        for column in &self.columns {
            match &column.ty.ty_inner {
                ColumnTyInner::Primitive(column_ty) => {
                    let rs_ty = column_ty.clone().rs_ty(column.ty.nullability);
                    let sql_ty = column_ty.clone().sql_ty(column.ty.nullability);

                    let create_column = fmt2::fmt! { { str } =>
                        {column.name;std} " " {sql_ty}
                    };
                    create_columns.push(create_column);

                    let request_column = RequestColumn {
                        name: column.name.clone(),
                        ty: rs_ty.clone(),
                    };
                    request_columns.push(request_column);

                    let expanded_response_column = ExpandedReponseColumn {
                        inner_name: column.response_name.clone(),
                        table_name: query_table_name.clone(),
                    };
                    let response_column = expanded_response_column.to_response_column(rs_ty);
                    expanded_response_columns.push(expanded_response_column);
                    response_columns.push(response_column);
                }
                // ColumnTyInner::Primary(kw_id) => {}
                ColumnTyInner::Foreign(foreign_table_ty) => {
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
            "CREATE TABLE IF NOT EXISTS " {db.name} "." {self.name} " ("
                @..join(create_columns => "," => |c| {c})
            ");"
        };
        let migration_down = fmt2::fmt! { { str } =>
            "DROP TABLE " {db.name} "." {self.name} ";"
        };

        let get_all = fmt2::fmt! { { str } =>
            "SELECT "
            @..join(expanded_response_columns => "," => |c| {c.get_query()})
            @..(joins => |join| " " {join})
        };
        let get_one = fmt2::fmt! { { str } =>
            {get_all} " WHERE " {query_table_name} "." {self.id_name;std} " = ?"
        };
        let create_one = fmt2::fmt! { { str } =>
            "INSERT INTO " {db.name} "." {self.name} " ("
                @..join(request_columns.iter() => "," => |c| {c.name;std})
            ") VALUES ("
                @..join(request_columns.iter() => "," => |_c| "?")
            ")"
        };
        let update_one = fmt2::fmt! { { str } =>
            "UPDATE " {db.name} "." {self.name} " SET "
            @..join(request_columns.iter() => "," => |c| {c.name;std} "=?")
            " WHERE " {self.id_name;std} " = ?"
        };
        let delete_one = fmt2::fmt! { { str } =>
            "DELETE FROM " {db.name} "." {self.name}
            " WHERE " {self.id_name;std} " = ?"
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

        let request_setter = RequestColumn::request_setter(&request_columns);
        let table_ty = &self.ty;
        let db_ty = &db.self_ty;

        let controller_ts = if self.auto_impl_controller {
            quote! {
                impl ::laraxum::Controller for #table_ty {
                    type State = #db_ty;
                }
            }
        } else {
            quote! {}
        };

        let doc = fmt2::fmt! { { str } => "`" {db.name} "." {self.name} "`"};

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
            }

            impl ::laraxum::Model for #table_ty {
                type RequestError = ();
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
                /// `create_one`
                ///
                /// ```sql
                #[doc = #create_one]
                /// ```
                async fn create_one(db: &Self::Db, request: Self::Request) -> ::core::result::Result<::laraxum::Id, ::sqlx::Error> {
                    ::core::result::Result::map(
                        ::sqlx::query!(
                            #create_one,
                            #request_setter
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
                    request: Self::Request,
                    id: ::laraxum::Id,
                ) -> ::core::result::Result<(), ::sqlx::Error> {
                    ::core::result::Result::map(
                        ::sqlx::query!(
                            #update_one,
                            #request_setter
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
    }
}

pub struct Db {
    /// the name for the database struct, for example `AppDb`
    self_ty: Ident,
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
            self_ty: self_type,
            name,
            pool_type,
            tables,
        })
    }
}

impl From<Db> for proc_macro::TokenStream {
    fn from(db: Db) -> Self {
        let tables = db
            .tables
            .iter()
            .map(|table| table.transform_table(&db))
            .collect::<Vec<_>>();

        let migration_up = fmt2::fmt! { { str } =>
            "BEGIN TRANSACTION;"
            @..(tables.iter() => |table| {table.1})
            "COMMIT;"
        };
        let migration_down = fmt2::fmt! { { str } =>
            "BEGIN TRANSACTION;"
            @..(tables.iter().rev() => |table| {table.2})
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

        let tables_ts = tables.iter().map(|table| &table.0);

        let db_type = &db.self_ty;
        let db_pool_type = db.pool_type;

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
