Create a router.

The router has methods and routes. A route is a path and a router, which makes it nested and recursive. If the router has methods, they are created at the start in a `use` statement. You can either create each method route with curly brackets like a struct expression where the field name is the router method, or you can give the controller, which will create all the method routes and nested method routes for that controller.

# Examples

```rust
// There are tables and functions defined in the `db` module.
mod db;
...
use laraxum::{router, serve, AppError, Connect};
...
#[tokio::main]
async fn main() -> Result<(), AppError> {
    let router = router! {
        // `GET /`
        use { get: db::api_docs };

        "/api/v3" {
            // `GET /api/v3`
            use { get: db::home };

            // `POST /api/v3/login`
            "/login" { use { post: db::login }; },

            // `GET /api/v3/users`,
            // `POST /api/v3/users`,
            // `GET /api/v3/users/{id}`,
            // `PUT /api/v3/users/{id}`,
            // `PATCH /api/v3/users/{id}`,
            // `DELETE /api/v3/users/{id}`,
            "/users" { use db::User; },

            "/settings" {
                // `GET /api/v3/settings`
                use { get: db::settings_home };

                // `GET /api/v3/settings/contacts`,
                // `POST /api/v3/settings/contacts`,
                // `GET /api/v3/settings/contacts/{id}`,
                // `PUT /api/v3/settings/contacts/{id}`,
                // `PATCH /api/v3/settings/contacts/{id}`,
                // `DELETE /api/v3/settings/contacts/{id}`,
                "/contacts" { use db::Contact; },
            },
        },
    };
    // Connect to database at address in `DATABASE_URL`.
    // Returns `Ok(db::AppDb)` or `Err(sqlx::Error)`.
    let db = db::AppDb::connect().await?;
    // `Arc` to share the database connection across threads.
    let state = std::sync::Arc::new(db);
    // router + state = app.
    let app = router.with_state(state);
    // Serve app at address in `URL` environment variable, defaults to "localhost:80".
    // Returns `Ok(())` or `Err(std::io::Error)`.
    serve!(app).await
}
```
