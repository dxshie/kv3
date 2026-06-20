use serde::ser::{
    self, Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant, Serializer,
};
use serde::Deserialize;
use serde::{
    de::{self, DeserializeSeed, Deserializer, IntoDeserializer, MapAccess, Visitor},
    forward_to_deserialize_any,
};
use std::{collections::HashMap, fmt};

use crate::parser::parse_kv3;
use crate::types::{KV3Object, KV3Value};

impl<'de> Deserializer<'de> for KV3Object {
    type Error = de::value::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(KV3ObjectMapAccess {
            iter: self.fields.into_iter(),
            value: None,
        })
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }
}

struct KV3ObjectMapAccess {
    iter: std::collections::hash_map::IntoIter<String, KV3Value>,
    value: Option<KV3Value>,
}

impl<'de> MapAccess<'de> for KV3ObjectMapAccess {
    type Error = de::value::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some((key, value)) = self.iter.next() {
            self.value = Some(value);
            // Wrap the result in Ok(Some(...))
            Ok(Some(seed.deserialize(key.into_deserializer())?))
        } else {
            // Return Ok(None) when there are no more keys
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        if let Some(value) = self.value.take() {
            seed.deserialize(value)
        } else {
            Err(de::Error::custom("Value is missing for KV3Object map"))
        }
    }
}
struct KV3ValueSeqAccess {
    iter: std::vec::IntoIter<KV3Value>,
}

impl<'de> serde::de::SeqAccess<'de> for KV3ValueSeqAccess {
    type Error = de::value::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if let Some(value) = self.iter.next() {
            seed.deserialize(value).map(Some)
        } else {
            Ok(None)
        }
    }
}

struct KV3ByteSeqAccess {
    iter: std::vec::IntoIter<u8>,
}

impl<'de> serde::de::SeqAccess<'de> for KV3ByteSeqAccess {
    type Error = de::value::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(byte) => seed.deserialize(byte.into_deserializer()).map(Some),
            None => Ok(None),
        }
    }
}

impl<'de> serde::Deserializer<'de> for KV3Value {
    type Error = de::value::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            KV3Value::Bool(b) => visitor.visit_bool(b),
            KV3Value::Int(i) => visitor.visit_i64(i),
            KV3Value::Double(d) => visitor.visit_f64(d),
            KV3Value::String(s) => visitor.visit_string(s),
            KV3Value::Array(arr) => {
                // Custom SeqAccess for KV3Value::Array
                visitor.visit_seq(KV3ValueSeqAccess {
                    iter: arr.into_iter(),
                })
            }
            KV3Value::HexArray(arr) => visitor.visit_seq(KV3ByteSeqAccess {
                iter: arr.into_iter(),
            }),
            KV3Value::Object(obj) => visitor.visit_map(KV3ObjectMapAccess {
                iter: obj.fields.into_iter(),
                value: None,
            }),
            KV3Value::Null => visitor.visit_unit(),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            KV3Value::HexArray(bytes) => visitor.visit_byte_buf(bytes),
            other => other.deserialize_any(visitor),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            KV3Value::HexArray(bytes) => visitor.visit_byte_buf(bytes),
            other => other.deserialize_any(visitor),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            KV3Value::Null => visitor.visit_none(),
            other => visitor.visit_some(other),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }
}

/// Parses your KV3 input data into a Rust structure.
///
/// # Example
///
/// ```rust
/// use serde::Deserialize;
/// use kv3::kv3_serde::serde_kv3;
///
/// #[derive(Debug, Deserialize)]
/// struct MyStruct {
///     name: String,
///     value: i32,
///     active: bool,
/// }
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let kv3_input = r#"
///     {
///         name = "Example"
///         value = 42
///         active = true
///     }
///     "#;
///
///     let my_struct: MyStruct = serde_kv3(kv3_input)?;
///     println!("{:?}", my_struct);
///     Ok(())
/// }
/// ```
///
/// This will output:
///
/// ```text
/// MyStruct { name: "Example", value: 42, active: true }
/// ```
///
pub fn serde_kv3<'de, T>(input: &'de str) -> Result<T, Box<dyn std::error::Error + 'de>>
where
    T: Deserialize<'de>,
{
    let parsed_kv3 = parse_kv3(input).map_err(|errs| {
        let rendered = errs
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        Box::<dyn std::error::Error>::from(rendered)
    })?;
    let kv3_object = KV3Object::new(parsed_kv3);
    let result: T = T::deserialize(kv3_object)?;
    Ok(result)
}

