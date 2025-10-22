# Laraxum

Create API database servers easily using Axum and SQLX.
It uses the MVC paradigm:

- Model: manages the data storage and interacts with the database.
- View: manages the data input/output and interacts with the end user.
- Controller: manages the connection between model and view.

This framework is only responsible for creating backend half of an app,
meaning the model and controller.
You can create your own frontend, meaing the view.
Or you can just use it as an API and interact with the controller.

## Model

A model manages the data storage and interacts with the database.  
It implements the `Collection` trait:

- `get_all` Return all records.
- `create_one` Create a record.

and the `Model` trait:

- `get_one` Return a record.
- `create_get_one` Create a record and return it.
- `update_one` Update a record.
- `update_get_one` Update a record and return it.
- `patch_one` Patch update a record.
- `patch_get_one` Patch update a record and return it.
- `delete_one` Delete a record.

### ManyModel

A manymodel is similar to a model but with two columns.
The `ManyModel` trait can be implemented for each column.
The column will be used as an id for multiple values in the other column.  
This can be used to create many-to-many relationships.

## Controller

A controller manages the connection between model and view.  
It implements the `Controller` trait:

- `get_many` Return records.
- `get` Return a record.
- `create` Create a record and return it.
- `update` Update a record and return it.
- `patch` Patch update a record and return it.
- `delete` Delete a record.

- `GetManyRequestQuery` The query parameters that can be used for custom requests using indexes.

- `type State` The stateful context of the controller,
which contains the database connection.

- `type Auth` Request authentication.  
`AuthToken<()>` doesn't do any authentication.  
You can implement the authenticate trait for custom authentication and use it like `AuthToken<T>`.  

## Db

The database is defined using the `db` attribute macro on a module:

>`db(`
>
>>The name of the database.  
>>Default: the module name.  
>>
>>`name =` `<String>` `,`
>
>`)`

Each struct in the module is a table. Use the `db` attribute on the struct:

>`db(`
>
>>The name of the table in the database.  
>>Default: the struct name.  
>>
>>`name =` `<String>` `,`
>
>>Implement model.  
>>
>>`model(`
>>
>>>Implement manymodel instead of model.  
>>>Use the `struct_name` attribute to change the type used to refer to each column.  
>>>Default: `false`.  
>>>
>>>`many =` `<bool>` `,`
>>
>>`),`
>
>>Implement controller.  
>>
>>`controller(`
>>
>>>The type to use for authentication and authorization.  
>>>It must implement the `Àuthorize` trait.
>>>Anything that implements that the `Authenticate` trait also
>>>automatically implements the `Authorize` trait.  
>>>
>>>`auth(` `<Type>` `),`
>>
>>`),`
>
>>A custom request type for the controller.  
>>Is used as `type GetManyRequestQuery`
>>Includes all indexes for this table that have set the `controller` attribute.  
>>A query with the corresponding keys will select one of the indexes and fetch using that index.  
>>
>>`index_name(` `<Ident>` `),`
>
>`)`

Each field in the struct is a column. Use the `db` attribute on the field:

