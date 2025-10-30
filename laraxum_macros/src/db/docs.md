Define a database, tables and columns.

The database is defined using the `db` attribute macro on a module:

- `name`  
  The name of the database.  
  __Type__: string  
  __Optional__: true  
  __Default__: The module name.  
  __Examples__:

  - `name = "my_database_name"`

Each table is defined using the `db` attribute on a struct in the module:

- `name`  
  The name of the table in the database.  
  __Type__: string  
  __Optional__: true  
  __Default__: The struct name.  
  __Examples__:

  - `name = "my_table_name"`

- `model`  
  Implement the [Model] trait for the table, there must be a column that is an id.  
  Use the `many` attribute to implement [ManyModel] instead.  
  __Type__: object  
  __Optional__: true  
  __Fields__:

  - `many`  
    Implement the [ManyModel] trait instead of the [Model] trait.  
    There is a value column and an identifier column to identify multiple values in the value column.
    The `AggregateBy` type generic is a marker type for the identifier column.  
    The trait is implemented for each column, distinguished by the marker type for the column. The marker type is the struct defined in the `struct_name` attribute of the column, otherwise it defaults to the type of the column. This can be used to create many-to-many relationships.  
    __Type__: bool  
    __Optional__: true  
    __Default__: false  
    __Examples__:

    - The struct is called `CustomersAndGroups` with a `group` field with a `Group` type and a `customer` field with a `Customer` type, with a `struct_name` attribute of `CustomerGroups` (defines new struct to reference this field).  
    To access the groups that correspond to/belong to a certain customer, use `<CustomersAndGroups as ManyModel<CustomerGroups>>` with a customer's id. This is because the `customer` field set the `struct_name` attribute as `CustomerGroups`.  
    To access the customers that correspond to/belong to a certain group, use `<CustomersAndGroups as ManyModel<Group>>` with a group's id. This is because the `group` field didn't set the `struct_name` attribute, so it defaults to `Group`.  

  __Examples__:

  - `model(many = false)`
  - `model(many = true)`
  - `model()` (same as `model(many = false)`)
  - `model(many)` (same as `model(many = true)`)

- `controller`  
  Implement the [Controller] trait.  
  It can also be implemented manually for more control.  
  __Type__: object  
  __Optional__: true  
  __Fields__:

  - `auth`  
    Authentication and authorization type for the controller.  
    See [auth].  
    __Type__: type  
    __Optional__: true  

  __Examples__:

  - `controller()`
  - `controller(auth(UserCredentials))`

- `aggregate_name`  
  Defines an aggregator which is an enum of all aggregators in this table.  
  Only aggregators with the `pub` attribute will be included.  
  It is used as [Controller::GetManyRequestQuery], which will select the correct aggregator based on the fields in the query parameters, otherwise it will default to returning all records. The aggregator must be an exact match with the query parameters, which means the query parameters can have other fields, but not fields from other aggregators.  
  If there is a sort aggregator and a filter aggregator, it will fail if there are query parameters for both sorting and for filtering because it doesn't know what to do and silently ignoring it would be ambiguous.  
  If there is a sort aggregator only, it will sort if there are query parameters for both sorting and for filtering because filtering isn't possible.  
  See [AggregateMany] and [AggregateOne].  
  __Type__: identifier  
  __Optional__: true  
  __Examples__:

  - `aggregate_name(TableAggregate)`

Each column is defined using the `db` attribute on a field in the struct:

- `name`  
  The name of the column in the database.  
  __Type__: string  
  __Optional__: true  
  __Default__: The field name.  
  __Examples__:

  - `name = "my_column_name"`
- `ty`  
  Optional type information in addition to the field's type.  
  They can change the behavior of the column and only work with specific types.  
  Without this, each field will behave as a normal readable/writable column.  
  __Type__: enum  
  __Optional__: true  
  __Variants__:

  - `id`  
    Primary Key.  
    Set automatically once when the record is created.  
    Field type must be an integer.  
  - `foreign`  
    Refer to a record in another table.
    The field type is the struct of the table and can be an `Option<T>` to make it nullable.
    The request type is the primary key of the table.  
    By default, this is a many-to-one relation which
    means many records can refer to one foreign record.
    If the column is unique, then this is a one-to-one relation which
    means one record can refer to one foreign record.  
    Use the `many` attribute to make this a many-to-many relation.  
    __Type__: object  
    __Fields__:

    - `many`  
      Refer to many records in another table.  
      This a many-to-many relation which means many records can refer to many foreign records.  
      The field type is a `Vec<T>` of the struct of the table.
      The request type is a `Vec<T>` of the primary keys of the table.  
      __Type__: object  
      __Optional__: true  
      __Fields__:

      - `model`  
        The [ManyModel] that refers to this table and the foreign table.  
        __Type__: identifier  
        __Optional__: false  
      - `aggregate`  
        The type used to aggregate the [ManyModel] to get the foreign table.  
        If the field in the [ManyModel] hasn't set the `struct_name` attribute, the type is the type of this table. If you leave this attribute empty, it will be the type of this table. Therefore, if `struct_name` isn't set, don't set this attribute either.  
        __Type__: identifier  
        __Optional__: true  
    - `on_create`  
      The time this record was created.  
      This column isn't settable and never changes.  
      The field type must be a time type.  
    - `on_update`  
      The time this record was last updated.  
      This column isn't settable and changes whenever this record is updated.  
      The field type must be a time type.  
    - `varchar`  
      A string with a dynamic length.  
      The field type must be a `String` or an `Option<String>`.  
      The number is the maximum length of the string.  
      If the field type is a string and the `ty` attribute isn't set, it is the same thing as using this attribute with a length of `255`.  
      __Type__: unsigned integer  
      __Optional__: false  
    - `char`  
      A string with a set length.  
      The field type must be a `String` or an `Option<String>`.
      The number is the length of the string.  
      __Type__: unsigned integer  
      __Optional__: false  
    - `text`  
      A string with a dynamic length for very large strings.
      The field type must be a `String` or an `Option<String>`.  