impl<'de> Deserialize<'de> for KV3Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct KV3ValueVisitor;

        impl<'de> Visitor<'de> for KV3ValueVisitor {
            type Value = KV3Value;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid KV3Value")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(KV3Value::Bool(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(KV3Value::Int(value))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(KV3Value::Double(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(KV3Value::String(value.to_string()))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut values = Vec::new();
                while let Some(value) = seq.next_element()? {
                    values.push(value);
                }
                Ok(KV3Value::Array(values))
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: de::MapAccess<'de>,
            {
                let mut fields = HashMap::new();
                while let Some((key, value)) = map.next_entry()? {
                    fields.insert(key, value);
                }
                Ok(KV3Value::Object(KV3Object { fields }))
            }
        }

        deserializer.deserialize_any(KV3ValueVisitor)
    }
}

// =============================================================================
// Serialization: Rust value -> KV3 text
// =============================================================================

/// Errors produced while serializing a Rust value into KV3 text.
#[derive(Debug)]
pub enum KV3SerError {
    /// `serde::ser::Error::custom` from a downstream `Serialize` impl.
    Custom(String),
    /// KV3 objects can only be keyed by strings.
    NonStringKey,
    /// A `u64` value didn't fit into KV3's signed 64-bit integer type.
    U64Overflow(u64),
    /// `to_kv3_string` was called with a value that doesn't serialize as a struct/map.
    TopLevelNotObject,
}

impl fmt::Display for KV3SerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KV3SerError::Custom(s) => f.write_str(s),
            KV3SerError::NonStringKey => f.write_str("KV3 map keys must be strings"),
            KV3SerError::U64Overflow(v) => {
                write!(f, "u64 value {} overflows KV3's i64 range", v)
            }
            KV3SerError::TopLevelNotObject => {
                f.write_str("top-level KV3 value must be a struct or map")
            }
        }
    }
}

impl std::error::Error for KV3SerError {}

impl ser::Error for KV3SerError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        KV3SerError::Custom(msg.to_string())
    }
}

/// Serialize `value` to a KV3 text string.
///
/// The top-level value must be a struct or map; primitives, arrays, and other
/// shapes can't be expressed standalone in KV3.
///
/// # Example
///
/// ```rust
/// use serde::Serialize;
/// use kv3::kv3_serde::to_kv3_string;
///
/// #[derive(Serialize)]
/// struct Cfg { name: String, value: i32 }
///
/// let out = to_kv3_string(&Cfg { name: "demo".into(), value: 7 }).unwrap();
/// assert!(out.starts_with('{') && out.ends_with('}'));
/// ```
pub fn to_kv3_string<T>(value: &T) -> Result<String, KV3SerError>
where
    T: Serialize + ?Sized,
{
    match value.serialize(KV3ValueSerializer)? {
        KV3Value::Object(obj) => Ok(obj.to_string()),
        _ => Err(KV3SerError::TopLevelNotObject),
    }
}

pub struct KV3ValueSerializer;

