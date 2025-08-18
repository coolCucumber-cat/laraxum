# Laraxum

Create API database servers easily using Axum and SQLX.
It is inspired by Laravel and uses the MVC paradigm:

- Model: manages the data storage and interacts with the database.
- View: manages the data input/output and interacts with the end user.
- Controller: manages the connection between model and view.

This framework is only responsible for creating backend half of an app,
meaning the model and controller.
You can create your own frontend, meaing the view.
Or you can just use it as an API and interact with the controller.

## Model

A model manages the data storage and interacts with the database.
It implements the collection trait:

- `get_all`
- `create_one`

and the model trait:

- `get_one`
- `update_one`
- `delete_one`

### Many Model

A manymodel is similar to a model but with two columns.
The manymodel trait can be implemented for each column.
The column will be used as an id for multiple values in the other column.

This can be used to create many-to-many relationships.

## Controller

A controller manages the connection between model and view.
It implements the controller trait:

- `get_many`
- `get`
- `create`
- `update`
- `delete`

The `State` type is the type that will be available as context in the controller,
which contains the database connection.

The `Auth` type is the type that will authenticate the request.
`AuthToken<()>` doesn't do any authentication.
You can implement the authenticate trait for custom authentication and use it like `AuthToken<T>`.

## Auth

Implement the authenticate trait to allow custom authentication that can be used in the controller.
`AuthToken<T>` should be returned from the login endpoint, which will encrypt it.
To authenticate the request, it will be decrypted.

Implement the authorize trait for custom authorization.
You must specify a type that is authenticatable

### Example

```rs
struct UserAuth {
    admin: bool,
}
impl From<bool> for UserAuth {
    fn from(admin: bool) -> Self {
        Self { admin }
    }
}
impl From<UserAuth> for bool {
    fn from(user: UserAuth) -> Self {
        user.admin
    }
}
impl Authenticate for UserAuth {
    type State = AppDb;
    fn authenticate(&self, state: &Arc<Self::State>) -> Result<(), AuthError> {
        // authenticate the user
        Ok(())
    }
}

struct UserAuthAdmin;
impl Authorize for UserAuthAdmin {
    type Authenticate = UserAuth;
    fn authorize(authorize: Self::Authenticate) -> Result<Self, AuthError> {
        if authorize.admin {
            Ok(UserAuthAdmin)
        } else {
            Err(AuthError::Unauthorized)
        }
    }
}

mod AppDb {
    #[db(model(), controller())]
    struct Anyone {}

    #[db(model(), controller(auth(User)))]
    struct UserOnly {}

    #[db(model(), controller(auth(UserAdmin)))]
    struct AdminOnly {}
}
```

## Db

The database is defined using the `db` attribute macro on a module:

- `name`: `Option<String>`, the name of the database

Each struct in the module is a table. Use the `db` attribute on the struct:

- `name`: `Option<String>`, the name of the table
- `model`: `Option<struct>`, implement model and use these options
  - `many`: `Option<bool>`, implement a manymodel instead of model
- `controller`: `Option<struct>`, implement controller and use these options
  - `auth`: `Option<Type>`, the type to use for authentication and authorization

Each field in the struct is a column. Use the `db` attribute on the field:

- `name`: `Option<String>`, the name of the column
- `ty`: `Option<enum>`
    1. `id` primary key
    2. `foreign`: `struct`, foreign key
        - `many`: `Option<Ident>`, a many-to-many relationship with the named table
    3. `on_create` the time this entity was created
    4. `on_update` the time this entity was last update
    5. `varchar`: `u16` string type with dynamic length, set max length
    6. `char`: `u16`, string type with fixed length, set length
    7. `text` string type for extra large strings
- `response`: `Option<struct>`
  - `name`: `Option<String>`, the name of the field in the response when serialized
  - `skip`: `bool`, skip the field in the response when serialized
- `request`:
  - `name`: `Option<String>`, the name of the field in the request when deserialized
  - `validate`: `Option<[enum]>`
    1. `min_len`: `Expr`
    2. `func`: `Expr`
    3. `matches`: `Pat`
    4. `n_matches`: `Pat`
    5. `eq`: `Expr`
    6. `n_eq`: `Expr`
    7. `gt`: `Expr`
    8. `lt`: `Expr`
    9. `gte`: `Expr`
    10. `lte`: `Expr`
- `real_ty`: `Option<Type>`, when using wrapper types, this is the inner type
- `unique`: `Option<bool>`, this column is unique
- `index`: `Option<Ident>`, create an index that can filter using this column, like `<User as CollectionIndexOne<UserEmail>>`

If you don't use the controller option, the controller won't be implemented.
You can implement it yourself or not at all.
The functions have a default implementation so you don't have to implement each one by yourself.

### Example

```rs
#[db(name = "database")]
pub mod AppDb {
    #[db(name = "addresses", model(), controller())]
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
    #[db(name = "contacts", model(), controller())]
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
    #[db(name = "groups", model(), controller())]
    pub struct Group {
        #[db(ty(id))]
        id: u64,
        #[db(ty(varchar = 255))]
        title: String,
    }
    #[db(name = "users", model(), controller())]
    pub struct User {
        #[db(ty(id))]
        id: u64,
        #[db(ty(foreign(many(GroupUser))))]
        groups: Vec<Group>, // many to many relationship
        #[db(ty(foreign()), request(name = "contact_id"), name = "contact_id")]
        contact: Contact,
        #[db(ty(varchar = 255))]
        name: String,
        #[db(ty(varchar = 255), unique, index(UserEmail))] // <User as CollectionIndexOne<UserEmail>>
        email: String,
        #[db(ty(varchar = 255), response(skip), request(validate(min_len(12))))]
        password: String,
        #[db(ty(on_create))]
        created_at: chrono::DateTime<chrono::Utc>,
        #[db(ty(on_update))]
        updated_at: chrono::DateTime<chrono::Utc>,
        admin: bool
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
