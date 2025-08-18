use crate::utils::syn::parse_curly_brackets;

use syn::{
    Expr, Ident, LitStr, Token, TypePath,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

pub struct MethodRoute {
    pub method: Ident,
    pub f: Expr,
}
impl Parse for MethodRoute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let method = input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let f = input.parse::<Expr>()?;
        Ok(Self { method, f })
    }
}

pub enum MethodRouter {
    MethodRoutes(Vec<MethodRoute>),
    Controller(TypePath),
}
impl Parse for MethodRouter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![use]>()?;
        let method_router = if let Ok(ty) = input.parse::<TypePath>() {
            Self::Controller(ty)
        } else {
            let content = parse_curly_brackets(input)?;
            let method_routes = Punctuated::<MethodRoute, Token![,]>::parse_terminated(&content)?;
            let method_routes = method_routes.into_iter().collect();
            Self::MethodRoutes(method_routes)
        };
        input.parse::<Token![;]>()?;
        Ok(method_router)
    }
}

pub struct Route {
    pub path: String,
    pub router: Router,
}
impl Parse for Route {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path = input.parse::<LitStr>()?;
        let path = path.value();
        let input = parse_curly_brackets(input)?;
        let router = input.parse::<Router>()?;
        Ok(Self { path, router })
    }
}

pub struct Router {
    pub method_router: Option<MethodRouter>,
    pub routes: Vec<Route>,
}
impl Parse for Router {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        #[expect(clippy::if_then_some_else_none)]
        let method_router = if input.peek(Token![use]) {
            Some(input.parse::<MethodRouter>()?)
        } else {
            None
        };
        let routes = Punctuated::<Route, Token![,]>::parse_terminated(input)?;
        let routes = routes.into_iter().collect();
        Ok(Self {
            method_router,
            routes,
        })
    }
}
