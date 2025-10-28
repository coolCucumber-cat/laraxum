The database is defined using the `db` attribute macro on a module:

> `db(`
>
>> The name of the database.  
>>
>> Default: the module name.  
>>
>> `name =` `<String>` `,`
>
> `)`

Each table is defined using the `db` attribute on a struct in the module:

> `db(`
>
>> The name of the table in the database.  
>>
>> Default: the struct name.  
>>
>> `name =` `<String>` `,`
>
>> Implement model.  
>>
>> `model(`
>>
>>> Implement manymodel instead of model.  
>>> Use the `struct_name` attribute to change the type used to refer to each column.  
>>>
>>> Default: `false`.  
>>>
>>> `many =` `<bool>` `,`
>>
>> `),`
>
>> Implement controller.  
>> It can also be implemented manually for extra control.  
>>
>> `controller(`
>>
>>> The type to use for authentication and authorization.  
>>> It must implement the `Àuthorize` trait.
>>> Anything that implements that the `Authenticate` trait also
>>> automatically implements the `Authorize` trait.  
>>>
>>> `auth(` `<Type>` `),`
>>
>> `),`
>
>> An aggregator which is an enum of all aggregators in this table.  
>> Only aggregators with the `pub` attribute will be included.  
>> Is used as `Controller::GetManyRequestQuery`,
>> which will select the correct aggregator based on the fields in the query parameters,
>> otherwise it will default to returning all records.
>> The aggregator must be an exact match with the query parameters,
>> which means the query parameters can have other fields, but not fields from other aggregators.  
>>
>> Example: if there is a sort aggregator and a filter aggregator,
>> it will fail if there are query parameters for both sorting and for filtering
>> because it doesn't know what to do and silently ignoring it would be ambiguous.  
>>
>> Example: if there is a sort aggregator only,
>> it will sort if there are query parameters for both sorting and for filtering
>> because filtering isn't possible.
>>
>> `aggregate_name(` `<Ident>` `),`
>
> `)`

Each column is defined using the `db` attribute on a field in the struct:

