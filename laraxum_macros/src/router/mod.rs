mod stage1;
mod stage2;
mod stage3;

/// Create a router.
///
/// The router has methods and routes. A route is a path and a router, which makes it nested and recursive. If the router has methods, they are created at the start in a `use` statement. You can either create each method route with curly brackets like a struct expression where the field name is the router method, or you can give the controller, which will create all the method routes and nested method routes for that controller.
///
///  verb | noun
/// ------|----------------
///       |tokenstream
/// stage1|
///       |syntax data
/// ------|----------------
/// stage2|
///       |abstract data
/// ------|----------------
/// stage3|
///       |tokenstream
/// ------|----------------
pub fn router(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    // stage 1: frontend -> syntax + processing
    let router = syn::parse2::<stage1::Router>(input)?;

    // stage 2: backend -> processing
    let router = stage2::Router::from(&router);
    // stage 3: backend -> syntax
    let router = stage3::Router::from(&router);

    Ok(router)
}
