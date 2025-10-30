/// Serve the app.
///
/// Serve the app at address in `URL` environment variable, defaults to "localhost:80".
///
/// See [router](crate::router) for examples.
///
/// # Returns
/// `Future<Output = Result<(), std::io::Error>>`
#[cfg_attr(not(feature = "macros"), docs(hidden))]
#[macro_export]
macro_rules! serve {
    ($app:expr) => {
        async {
            let url = $crate::controller::url();
            let url = &*url;
            let app_listener = ::tokio::net::TcpListener::bind(url).await?;
            ::std::println!("Listening at: {url:?}");
            ::axum::serve(app_listener, $app).await?;
            ::core::result::Result::Ok(())
        }
    };
}
