/// laraxum macros
///
/// todo:
/// - many foreign keys
/// - encode/decode custom types like enums
/// - enum types (maybe attr to make enum type)
/// - schema generator
/// - unique
/// - auto increment
/// - sql types for all DBs
mod db;
mod router;
mod utils;

#[proc_macro]
pub fn router(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let router: router::Router = syn::parse_macro_input!(input);
    proc_macro::TokenStream::from(proc_macro2::TokenStream::from(router))
}

#[proc_macro_attribute]
pub fn db(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    db_syn(attr.into(), input.into())
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}

fn db_syn(
    attr: proc_macro2::TokenStream,
    input: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    // stage 1: frontend -> syntax
    let stage1_db_attr = db::stage1::DbAttr::try_from(attr)?;
    let stage1_db = syn::parse2::<db::stage1::Db>(input)?;
    // stage 2: frontend -> processing
    let stage2_db = db::stage2::Db::try_new(stage1_db, stage1_db_attr)?;

    // stage 3: backend -> processing
    let stage3_db = db::stage3::Db::try_from(&stage2_db)?;
    // stage 4: backend -> syntax

    Ok(stage3_db)
}
