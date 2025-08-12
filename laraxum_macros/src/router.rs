use crate::utils::syn::parse_curly_brackets;

use quote::quote;
use syn::{
    Expr, Ident, LitStr, Token, TypePath,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

struct MethodRoute {
    method: Ident,
    f: Expr,
}

impl Parse for MethodRoute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let method = input.parse::<Ident>()?;
        let f = if input.parse::<Token![:]>().is_ok() {
            input.parse::<Expr>()?
        } else {
            syn::parse_quote!(#method)
            // Expr::Path(syn::ExprPath {
            //     path: Path::from(method.clone()),
            //     attrs: vec![],
            //     qself: None,
            // })
        };
        Ok(Self { method, f })
    }
}

enum MethodRouter {
    MethodRouter(Vec<MethodRoute>),
    Controller(TypePath),
}

impl Parse for MethodRouter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Ok(ty) = input.parse::<TypePath>() {
            Ok(Self::Controller(ty))
        } else {
            let content = parse_curly_brackets(input)?;
            let method_router = Punctuated::<MethodRoute, Token![,]>::parse_terminated(&content)?;
            let method_router = method_router.into_iter().collect();
            Ok(Self::MethodRouter(method_router))
        }
    }
}

enum NestedRouteAttr {
    Nested(String, NestedRouteAttrs),
    Route { method_router: MethodRouter },
    FnCall { fn_call: Expr },
}

impl Parse for NestedRouteAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Ok(path) = input.parse::<LitStr>() {
            let path = path.value();
            let nested_route_attrs = parse_curly_brackets(input)?;
            let nested_route_attrs = nested_route_attrs.parse::<NestedRouteAttrs>()?;
            Ok(Self::Nested(path, nested_route_attrs))
        } else if input.parse::<Token![use]>().is_ok() {
            let method_router = input.parse::<MethodRouter>()?;
            // input.parse::<Token![;]>()?;
            Ok(Self::Route { method_router })
        } else {
            let fn_call = input.parse::<Expr>()?;
            Ok(Self::FnCall { fn_call })
        }
    }
}

struct NestedRouteAttrs {
    nested_route_attrs: Vec<NestedRouteAttr>,
}

impl Parse for NestedRouteAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let nested_route_attrs = Punctuated::<NestedRouteAttr, Token![,]>::parse_terminated(input)?;
        let nested_route_attrs = nested_route_attrs.into_iter().collect();
        Ok(Self { nested_route_attrs })
    }
}

enum RouteAttr {
    Route {
        path: String,
        method_router: MethodRouter,
    },
    FnCall {
        fn_call: Expr,
    },
}

struct RouteAttrs {
    route_attrs: Vec<RouteAttr>,
}

impl From<NestedRouteAttrs> for RouteAttrs {
    fn from(nested_route_attrs: NestedRouteAttrs) -> Self {
        let mut route_attrs = vec![];
        unnest_route_attrs(nested_route_attrs, &mut route_attrs, None);
        Self { route_attrs }
    }
}

impl Parse for RouteAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let nested_route_attrs = input.parse::<NestedRouteAttrs>()?;
        Ok(Self::from(nested_route_attrs))
    }
}

fn unnest_route_attrs(
    nested_route_attrs: NestedRouteAttrs,
    route_attrs: &mut Vec<RouteAttr>,
    path: Option<&str>,
) {
    for nested_route_attr in nested_route_attrs.nested_route_attrs {
        match nested_route_attr {
            NestedRouteAttr::Route { method_router } => {
                let path = path.unwrap_or("/").into();
                route_attrs.push(RouteAttr::Route {
                    path,
                    method_router,
                });
            }
            NestedRouteAttr::FnCall { fn_call } => {
                route_attrs.push(RouteAttr::FnCall { fn_call });
            }
            NestedRouteAttr::Nested(path2, nested_route_attrs) => {
                let path3 = match path {
                    Some(path) => fmt2::fmt! { { str } => {path} {path2}},
                    None => path2,
                };
                unnest_route_attrs(nested_route_attrs, route_attrs, Some(&path3));
            }
        }
    }
}

pub struct Router {
    route_attrs: RouteAttrs,
}

impl Parse for Router {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let route_attrs = input.parse::<RouteAttrs>()?;
        Ok(Self { route_attrs })
    }
}

impl From<Router> for proc_macro2::TokenStream {
    fn from(router: Router) -> Self {
        let mut router_expr = quote! { ::axum::routing::Router::new() };
        for route_attr in &router.route_attrs.route_attrs {
            match route_attr {
                RouteAttr::FnCall { fn_call } => {
                    router_expr = quote! { #router_expr.#fn_call };
                }
                RouteAttr::Route {
                    path,
                    method_router: MethodRouter::Controller(ty),
                } => {
                    let path_id = fmt2::fmt! { { str } => {path} "/{id}" };
                    router_expr = quote! {
                        #router_expr
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

                    };
                }
                RouteAttr::Route {
                    path,
                    method_router: MethodRouter::MethodRouter(method_routes),
                } => {
                    let method = method_routes.iter().map(|mr| &mr.method);
                    let f = method_routes.iter().map(|mr| &mr.f);
                    router_expr = quote! {
                        #router_expr
                            .route(
                                #path,
                                ::axum::routing::MethodRouter::new()
                                    #(.#method(#f))*,
                            )
                    };
                }
            }
        }
        router_expr
    }
}
