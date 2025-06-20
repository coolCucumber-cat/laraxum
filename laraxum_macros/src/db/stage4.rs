use super::stage3;

use crate::utils::{collections::TryCollectAll, syn::from_str_to_rs_ident};

use std::{borrow::Cow, vec};

use quote::quote;
use syn::{Ident, Type};

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

impl stage3::RequestColumn<'_> {
    pub fn request_setter_column(&self) -> Option<(&str, &str)> {
        match self {
            Self::Some {
                setter: stage3::RequestColumnSetter { name, .. },
                ..
            } => Some((name, "?")),
            Self::AutoTime { name, time_ty } => Some((name, time_ty.ty())),
            _ => None,
        }
    }
}

fn create_table<'columns>(
    table_name_intern: &str,
    columns: impl IntoIterator<Item = &'columns stage3::Column<'columns>>,
) -> String {
    let create_columns = columns.into_iter().flat_map(|column| &column.create);

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
    response_getter_column_elements: &[stage3::ResponseColumnGetterElement<'elements>],
    response_getter_column_compounds: &[stage3::ResponseColumnGetterCompound<'compounds>],
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
    response_getter_column_elements: &[stage3::ResponseColumnGetterElement<'elements>],
    response_getter_column_compounds: &[stage3::ResponseColumnGetterCompound<'compounds>],
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
    columns: &[stage3::Column<'request_columns>],
) -> String {
    let request_columns = columns
        .iter()
        .flat_map(|column| column.request.request_setter_column());
    fmt2::fmt! { { str } =>
        "INSERT INTO " {table_name_intern} " ("
            @..join(request_columns.clone() => "," => |column| {column.0})
        ") VALUES ("
            @..join(request_columns => "," => |column| {column.1})
        ")"
    }
}
fn update_one<'request_columns>(
    table_name_intern: &str,
    id_name: &str,
    columns: &[stage3::Column<'request_columns>],
) -> String {
    let request_columns = columns
        .iter()
        .flat_map(|column| column.request.request_setter_column());
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
        fn flatten_internal<'columns>(
            response_getter_columns: impl IntoIterator<
                Item = &'columns stage3::ResponseColumnGetter<'columns>,
            >,
            response_getter_column_elements: &mut Vec<
                &'columns stage3::ResponseColumnGetterElement<'columns>,
            >,
            response_getter_column_compounds: &mut Vec<
                &'columns stage3::ResponseColumnGetterCompound<'columns>,
            >,
        ) {
            for response_getter_column in response_getter_columns {
                match response_getter_column {
                    stage3::ResponseColumnGetter::Element(element) => {
                        response_getter_column_elements.push(element);
                    }
                    stage3::ResponseColumnGetter::Compound(compound) => {
                        response_getter_column_compounds.push(compound);
                        flatten_internal(
                            compound.columns.iter(),
                            response_getter_column_elements,
                            response_getter_column_compounds,
                        );
                    }
                    stage3::ResponseColumnGetter::Compounds(_) => {}
                }
            }
        }
        fn flatten<'columns>(
            response_getter_columns: impl Iterator<
                Item = &'columns stage3::ResponseColumnGetter<'columns>,
            >,
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

        fn response_field_access(ident: &Ident) -> proc_macro2::TokenStream {
            quote! {
                response.#ident
            }
        }

        fn response_getter_element(
            name_extern: &str,
            optional: bool,
            foreign: bool,
        ) -> proc_macro2::TokenStream {
            let name_extern = from_str_to_rs_ident(name_extern);
            let field_access = response_field_access(&name_extern);

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
                quote! {
                    ::core::option::Option::map(#field_access, ::laraxum::Decode::decode)
                }
            } else {
                quote! {
                    ::laraxum::Decode::decode(#field_access)
                }
            }
        }

        fn response_getter_compound<'columns>(
            table_ty: &Ident,
            columns: impl IntoIterator<Item = &'columns stage3::ResponseColumnGetter<'columns>>,
            optional: bool,
            foreign: bool,
        ) -> proc_macro2::TokenStream {
            let columns = columns.into_iter().map(|column| match column {
                stage3::ResponseColumnGetter::Element(element) => {
                    let stage3::ResponseColumnGetterElement {
                        name_extern,
                        optional,
                        rs_name,
                        ..
                    } = element;

					let name_extern = from_str_to_rs_ident(name_extern);
					let field_access = response_field_access(&name_extern);

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
						let getter = if *optional {
                            quote! {
								::core::option::Option::map(#field_access, ::laraxum::Decode::decode)
							}
                        } else {
                            quote! {
								::laraxum::Decode::decode(#field_access)
							}
                        };
                    quote! {
                        #rs_name: #getter
                    }
                }
                stage3::ResponseColumnGetter::Compound(compound) => {
                    let stage3::ResponseColumnGetterCompound {
                        optional,
                        rs_name,
                        rs_ty_name,
                        columns,
                        ..
                    } = compound;
                    let getter = response_getter_compound(rs_ty_name, columns.iter(), *optional, true);
                    quote! {
                        #rs_name: #getter
                    }
                }
                stage3::ResponseColumnGetter::Compounds(compounds) => {
                    let stage3::ResponseColumnGetterCompounds {
                        rs_name,
                        table_rs_name,
                        table_id_rs_name,
                        foreign_table_rs_name,
                    } = compounds;
                    // TODO: foreign key
                    quote! {
                        #rs_name: vec![]
                    }
                }
            });

            let getter = quote! {
                #table_ty { #( #columns ),* }
            };
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

        let response_column_fields = table.columns.iter().map(|column| {
            let stage3::ResponseColumnField {
                rs_name,
                rs_ty,
                attr,
                rs_attrs,
            } = column.response.field;

            let serde_skip = attr.skip.then(serde_skip);
            let serde_name = attr.name.as_deref().map(serde_name);

            quote! {
                #(#rs_attrs)* #serde_skip #serde_name
                pub #rs_name: #rs_ty
            }
        });

        let response_getter = response_getter_compound(
            &table.rs_name,
            table.columns.iter().map(|column| &column.response.getter),
            true,
            false,
        );
        let response_getter = map_option_to_result(response_getter);

        let (response_getter_column_elements, response_getter_column_compounds) =
            flatten(table.columns.iter().map(|column| &column.response.getter));

        let request_column_fields = table
            .columns
            .iter()
            .filter_map(|column| match &column.request {
                stage3::RequestColumn::Some { field, .. }
                | stage3::RequestColumn::Compounds { field, .. } => Some(field),
                _ => None,
            })
            .map(|column| {
                let &stage3::RequestColumnField {
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

        let request_column_setters = table
            .columns
            .iter()
            .filter_map(|column| match &column.request {
                stage3::RequestColumn::Some { setter, .. } => Some(setter),
                _ => None,
            })
            .map(|column| {
                let stage3::RequestColumnSetter {
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
        let request_column_setters = quote! { #(#request_column_setters,)* };

        let create_table = create_table(
            &table.name_intern,
            table.columns.iter().map(|column| &column.create),
        );
        let delete_table = delete_table(&table.name_intern);

        let get_all = get_all(
            &table.name_intern,
            &table.name_extern,
            response_getter_column_elements.iter().copied(),
            response_getter_column_compounds.iter().copied(),
        );
        let create_one = create_one(
            &table.name_intern,
            table.columns.iter().flat_map(|column| &column.request),
        );

        let table_rs_name = &table.rs_name;
        let table_rs_name_request = quote::format_ident!("{}Request", table.rs_name);
        let table_rs_attrs = &*table.rs_attrs;
        let db_rs_name = &db.rs_name;
        let doc = fmt2::fmt! { { str } => "`" {table.name_intern} "`"};
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
                            #request_column_setters
                        )
                        .execute(&db.pool)
                        .await?;
                    ::core::result::Result::Ok(())
                }
            }
        });

        let model_token_stream = table.columns.model().map(|table_id| {
            let get_one = get_one(
                &table.name_intern,
                &table.name_extern,
                &table_id.,
                &response_getter_column_elements,
                &response_getter_column_compounds,
            );
            let update_one = update_one(&table.name_intern, &table_id.name, table.columns.iter().flat_map(|column|&column.request));
            let delete_one = delete_one(&table.name_intern, &table_id.name);

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
                        let response = ::sqlx::query!(#create_one, #request_column_setters)
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
                        ::sqlx::query!(#update_one, #request_column_setters id)
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
