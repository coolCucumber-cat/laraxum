pub trait Visitor<'de>: Sized {
    /// The value produced by this visitor.
    type Value;
    // /// The error produced by this visitor.
    // type Error = &'static str;

    /// Format a message stating what data this Visitor expects to receive.
    ///
    /// This is used in error messages. The message should complete the sentence
    /// "This Visitor expects to receive ...", for example the message could be
    /// "an integer between 0 and 64". The message should not be capitalized and
    /// should not end with a period.
    ///
    /// ```edition2021
    /// # use std::fmt;
    /// #
    /// # struct S {
    /// #     max: usize,
    /// # }
    /// #
    /// # impl<'de> serde::de::Visitor<'de> for S {
    /// #     type Value = ();
    /// #
    /// fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    ///     write!(formatter, "an integer between 0 and {}", self.max)
    /// }
    /// # }
    /// ```
    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str(Self::TY)
    }
    /// The input contains a boolean.
    ///
    /// The default implementation fails with a type error.
    fn visit_bool(self, _: bool) -> Result<Self::Value, &'static str> {
        // Err(Error::invalid_type(Unexpected::Bool(v), &self))
        Err(Self::BOOL)
    }
    /// The input contains an `i8`.
    ///
    /// The default implementation forwards to [`visit_i64`].
    ///
    /// [`visit_i64`]: #method.visit_i64
    fn visit_i8(self, v: i8) -> Result<Self::Value, &'static str> {
        self.visit_i16(v.into())
    }
    /// The input contains an `i16`.
    ///
    /// The default implementation forwards to [`visit_i64`].
    ///
    /// [`visit_i64`]: #method.visit_i64
    fn visit_i16(self, v: i16) -> Result<Self::Value, &'static str> {
        self.visit_i32(v.into())
    }
    /// The input contains an `i32`.
    ///
    /// The default implementation forwards to [`visit_i64`].
    ///
    /// [`visit_i64`]: #method.visit_i64
    fn visit_i32(self, v: i32) -> Result<Self::Value, &'static str> {
        self.visit_i64(v.into())
    }
    /// The input contains an `i64`.
    ///
    /// The default implementation fails with a type error.
    fn visit_i64(self, _: i64) -> Result<Self::Value, &'static str> {
        // Err(Error::invalid_type(Unexpected::Signed(v), &self))
        Err(Self::UNSIGNED_INTEGER)
    }
    // /// The input contains a `i128`.
    // ///
    // /// The default implementation fails with a type error.
    // fn visit_i128(self, v: i128) -> Result<Self::Value, &'static str> {
    //     // let mut buf = [0u8; 58];
    //     // let mut writer = crate::format::Buf::new(&mut buf);
    //     // fmt::Write::write_fmt(&mut writer, format_args!("integer `{}` as i128", v)).unwrap();
    //     // Err(Error::invalid_type(
    //     //     Unexpected::Other(writer.as_str()),
    //     //     &self,
    //     // ))
    // 	Err(Self::UNSIGNED_INTEGER)
    // }
    /// The input contains a `u8`.
    ///
    /// The default implementation forwards to [`visit_u64`].
    ///
    /// [`visit_u64`]: #method.visit_u64
    fn visit_u8(self, v: u8) -> Result<Self::Value, &'static str> {
        self.visit_u16(v.into())
    }
    /// The input contains a `u16`.
    ///
    /// The default implementation forwards to [`visit_u64`].
    ///
    /// [`visit_u64`]: #method.visit_u64
    fn visit_u16(self, v: u16) -> Result<Self::Value, &'static str> {
        self.visit_u32(v.into())
    }
    /// The input contains a `u32`.
    ///
    /// The default implementation forwards to [`visit_u64`].
    ///
    /// [`visit_u64`]: #method.visit_u64
    fn visit_u32(self, v: u32) -> Result<Self::Value, &'static str> {
        self.visit_u64(v.into())
    }
    /// The input contains a `u64`.
    ///
    /// The default implementation fails with a type error.
    fn visit_u64(self, _: u64) -> Result<Self::Value, &'static str> {
        // Err(Error::invalid_type(Unexpected::Unsigned(v), &self))
        Err(Self::UNSIGNED_INTEGER)
    }
    // /// The input contains a `u128`.
    // ///
    // /// The default implementation fails with a type error.
    // fn visit_u128(self, v: u128) -> Result<Self::Value, &'static str> {
    //     let mut buf = [0u8; 57];
    //     let mut writer = crate::format::Buf::new(&mut buf);
    //     fmt::Write::write_fmt(&mut writer, format_args!("integer `{}` as u128", v)).unwrap();
    //     Err(Error::invalid_type(
    //         Unexpected::Other(writer.as_str()),
    //         &self,
    //     ))
    // }
    /// The input contains an `f32`.
    ///
    /// The default implementation forwards to [`visit_f64`].
    ///
    /// [`visit_f64`]: #method.visit_f64
    fn visit_f32(self, v: f32) -> Result<Self::Value, &'static str> {
        self.visit_f64(v.into())
    }
    /// The input contains an `f64`.
    ///
    /// The default implementation fails with a type error.
    fn visit_f64(self, _: f64) -> Result<Self::Value, &'static str> {
        // Err(Error::invalid_type(Unexpected::Float(v), &self))
        Err(Self::FLOAT)
    }
    /// The input contains a `char`.
    ///
    /// The default implementation forwards to [`visit_str`] as a one-character
    /// string.
    ///
    /// [`visit_str`]: #method.visit_str
    #[inline]
    fn visit_char(self, v: char) -> Result<Self::Value, &'static str> {
        self.visit_str(v.encode_utf8(&mut [0u8; 4]))
    }
    /// The input contains a string. The lifetime of the string is ephemeral and
    /// it may be destroyed after this method returns.
    ///
    /// This method allows the `Deserializer` to avoid a copy by retaining
    /// ownership of any buffered data. `Deserialize` implementations that do
    /// not benefit from taking ownership of `String` data should indicate that
    /// to the deserializer by using `Deserializer::deserialize_str` rather than
    /// `Deserializer::deserialize_string`.
    ///
    /// It is never correct to implement `visit_string` without implementing
    /// `visit_str`. Implement neither, both, or just `visit_str`.
    fn visit_str(self, _: &str) -> Result<Self::Value, &'static str> {
        // Err(Error::invalid_type(Unexpected::Str(v), &self))
        Err(Self::STR)
    }
    /// The input contains a string that lives at least as long as the
    /// `Deserializer`.
    ///
    /// This enables zero-copy deserialization of strings in some formats. For
    /// example JSON input containing the JSON string `"borrowed"` can be
    /// deserialized with zero copying into a `&'a str` as long as the input
    /// data outlives `'a`.
    ///
    /// The default implementation forwards to `visit_str`.
    #[inline]
    fn visit_borrowed_str(self, v: &'de str) -> Result<Self::Value, &'static str> {
        self.visit_str(v)
    }
    /// The input contains a string and ownership of the string is being given
    /// to the `Visitor`.
    ///
    /// This method allows the `Visitor` to avoid a copy by taking ownership of
    /// a string created by the `Deserializer`. `Deserialize` implementations
    /// that benefit from taking ownership of `String` data should indicate that
    /// to the deserializer by using `Deserializer::deserialize_string` rather
    /// than `Deserializer::deserialize_str`, although not every deserializer
    /// will honor such a request.
    ///
    /// It is never correct to implement `visit_string` without implementing
    /// `visit_str`. Implement neither, both, or just `visit_str`.
    ///
    /// The default implementation forwards to `visit_str` and then drops the
    /// `String`.
    #[inline]
    fn visit_string(self, v: String) -> Result<Self::Value, &'static str> {
        self.visit_str(&v)
    }
    /// The input contains a byte array. The lifetime of the byte array is
    /// ephemeral and it may be destroyed after this method returns.
    ///
    /// This method allows the `Deserializer` to avoid a copy by retaining
    /// ownership of any buffered data. `Deserialize` implementations that do
    /// not benefit from taking ownership of `Vec<u8>` data should indicate that
    /// to the deserializer by using `Deserializer::deserialize_bytes` rather
    /// than `Deserializer::deserialize_byte_buf`.
    ///
    /// It is never correct to implement `visit_byte_buf` without implementing
    /// `visit_bytes`. Implement neither, both, or just `visit_bytes`.
    fn visit_bytes(self, _: &[u8]) -> Result<Self::Value, &'static str> {
        // Err(Error::invalid_type(Unexpected::Bytes(v), &self))
        Err(Self::BYTES)
    }
    /// The input contains a byte array that lives at least as long as the
    /// `Deserializer`.
    ///
    /// This enables zero-copy deserialization of bytes in some formats. For
    /// example Postcard data containing bytes can be deserialized with zero
    /// copying into a `&'a [u8]` as long as the input data outlives `'a`.
    ///
    /// The default implementation forwards to `visit_bytes`.
    #[inline]
    fn visit_borrowed_bytes(self, v: &'de [u8]) -> Result<Self::Value, &'static str> {
        self.visit_bytes(v)
    }
    /// The input contains a byte array and ownership of the byte array is being
    /// given to the `Visitor`.
    ///
    /// This method allows the `Visitor` to avoid a copy by taking ownership of
    /// a byte buffer created by the `Deserializer`. `Deserialize`
    /// implementations that benefit from taking ownership of `Vec<u8>` data
    /// should indicate that to the deserializer by using
    /// `Deserializer::deserialize_byte_buf` rather than
    /// `Deserializer::deserialize_bytes`, although not every deserializer will
    /// honor such a request.
    ///
    /// It is never correct to implement `visit_byte_buf` without implementing
    /// `visit_bytes`. Implement neither, both, or just `visit_bytes`.
    ///
    /// The default implementation forwards to `visit_bytes` and then drops the
    /// `Vec<u8>`.
    fn visit_byte_buf(self, v: Vec<u8>) -> Result<Self::Value, &'static str> {
        self.visit_bytes(&v)
    }
    // /// The input contains an optional that is absent.
    // ///
    // /// The default implementation fails with a type error.
    // fn visit_none(self) -> Result<Self::Value, &'static str> {
    //     Err(Error::invalid_type(Unexpected::Option, &self))
    // }
    // /// The input contains an optional that is present.
    // ///
    // /// The default implementation fails with a type error.
    // fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    // where
    //     D: Deserializer<'de>,
    // {
    //     let _ = deserializer;
    //     Err(Error::invalid_type(Unexpected::Option, &self))
    // }
    /// The input contains a unit `()`.
    ///
    /// The default implementation fails with a type error.
    fn visit_unit(self) -> Result<Self::Value, &'static str> {
        // Err(Error::invalid_type(Unexpected::Unit, &self))
        Err(Self::UNIT)
    }
    // /// The input contains a newtype struct.
    // ///
    // /// The content of the newtype struct may be read from the given
    // /// `Deserializer`.
    // ///
    // /// The default implementation fails with a type error.
    // fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    // where
    //     D: Deserializer<'de>,
    // {
    //     let _ = deserializer;
    //     Err(Error::invalid_type(Unexpected::NewtypeStruct, &self))
    // }
    // /// The input contains a sequence of elements.
    // ///
    // /// The default implementation fails with a type error.
    // fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    // where
    //     A: SeqAccess<'de>,
    // {
    //     let _ = seq;
    //     Err(Error::invalid_type(Unexpected::Seq, &self))
    // }
    // /// The input contains a key-value map.
    // ///
    // /// The default implementation fails with a type error.
    // fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    // where
    //     A: MapAccess<'de>,
    // {
    //     let _ = map;
    //     Err(Error::invalid_type(Unexpected::Map, &self))
    // }
    // /// The input contains an enum.
    // ///
    // /// The default implementation fails with a type error.
    // fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    // where
    //     A: EnumAccess<'de>,
    // {
    //     let _ = data;
    //     Err(Error::invalid_type(Unexpected::Enum, &self))
    // }

    const TY: &'static str;

    const BOOL: &'static str = "expected bool";
    const SIGNED_INTEGER: &'static str = "expected signed integer";
    const UNSIGNED_INTEGER: &'static str = "expected unsigned integer";
    const FLOAT: &'static str = "expected float";
    const CHAR: &'static str = "expected char";
    const STR: &'static str = "expected str";
    const BYTES: &'static str = "expected bytes";
    // const NONE: &'static str = "expected none";
    // const SOME: &'static str = "expected some";
    const UNIT: &'static str = "expected unit";
    // const NEWTYPE_STRUCT: &'static str = "expected newtype struct";
    // const LIST: &'static str = "expected list";
    // const OBJECT: &'static str = "expected object";
    // const ENUM: &'static str = "expected enum";
}