> `db(`
>
>> The name of the column in the database.  
>>
>> Default: the field name.  
>>
>> `name =` `<String>` `,`
>
>> Optional type information in addition to the field's type.  
>> They can change the behavior of the column and only work with specific types.  
>> Without this, each field will behave as a normal readable/writable column.  
>>
>> `ty(`
>>
>> 1. >Primary Key.  
>>     >Set automatically once when the record is created.  
>>     >Field type must be an integer.  
>>     >
>>     >`id`
>>
>> 2. >Foreign Key.  
>>     >Refer to a record in another table.
>>     >The field type is the struct of the table and can be an `Option<T>` to make it nullable.
>>     >The request type is the primary key of the table.  
>>     >By default, this is a many-to-one relation which
>>     >means many records can refer to one foreign record.
>>     >If the column is unique, then this is a one-to-one relation which
>>     >means one record can refer to one foreign record.  
>>     >Use the `many` attribute to make this a many-to-many relation.  
>>     >
>>     >`foreign(`
>>     >
>>     >>Refer to many records in another table.  
>>     >>This a many-to-many relation which means many records can refer to many foreign records.  
>>     >>The field type is a `Vec<T>` of the struct of the table.
>>     >>The request type is a `Vec<T>` of the primary keys of the table.  
>>     >>
>>     >>`many(`
>>     >>
>>     >>>The manymodel that refers to this table and the foreign table.
>>     >>>
>>     >>>`model(` `<Ident>` `),`
>>     >>
>>     >>>The type used to aggregate the manymodel to get the foreign table.  
>>     >>>If the field in the manymodel hasn't set the `struct_name` attribute,
>>     >>>the type is the type of this table. If you leave this attribute empty,
>>     >>>it will be the type of this table.
>>     >>>Therefore, if `struct_name` isn't set, don't set this attribute either.  
>>     >>>
>>     >>>`aggregate(` `<Ident>` `),`
>>     >>
>>     >>`),`
>>     >
>>     >`),`
>>
>> 3. >The time this record was created.  
>>     >This column isn't settable and never changes.  
>>     >The field type must be a time type.  
>>     >
>>     >`on_create`
>>
>> 4. >The time this record was last updated.  
>>     >This column isn't settable and changes whenever this record is updated.  
>>     >The field type must be a time type.  
>>     >
>>     >`on_update`
>>
>> 5. >A string with a dynamic length.  
>>     >The field type must be a `String` or an `Option<String>`.
>>     >The number is the maximum length of the string.
>>     >If the field type is a string and the `ty` attribute isn't set,
>>     >it is the same thing as using this attribute with a length of `255`.  
>>     >
>>     >`varchar =` `<u16>`
>>
>> 6. >A string with a set length.
>>     >The field type must be a `String` or an `Option<String>`.
>>     >The number is the length of the string.  
>>     >
>>     >`char =` `<u16>`
>>
>> 7. >A string with a dynamic length for very large strings.
>>     >The field type must be a `String` or an `Option<String>`.  
>>     >
>>     >`text`
>>
>> `),`
>
>> Attributes for the field in a response to get the record.
>>
>> `response(`
>>
>>> The name of the field when serializing the response.  
>>>
>>> Default: the field name.  
>>>
>>> `name =` `<String>` `,`
>>
>>> Skip the field when serializing the response.  
>>>
>>> Default: `false`.  
>>>
>>> `skip =` `<bool>` `,`
>>
>> `),`
>
>> Attributes for the field in a request to create or update the record.
>>
>> `request(`
>>
>>> The name of the field when deserializing the request.  
>>>
>>> Default: the field name.  
>>>
>>> `name =` `<String>` `,`
>>
>>> Validation rules for the request.  
>>>
>>> `validate(`
>>>
>>>> Must be minimum length.
>>>>
>>>> `min_len =` `<int>` `,`
>>>
>>>> Custom function to validate.  
>>>> The type of the function is `fn(&T) -> Result<(), &'static str>`.
>>>> Return `Ok(())` if successful or `Err(&'static str)` with an error message if failed.  
>>>>
>>>> `func(` `<Expr>` `),`
>>>
>>>> Must match range pattern.  
>>>>
>>>> Example:
>>>>
>>>> - `..6` less than 6  
>>>> - `..=6` less than or equal to 6  
>>>> - `10..` greater or equal to 10  
>>>>
>>>> `matches(` `<PatRange>` `),`
>>>
>>> `),`
>>
>> `),`
>
>> When using a wrapper type, set this attribute to the inner type and
>> set the field type to the wrapper type.  
>>
>> `real_ty(` `<Type>` `),`
>
>> This column is unique.  
>> This affects how aggregating works.
>> If you filter by this column, it will return zero or one records.  
>>
>> Default: `false`.  
>>
>> `unique =` `<bool>` `,`
>
>> This column cannot be updated.  
>>
>> Default: `false`.  
>>
>> `mut =` `<bool>` `,`
>
>> Create an aggregate that can query this column.  
>> This attribute can be set multiple times to create multiple aggregators.  
>>
>> `aggregate(`
>>
>>> The name.  
>>>
>>> `name =` `<Ident>` `,`
>>
>>> Filter by this column.  
>>>
>>> Default: `none`.  
>>>
>>> `filter(`
>>>
>>> 1. >Do not use filter.  
>>>     >
>>>     >`none`
>>>
>>> 2. >Filter where equal.  
>>>     >
>>>     >`eq`
>>>
>>> 3. >Filter where using SQL like comparison.  
>>>     >
>>>     >`like`
>>>
>>> 4. >Filter where greater.
>>>     >
>>>     >`gt`
>>>
>>> 5. >Filter where less.
>>>     >
>>>     >`lt`
>>>
>>> 6. >Filter where greater or equal.
>>>     >
>>>     >`gte`
>>>
>>> 7. >Filter where less or equal.
>>>     >
>>>     >`lte`
>>>
>>> `),`
>>
>>> Sort by this column.  
>>>
>>> Default: `false`.  
>>>
>>> `sort =` `<bool>` `,`
>>
>>> Limit number of records.  
>>>
>>> Default: `none`.  
>>>
>>> `limit(`
>>>
>>> 1. >Do not use limit.  
>>>     >
>>>     >`none`
>>>
>>> 2. >Use limit.  
>>>     >
>>>     >`limit`
>>>
>>> 3. >Page based pagination.  
>>>     >Select page to view with a number.  
>>>     >Page size isn't dynamic (yet) so you have to choose one size to use.  
>>>     >
>>>     >`page(`
>>>     >
>>>     >>How many items per page.  
>>>     >>
>>>     >>`per_page =` `<u64>`
>>>     >
>>>     >`)`
>>>
>>> `),`
>>
>>> If `aggregate_name` for the table is set, this aggregator will be in the table aggregator as well.  
>>>
>>> Default: `false`.  
>>>
>>> `controller =` `<bool>` `,`
>>
>> `),`
>
>> This column should be borrowed instead of owned when possible, for example when aggregating.  
>> Optionally set a different type to use when borrowing.  
>> When the column is a string, it is recommended to set this attribute as `borrow(str)`,
>> so that you don't need to allocate a `String` on the heap just to aggregate.  
>>
>> `borrow`  `|`  `borrow(` `<Type>` `)` `,`
>
>> Create an empty struct to represent this field.
>> This affects how the manymodel works.
>> If this is set, the struct will be used instead of the field type.
>> See the `ty(foreign(many(aggregate())))` attribute to see how this affects the manymodel when
>> a table references a manymodel.  
>>
>> `struct_name(` `<Ident>` `),`
>
> `)`

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
        groups: Vec<Group>, // many to many relationship
        #[db(ty(foreign()), request(name = "contact_id"), name = "contact_id")]
        contact: Contact,
        #[db(ty(varchar = 255))]
        name: String,
        #[db(ty(varchar = 255), unique, aggregate(name(UserEmail)), borrow(str))] // <User as CollectionaggregateOne<UserEmail>>
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
