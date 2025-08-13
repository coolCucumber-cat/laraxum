use std::borrow::Cow;

use super::stage1;

pub use stage1::MethodRouter;

pub struct Route<'a> {
    pub path: Cow<'a, str>,
    pub method_router: &'a MethodRouter,
}

pub struct Router<'a> {
    pub routes: Vec<Route<'a>>,
}

impl<'a> From<&'a stage1::Router> for Router<'a> {
    fn from(router: &'a stage1::Router) -> Self {
        let mut flat_routes = vec![];
        flatten_routes(router, None, &mut flat_routes);
        Self {
            routes: flat_routes,
        }
    }
}

fn flatten_routes<'a>(
    router: &'a stage1::Router,
    parent_path: Option<Cow<'a, str>>,
    flat_routes: &mut Vec<Route<'a>>,
) {
    let stage1::Router {
        method_router,
        routes,
    } = router;
    for route in routes.iter().rev() {
        let stage1::Route { path, router } = route;
        let path: Cow<'a, str> = if let Some(parent_path) = parent_path.as_deref() {
            Cow::Owned(fmt2::fmt! { { str } => {parent_path} {path} })
        } else {
            Cow::Borrowed(&**path)
        };
        flatten_routes(router, Some(path), flat_routes);
    }
    if let Some(method_router) = method_router {
        let path = parent_path.unwrap_or(Cow::Borrowed(""));
        flat_routes.push(Route {
            path,
            method_router,
        });
    }
}