impl Serializer for KV3ValueSerializer {
    type Ok = KV3Value;
    type Error = KV3SerError;
    type SerializeSeq = SeqBuilder;
    type SerializeTuple = SeqBuilder;
    type SerializeTupleStruct = SeqBuilder;
    type SerializeTupleVariant = TupleVariantBuilder;
    type SerializeMap = MapBuilder;
    type SerializeStruct = MapBuilder;
    type SerializeStructVariant = StructVariantBuilder;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(KV3Value::Bool(v))
    }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(KV3Value::Int(v as i64))
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(KV3Value::Int(v as i64))
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(KV3Value::Int(v as i64))
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(KV3Value::Int(v))
    }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(KV3Value::Int(v as i64))
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(KV3Value::Int(v as i64))
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(KV3Value::Int(v as i64))
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        i64::try_from(v)
            .map(KV3Value::Int)
            .map_err(|_| KV3SerError::U64Overflow(v))
    }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(KV3Value::Double(v as f64))
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(KV3Value::Double(v))
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(KV3Value::String(v.to_string()))
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(KV3Value::String(v.to_string()))
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(KV3Value::HexArray(v.to_vec()))
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(KV3Value::Null)
    }
    fn serialize_some<T>(self, v: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        v.serialize(self)
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(KV3Value::Null)
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(KV3Value::Null)
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(KV3Value::String(variant.to_string()))
    }
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        v: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        v.serialize(self)
    }
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        v: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        let mut fields = HashMap::new();
        fields.insert(variant.to_string(), v.serialize(KV3ValueSerializer)?);
        Ok(KV3Value::Object(KV3Object::new(fields)))
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SeqBuilder { items: Vec::new() })
    }
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(TupleVariantBuilder {
            variant,
            items: Vec::new(),
        })
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(MapBuilder {
            entries: HashMap::new(),
            next_key: None,
        })
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(MapBuilder {
            entries: HashMap::new(),
            next_key: None,
        })
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(StructVariantBuilder {
            variant,
            fields: HashMap::new(),
        })
    }
}

pub struct SeqBuilder {
    items: Vec<KV3Value>,
}

impl SerializeSeq for SeqBuilder {
    type Ok = KV3Value;
    type Error = KV3SerError;
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.items.push(value.serialize(KV3ValueSerializer)?);
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(KV3Value::Array(self.items))
    }
}

impl SerializeTuple for SeqBuilder {
    type Ok = KV3Value;
    type Error = KV3SerError;
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl SerializeTupleStruct for SeqBuilder {
    type Ok = KV3Value;
    type Error = KV3SerError;
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

pub struct TupleVariantBuilder {
    variant: &'static str,
    items: Vec<KV3Value>,
}

impl SerializeTupleVariant for TupleVariantBuilder {
    type Ok = KV3Value;
    type Error = KV3SerError;
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.items.push(value.serialize(KV3ValueSerializer)?);
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut fields = HashMap::new();
        fields.insert(self.variant.to_string(), KV3Value::Array(self.items));
        Ok(KV3Value::Object(KV3Object::new(fields)))
    }
}

pub struct MapBuilder {
    entries: HashMap<String, KV3Value>,
    next_key: Option<String>,
}

impl SerializeMap for MapBuilder {
    type Ok = KV3Value;
    type Error = KV3SerError;
    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.next_key = Some(key.serialize(KeySerializer)?);
        Ok(())
    }
    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        let key = self.next_key.take().ok_or_else(|| {
            KV3SerError::Custom("serialize_value called before serialize_key".into())
        })?;
        self.entries
            .insert(key, value.serialize(KV3ValueSerializer)?);
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(KV3Value::Object(KV3Object::new(self.entries)))
    }
}

impl SerializeStruct for MapBuilder {
    type Ok = KV3Value;
    type Error = KV3SerError;
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.entries
            .insert(key.to_string(), value.serialize(KV3ValueSerializer)?);
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(KV3Value::Object(KV3Object::new(self.entries)))
    }
}

pub struct StructVariantBuilder {
    variant: &'static str,
    fields: HashMap<String, KV3Value>,
}

impl SerializeStructVariant for StructVariantBuilder {
    type Ok = KV3Value;
    type Error = KV3SerError;
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.fields
            .insert(key.to_string(), value.serialize(KV3ValueSerializer)?);
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut outer = HashMap::new();
        outer.insert(
            self.variant.to_string(),
            KV3Value::Object(KV3Object::new(self.fields)),
        );
        Ok(KV3Value::Object(KV3Object::new(outer)))
    }
}