- `response`  
  The response for returning a record.  
  __Type__: object  
  __Optional__: true  
  __Fields__:

  - `name`  
    The name to serialize the field in the response.  
    __Type__: string  
    __Optional__: true  
    __Default__: The name of the field.  
  - `skip`  
    Skip serializing the field in the response.  
    __Type__: bool  
    __Optional__: true  
    __Default__: false  
- `request`  
  The request for creating/updating a record.  
  __Type__: object  
  __Optional__: true  
  __Fields__:

  - `name`  
    The name to deserialize the field in the request.  
    __Type__: string  
    __Optional__: true  
    __Default__: The name of the field.  
  - `validate`  
    Validation rules for the request.  
    __Type__: object  
    __Optional__: true  
    __Fields__:

    - `min_len`  
      Minimum length.  
      __Type__: unsigned integer  
      __Optional__: true  
    - `matches`  
      Value must match range pattern.
      __Type__: range pattern  
      __Optional__: true  
      __Examples__:

      - `matches(..6)` Less than 6  
      - `matches(..=6)` Less than or equal to 6  
      - `matches(10..)` Greater or equal to 10  
    - `func`  
    Custom function to validate.  
    The type of the function is `fn(&T) -> Result<(), &'static str>`.
    Return `Ok(())` if successful or `Err(&'static str)` with an error message if failed.  
    __Type__: expression  
    __Optional__: true  
- `real_ty`  
  When using a wrapper type, set this attribute to the inner type and
  set the field type to the wrapper type.  
  __Type__: type  
  __Optional__: true  
- `unique`  
  This column is unique.  
  This affects how aggregating works.
  If you filter by this column, it will return zero or one records.  
  __Type__: bool  
  __Optional__: true  
- `mut`  
  This column is mutable and can be updated.  
  __Type__: bool  
  __Optional__: true  
  __Default__: true  
- `aggregate`  
  Create an aggregator that can query this column.  
  __Type__: object  
  __Repeatable__: true (This attribute can be set multiple times to create multiple aggregators)  
  __Fields__:

  - `name`  
    The name of the aggregator.  
    __Type__: identifier  
    __Optional__: false  
  - `filter`  
    Aggregator filter behavior.  
    __Type__: enum  
    __Optional__: false  
    __Variants__:

    - `none`  
      Do not filter by this column.  
    - `eq`  
      Filter where equal.  
    - `like`  
      Filter where using SQL like comparison.  
    - `gt`  
      Filter where greater.  
    - `lt`  
      Filter where less.  
    - `gte`  
      Filter where greater or equal.  
    - `lte`  
      Filter where less or equal.  
  - `sort`  
    Aggregator filter behavior. Sort by this column.  
    __Type__: bool  
    __Optional__: true  
    __Default__: false  
  - `limit`  
    Aggregator limit/pagination behavior.  
    __Type__: enum  
    __Optional__: true  
    __Default__: `none`  
    __Variants__:

    - `none`  
      Do not use limit.  
    - `limit`  
      Use limit.  
    - `page`  
      Page based pagination.  
      Select page to view with a number.  
      Page size isn't dynamic (yet) so you have to choose one size to use.  
      __Type__: object  
      __Fields__:

      - `per_page`  
        How many items per page.  
        __Type__: unsigned integer  
        __Optional__: false  
  - `pub`  
    If `aggregate_name` for the table is set, this aggregator will be in the table aggregator as well.  
    __Type__: bool  
    __Optional__: true  
    __Default__: false  
- `borrow`  
  This column should be borrowed instead of owned when possible, for example when aggregating.  
  Optionally set a different type to use when borrowing.  
  When the column is a string, it is recommended to set this attribute as `borrow(str)`,
  so that you don't need to allocate a `String` on the heap just to aggregate.  
  __Type__: type or none  
  __Optional__: true  
  __Examples__:

  - `borrow`
  - `borrow(str)`
- `struct_name`  
  Create an empty struct to represent this field.
  This affects how the [ManyModel] works.
  If this is set, the struct will be used instead of the field type.
  See the `ty(foreign(many(aggregate())))` attribute to see how this affects the [ManyModel] when
  a table references a [ManyModel].  
  __Type__: identifier  
  __Optional__: true  

### Example

```rust
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
        groups: Vec<Group>, // many-to-many relationship
        #[db(ty(foreign()), request(name = "contact_id"), name = "contact_id")]
        contact: Contact,
        #[db(ty(varchar = 255))]
        name: String,
        #[db(ty(varchar = 255), unique, aggregate(name(UserEmail)), borrow(str))] // <User as AggregateOne<UserEmail>>
        email: String,
        #[db(ty(varchar = 255), response(skip), request(validate(min_len = 12)))]
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

[Model]: laraxum::model::Model
[AggregateOne]: laraxum::model::AggregateOne
[AggregateMany]: laraxum::model::AggregateMany
[ManyModel]: laraxum::model::ManyModel
[Controller]: laraxum::controller::Controller
[Controller::GetManyRequestQuery]: laraxum::controller::Controller::GetManyRequestQuery
[auth]: laraxum::controller::auth
<!-- [Collection]: laraxum::model::Collection -->
