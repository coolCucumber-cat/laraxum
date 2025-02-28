mod db;
mod router;
mod utils;

use db::Db;
use router::Router;

use proc_macro::TokenStream;

#[proc_macro]
pub fn db(input: TokenStream) -> TokenStream {
    let db = syn::parse_macro_input!(input as Db);
    TokenStream::from(db)
}

#[proc_macro]
pub fn router(input: TokenStream) -> TokenStream {
    let router = syn::parse_macro_input!(input as Router);
    TokenStream::from(router)
}