/// Serializer that only accepts string-like values, used for map keys.
struct KeySerializer;

impl Serializer for KeySerializer {
    type Ok = String;
    type Error = KV3SerError;
    type SerializeSeq = ser::Impossible<String, KV3SerError>;
    type SerializeTuple = ser::Impossible<String, KV3SerError>;
    type SerializeTupleStruct = ser::Impossible<String, KV3SerError>;
    type SerializeTupleVariant = ser::Impossible<String, KV3SerError>;
    type SerializeMap = ser::Impossible<String, KV3SerError>;
    type SerializeStruct = ser::Impossible<String, KV3SerError>;
    type SerializeStructVariant = ser::Impossible<String, KV3SerError>;

    fn serialize_str(self, v: &str) -> Result<String, KV3SerError> {
        Ok(v.to_string())
    }
    fn serialize_char(self, v: char) -> Result<String, KV3SerError> {
        Ok(v.to_string())
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
    ) -> Result<String, KV3SerError> {
        Ok(variant.to_string())
    }
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<String, KV3SerError>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_bool(self, _v: bool) -> Result<String, KV3SerError> {
        Err(KV3SerError::NonStringKey)
    }
    fn serialize_i8(self, _v: i8) -> Result<String, KV3SerError> {
        Err(KV3SerError::NonStringKey)
    }
    fn serialize_i16(self, _v: i16) -> Result<String, KV3SerError> {
        Err(KV3SerError::NonStringKey)
    }
    fn serialize_i32(self, _v: i32) -> Result<String, KV3SerError> {
        Err(KV3SerError::NonStringKey)
    }
    fn serialize_i64(self, _v: i64) -> Result<String, KV3SerError> {
        Err(KV3SerError::NonStringKey)
    }
    fn serialize_u8(self, _v: u8) -> Result<String, KV3SerError> {
        Err(KV3SerError::NonStringKey)
    }
    fn serialize_u16(self, _v: u16) -> Result<String, KV3SerError> {
        Err(KV3SerError::NonStringKey)
    }
    fn serialize_u32(self, _v: u32) -> Result<String, KV3SerError> {
        Err(KV3SerError::NonStringKey)
    }
    fn serialize_u64(self, _v: u64) -> Result<String, KV3SerError> {
        Err(KV3SerError::NonStringKey)
    }
    fn serialize_f32(self, _v: f32) -> Result<String, KV3SerError> {
        Err(KV3SerError::NonStringKey)
    }
    fn serialize_f64(self, _v: f64) -> Result<String, KV3SerError> {
        Err(KV3SerError::NonStringKey)
    }
    fn serialize_bytes(self, _v: &[u8]) -> Result<String, KV3SerError> {
        Err(KV3SerError::NonStringKey)
    }
    fn serialize_none(self) -> Result<String, KV3SerError> {
        Err(KV3SerError::NonStringKey)
    }
    fn serialize_some<T>(self, _v: &T) -> Result<String, KV3SerError>
    where
        T: Serialize + ?Sized,
    {
        Err(KV3SerError::NonStringKey)
    }
    fn serialize_unit(self) -> Result<String, KV3SerError> {
        Err(KV3SerError::NonStringKey)
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<String, KV3SerError> {
        Err(KV3SerError::NonStringKey)
    }
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<String, KV3SerError>
    where
        T: Serialize + ?Sized,
    {
        Err(KV3SerError::NonStringKey)
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, KV3SerError> {
        Err(KV3SerError::NonStringKey)
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, KV3SerError> {
        Err(KV3SerError::NonStringKey)
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, KV3SerError> {
        Err(KV3SerError::NonStringKey)
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, KV3SerError> {
        Err(KV3SerError::NonStringKey)
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, KV3SerError> {
        Err(KV3SerError::NonStringKey)
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, KV3SerError> {
        Err(KV3SerError::NonStringKey)
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, KV3SerError> {
        Err(KV3SerError::NonStringKey)
    }
}
