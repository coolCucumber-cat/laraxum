mod db;
mod router;
mod utils;

#[proc_macro]
pub fn router(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    router::router(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
pub fn db(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    db::db(attr.into(), input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