impl<'de, V> serde::de::Visitor<'de> for V
where
    V: Visitor<'de>,
{
    type Value = Self::Value;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.expecting(formatter)
    }
    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_bool(v)
    }
    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i8(v)
    }
    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i16(v)
    }
    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i32(v)
    }
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v)
    }
    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i128(v)
    }
    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u8(v)
    }
    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u16(v)
    }
    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u32(v)
    }
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u64(v)
    }
    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u128(v)
    }
    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_f32(v)
    }
    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_f64(v)
    }
    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_char(v)
    }
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(v)
    }
    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_borrowed_str(v)
    }
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_string(v)
    }
    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_bytes(v)
    }
    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_borrowed_bytes(v)
    }
    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_byte_buf(v)
    }
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_none()
    }
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.visit_some(deserializer)
    }
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_unit()
    }
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.visit_newtype_struct(deserializer)
    }
    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        self.visit_seq(seq)
    }
    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        self.visit_map(map)
    }
    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        self.visit_enum(data)
    }
}

pub struct DeserializeTracker<T>(pub Result<T, &'static str>);

// #[derive(Deserialize)]
// enum E {
//     A,
//     B,
// }
// #[derive(Deserialize)]
// struct S {
//     a: i32,
//     b: Option<bool>,
//     c: Vec<char>,
// }
