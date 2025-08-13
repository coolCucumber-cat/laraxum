use super::stage2;

pub use proc_macro2::TokenStream as Router;
use quote::quote;
impl From<&stage2::Router<'_>> for Router {
    fn from(router: &stage2::Router) -> Self {
        // routes are added in reverse because of ownership rules so we reverse it to get the original order again
        router.routes.iter().rev().fold(
            quote! {
                ::axum::routing::Router::new()
            },
            |token_stream, route| {
                let stage2::Route {
                    method_router,
                    path,
                } = route;
                match method_router {
                    stage2::MethodRouter::MethodRoutes(method_routes) => {
                        let method = method_routes.iter().map(|mr| &mr.method);
                        let f = method_routes.iter().map(|mr| &mr.f);
                        quote! {
                            #token_stream
                                .route(
                                    #path,
                                    ::axum::routing::MethodRouter::new()
                                        #(.#method(#f))*,
                                )
                        }
                    }
                    stage2::MethodRouter::Controller(ty) => {
                        let path_id = fmt2::fmt! { { str } => {path} "/{id}" };
                        quote! {
                            #token_stream
                                .route(
                                    #path,
                                    ::axum::routing::MethodRouter::new()
                                        .get(<#ty as ::laraxum::Controller>::index)
                                        .post(<#ty as ::laraxum::Controller>::create),
                                )
                                .route(
                                    #path_id,
                                    ::axum::routing::MethodRouter::new()
                                        .get(<#ty as ::laraxum::Controller>::get)
                                        .patch(<#ty as ::laraxum::Controller>::update)
                                        .delete(<#ty as ::laraxum::Controller>::delete),
                                )

                        }
                    }
                }
            },
        )
    }
}