>`db(`
>
>>The name of the column in the database.  
>>Default: the field name.  
>>
>>`name =` `<String>` `,`
>
>>Optional type information in addition to the field's type.  
>>They can change the behavior of the column and only work with specific types.  
>>Without this, each field will behave as a normal readable/writable column.  
>>
>>`ty(`
>>
>> 1. >Primary Key.  
>>    >Set automatically once when the record is created.  
>>    >Field type must be an integer.  
>>    >
>>    >`id`
>>
>> 2. >Foreign Key.  
>>    >Refer to a record in another table.
>>    >The field type is the struct of the table and can be an `Option<T>` to make it nullable.
>>    >The request type is the primary key of the table.  
>>    >By default, this is a many-to-one relation which
>>    >means many records can refer to one foreign record.
>>    >If the column is unique, then this is a one-to-one relation which
>>    >means one record can refer to one foreign record.  
>>    >Use the `many` attribute to make this a many-to-many relation.  
>>    >
>>    >`foreign(`
>>    >
>>    >>Refer to many records in another table.  
>>    >>This a many-to-many relation which means many records can refer to many foreign records.  
>>    >>The field type is a `Vec<T>` of the struct of the table.
>>    >>The request type is a `Vec<T>` of the primary keys of the table.  
>>    >>
>>    >>`many(`
>>    >>
>>    >>>The manymodel that refers to this table and the foreign table.
>>    >>>
>>    >>>`model(` `<Ident>` `),`
>>    >>
>>    >>>The type used to index the manymodel to get the foreign table.  
>>    >>>If the field in the manymodel hasn't set the `struct_name` attribute,
>>    >>>the type is the type of this table. If you leave this attribute empty,
>>    >>>it will be the type of this table.
>>    >>>Therefore, if `struct_name` isn't set, don't set this attribute either.  
>>    >>>
>>    >>>`index(` `<Ident>` `),`
>>    >>
>>    >>`),`
>>    >
>>    >`),`
>>
>> 3. >The time this record was created.  
>>    >This column isn't settable and never changes.  
>>    >The field type must be a time type.  
>>    >
>>    >`on_create`
>>
>> 4. >The time this record was last updated.  
>>    >This column isn't settable and changes whenever this record is updated.  
>>    >The field type must be a time type.  
>>    >
>>    >`on_update`
>>
>> 5. >A string with a dynamic length.  
>>    >The field type must be a `String` or an `Option<String>`.
>>    >The number is the maximum length of the string.
>>    >If the field type is a string and the `ty` attribute isn't set,
>>    >it is the same thing as using this attribute with a length of `255`.  
>>    >
>>    >`varchar =` `<u16>`
>>
>> 6. >A string with a set length.
>>    >The field type must be a `String` or an `Option<String>`.
>>    >The number is the length of the string.  
>>    >
>>    >`char =` `<u16>`
>>
>> 7. >A string with a dynamic length for very large strings.
>>    >The field type must be a `String` or an `Option<String>`.  
>>    >
>>    >`text`
>>
>>`),`
>
>>Attributes for the field in a response to get the record.
>>
>>`response(`
>>
>>>The name of the field when serializing the response.  
>>>Default: the field name.  
>>>
>>>`name =` `<String>` `,`
>>
>>>Skip the field when serializing the response.  
>>>Default: `false`.  
>>>
>>>`skip =` `<bool>` `,`
>>
>>`),`
>
>>Attributes for the field in a request to create or update the record.
>>
>>`request(`
>>
>>>The name of the field when deserializing the request.  
>>>Default: the field name.  
>>>
>>>`name =` `<String>` `,`
>>
>>>Validation rules for the request.  
>>>
>>>`validate(`
>>>
>>>>Must be minimum length.
>>>>
>>>>`min_len =` `<int>` `,`
>>>
>>>>Custom function to validate.  
>>>>The type of the function is `fn(&T) -> Result<(), &'static str>`.
>>>>Return `Ok(())` if successful or `Err(&'static str)` with an error message if failed.  
>>>>
>>>>`func(` `<Expr>` `),`
>>>
>>>>Must match range pattern.  
>>>>Example:
>>>>
>>>> - `..6` less than 6  
>>>> - `..=6` less than or equal to 6  
>>>> - `10..` greater or equal to 10  
>>>>
>>>>`matches(` `<PatRange>` `),`
>>>
>>>`),`
>>
>>`),`
>
>>When using a wrapper type, set this attribute to the inner type and
>>set the field type to the wrapper type.  
>>
>>`real_ty(` `<Type>` `),`
>
>>This column is unique.  
>>This affects how indexing works.
>>If you filter by this column, it will return zero or one records.  
>>Default: `false`.  
>>
>>`unique =` `<bool>` `,`
>
>>This column cannot be updated.  
>>Default: `false`.  
>>
>>`mut =` `<bool>` `,`
>
>>Create an index that can query this column.  
>>This attribute can be set multiple times to create multiple indexes.  
>>
>>`index(`
>>
>>>The name.  
>>>
>>>`name =` `<Ident>` `,`
>>
>>>Filter by this column.  
>>>Default: `none`.  
>>>
>>>`filter(`
>>>
>>> 1. >Do not use filter.  
>>>    >
>>>    >`none`
>>>
>>> 2. >Filter where equal.  
>>>    >
>>>    >`eq`
>>>
>>> 3. >Filter where using SQL like comparison.  
>>>    >
>>>    >`like`
>>>
>>> 4. >Filter where greater.
>>>    >
>>>    >`gt`
>>>
>>> 5. >Filter where less.
>>>    >
>>>    >`lt`
>>>
>>> 6. >Filter where greater or equal.
>>>    >
>>>    >`gte`
>>>
>>> 7. >Filter where less or equal.
>>>    >
>>>    >`lte`
>>>
>>>`),`
>>
>>>Sort by this column.  
>>>Default: `false`.  
>>>
>>>`sort =` `<bool>` `,`
>>
>>>Limit number of records.  
>>>Default: `none`.  
>>>
>>>`limit(`
>>>
>>> 1. >Do not use limit.  
>>>    >
>>>    >`none`
>>>
>>> 2. >Use limit.  
>>>    >
>>>    >`limit`
>>>
>>> 3. >Page based pagination.  
>>>    >Select page to view with a number.  
>>>    >Page size isn't dynamic (yet) so you have to choose one size to use.  
>>>    >
>>>    >`page(`
>>>    >
>>>    >>How many items per page.  
>>>    >>
>>>    >>`per_page =` `<u64>`
>>>    >
>>>    >`)`
>>>
>>>`),`
>>
>>>If `index_name` for the table is set, this index will be in the table index as well.  
>>>Default: `false`.  
>>>
>>>`controller =` `<bool>` `,`
>>
>>`),`
>
>>This column should be borrowed instead of owned when possible, for example when indexing.  
>>Optionally set a different type to use when borrowing.  
>>When the column is a string, it is reccommended to set this attribute as `borrow(str)`,
>>so that you don't need to allocate a `String` on the heap just to index.  
>>
>>`borrow`  `|`  `borrow(` `<Type>` `)` `,`
>
>>Create an empty struct to represent this field.
>>This affects how the manymodel works.
>>If this is set, the struct will be used instead of the field type.
>>See the `ty(foreign(many(index())))` attribute to see how this affects the manymodel when
>>a table references a manymodel.  
>>
>>`struct_name(` `<Ident>` `),`
>
>`)`

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
        #[db(ty(foreign(many(model(GroupUser)))))]
        groups: Vec<Group>, // many to many relationship
        #[db(ty(foreign()), request(name = "contact_id"), name = "contact_id")]
        contact: Contact,
        #[db(ty(varchar = 255))]
        name: String,
        #[db(ty(varchar = 255), unique, index(name(UserEmail)), borrow(str))] // <User as CollectionIndexOne<UserEmail>>
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

