# Laraxum

Create backend API servers easily using [Axum](https://crates.io/crates/axum) and [SQLX](https://crates.io/crates/sqlx).  
It uses the MVC paradigm...:

- Model: manages the data storage and interacts with the database.
- View: manages the data input/output and interacts with the end user.
- Controller: manages the connection between model and view.

...and handles the Model and Controller, which the View (frontend client) can interact with.

## `macro db`

Define a database, tables and columns.  
The database is defined using the `db` attribute macro on a module.  
Each table is defined using the `db` attribute on a struct in the module.  
Each column is defined using the `db` attribute on a field in the struct.  

## Model

A model manages the data storage and interacts with the database.  

### `trait Collection`

- `fn get_all` Return all records.
- `fn create_one` Create a record.

### `trait Model`

- `fn get_one` Return a record.
- `fn create_get_one` Create a record and return it.
- `fn update_one` Update a record.
- `fn update_get_one` Update a record and return it.
- `fn patch_one` Patch update a record.
- `fn patch_get_one` Patch update a record and return it.
- `fn delete_one` Delete a record.

### `trait ManyModel`

A manymodel is similar to a model but with two columns.
The column will be used as an id for multiple values in the other column.  
This can be used to create many-to-many relationships.

### `trait AggregateMany` and `trait AggregateOne`

Aggregate in a table.

## Controller

A controller manages the connection between model and view.  

### `trait Controller`

- `fn get_many` Return records.
- `fn get` Return a record.
- `fn create` Create a record and return it.
- `fn update` Update a record and return it.
- `fn patch` Patch update a record and return it.
- `fn delete` Delete a record.

- `type GetManyRequestQuery` The query parameters that can be used for custom requests using indexes.

- `type State` The stateful context of the controller,
which contains the database connection.

- `type Auth` Request authentication.  
`AuthToken<()>` doesn't do any authentication.  
You can implement the authenticate trait for custom authentication and use it like `AuthToken<T>`.  

## `macro router`

Create a router.  
The router has methods and routes. A route is a path and a router, which makes it nested and recursive. If the router has methods, they are created at the start in a `use` statement. You can either create each method route with curly brackets like a struct expression where the field name is the router method, or you can give the controller, which will create all the method routes and nested method routes for that controller.  

## `macro serve`

Serve the app at address in `URL` environment variable, defaults to "localhost:80".  
