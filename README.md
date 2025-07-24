# Laraxum

A framework built on top of Axum and SQLX to simplify creating database servers.
It is inspired by Laravel and uses the MVC paradigm:

- Model: the data of our application, meaning the database
- View: the interface to interact with the application, meaning the data we send and receive
- Controller: the connection between the view and the controller

The database is defined using the `db` macro. It creates the model, view and controller.
You don't have to use the view or controller, it's possible to create them yourself and
use that instead.

## Example

```rs
#[db(name = "database")]
pub mod AppDb {
    #[db(name = "addresses", model, controller)]
    pub struct Address {
        #[db(ty(id))]
        id: u64,
        #[db(ty(varchar = 255))]
        street: String,
        #[db(ty(char = 5))]
        postcode: String,
        #[db(ty(varchar = 255))]
        city: String,
    }
    #[db(name = "contacts", model, controller)]
    pub struct Contact {
        #[db(ty(id))]
        id: u64,
        #[db(ty(foreign()), request(name = "address_id"), name = "address_id")]
        address: Address,
        #[db(ty(varchar = 255))]
        firstname: String,
        #[db(ty(varchar = 255))]
        lastname: String,
        #[db(ty(varchar = 16))]
        mobile: String,
        #[db(ty(varchar = 16))]
        landline: String,
        #[db(ty(varchar = 255))]
        email: String,
    }
    #[db(name = "groups", model, controller)]
    pub struct Group {
        #[db(ty(id))]
        id: u64,
        #[db(ty(varchar = 255))]
        title: String,
    }
    #[db(name = "users", model, controller)]
    pub struct User {
        #[db(ty(id))]
        id: u64,
        #[db(ty(foreign(many(GroupUser))))]
        groups: Vec<Group>,
        #[db(ty(foreign()), request(name = "contact_id"), name = "contact_id")]
        contact: Contact,
        #[db(ty(varchar = 255))]
        name: String,
        #[db(ty(varchar = 255))]
        email: String,
        email_verified_at: Option<chrono::DateTime<chrono::Utc>>,
        #[db(ty(varchar = 255), response(skip), request(validate(min_len(12))))]
        password: String,
        #[db(ty(varchar = 100), response(skip))]
        remember_token: Option<String>,
        #[db(ty(on_create))]
        created_at: chrono::DateTime<chrono::Utc>,
        #[db(ty(on_update))]
        updated_at: chrono::DateTime<chrono::Utc>,
    }
    #[db(name = "group_user", many_model)]
    pub struct GroupUser {
        #[db(ty(foreign()), request(name = "group_id"), name = "group_id")]
        group: Group,
        #[db(ty(foreign()), request(name = "user_id"), name = "user_id")]
        user: User,
    }
}
```