## Auth

Implement the `Authenticate` trait to allow custom authentication that can be used in the controller.
`AuthToken<T>` should be returned from the login endpoint, which will encrypt it.
To authenticate the request, it will be decrypted.

Implement the authorize trait for custom authorization.
You must specify a type that is authenticatable.

### Example

```rs
#[derive(serde::Serialize, serde::Deserialize)]
struct UserAuth {
    is_admin: bool,
}
/// Implementation agnostic logic for authentication.
///
/// Implementing this also implements the `Authorize` trait.
///
/// Anything can implement the `Authorize` trait to extend it and add custom authorization logic.
impl Authenticate for UserAuth {
    type State = AppDb;
    /// Executes after user is authenticated.
    ///
    /// You can add extra authentication logic that applies for all users.
    fn authenticate(&self, state: &Arc<Self::State>) -> Result<(), AuthError> {
        // authenticate the user
        Ok(())
    }
}
/// Token specific logic for authenticatiion.
///
/// Has sensible defaults which can customised, like the expiration time.
impl AuthenticateToken for UserAuth {}

/// A struct using the `UserAuth` token for authentication and extending it with authorization logic.
struct UserAuthAdmin;
impl Authorize for UserAuthAdmin {
    /// Use `UserAuth` for authentication.
    type Authenticate = UserAuth;
    /// Add extra authorization logic.
    fn authorize(authorize: Self::Authenticate) -> Result<Self, AuthError> {
        if authorize.is_admin {
            Ok(UserAuthAdmin)
        } else {
            Err(AuthError::Unauthorized)
        }
    }
}

mod AppDb {
    #[db(model(), controller())]
    struct Anyone {}

    #[db(model(), controller(auth(UserAuth)))]
    struct UserOnly {}

    #[db(model(), controller(auth(UserAuthAdmin)))]
    struct AdminOnly {}
}
```
