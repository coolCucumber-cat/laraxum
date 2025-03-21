mod db;
mod router;
mod utils;

use darling::FromMeta;
use db::{Db, DbArgs};
use router::Router;
use utils::helper_attribute_macro;

#[proc_macro_attribute]
pub fn db(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = match darling::ast::NestedMeta::parse_meta_list(attr.into()) {
        Ok(args) => args,
        Err(err) => return proc_macro::TokenStream::from(darling::Error::from(err).write_errors()),
    };
    let args = match DbArgs::from_list(&args) {
        Ok(args) => args,
        Err(err) => return proc_macro::TokenStream::from(err.write_errors()),
    };

    let item_mod: syn::ItemMod = syn::parse_macro_input!(input);

    let db = match Db::new(item_mod, args) {
        Ok(db) => db,
        Err(err) => return proc_macro::TokenStream::from(err.to_compile_error()),
    };
    proc_macro::TokenStream::from(proc_macro2::TokenStream::from(db))
}

#[proc_macro_attribute]
pub fn id(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    helper_attribute_macro!(db => id => ::syn::Item => input)
}

#[proc_macro_attribute]
pub fn foreign(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    helper_attribute_macro!(db => foreign => ::syn::Item => input)
}

#[proc_macro_attribute]
pub fn on_update(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    helper_attribute_macro!(db => on_update => ::syn::Item => input)
}

#[proc_macro_attribute]
pub fn on_create(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    helper_attribute_macro!(db => on_create => ::syn::Item => input)
}

#[proc_macro]
pub fn router(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let router: Router = syn::parse_macro_input!(input);
    proc_macro::TokenStream::from(proc_macro2::TokenStream::from(router))
}
