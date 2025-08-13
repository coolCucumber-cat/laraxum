mod stage1;
mod stage2;
mod stage3;

pub fn router(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    // stage 1: frontend -> syntax + processing
    let router = syn::parse2::<stage1::Router>(input)?;

    // stage 2: backend -> processing
    let router = stage2::Router::from(&router);
    // stage 3: backend -> syntax
    let router = stage3::Router::from(&router);

    Ok(router)
}
