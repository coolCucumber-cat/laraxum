mod db;
mod router;
mod utils;

use db::Db;
use proc_macro::TokenStream;

#[proc_macro]
pub fn db(input: TokenStream) -> TokenStream {
    let db = syn::parse_macro_input!(input as Db);
    TokenStream::from(db)
}
