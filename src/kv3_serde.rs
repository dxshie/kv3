use serde::Deserialize;
use serde::{
    de::{self, DeserializeSeed, Deserializer, IntoDeserializer, MapAccess, Visitor},
    forward_to_deserialize_any,
};
use std::{collections::HashMap, fmt};

use crate::{parse_kv3, KV3Object, KV3Value};

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
            KV3Value::HexArray(arr) => {
                // TODO: this should be idealy a binary blob
                // not a hex array parsaed to a Int list
                let int_values: Vec<KV3Value> =
                    arr.into_iter().map(|v| KV3Value::Int(v as i64)).collect();
                visitor.visit_seq(KV3ValueSeqAccess {
                    iter: int_values.into_iter(),
                })
            }
            KV3Value::Object(obj) => visitor.visit_map(KV3ObjectMapAccess {
                iter: obj.fields.into_iter(),
                value: None,
            }),
            KV3Value::Null => visitor.visit_unit(),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq
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
pub fn serde_kv3<'de, T>(input: &'static str) -> Result<T, Box<dyn std::error::Error>>
where
    T: Deserialize<'de>,
{
    // Parse the KV3 data
    let (_, parsed_kv3) = parse_kv3(input)?;

    // Wrap the parsed KV3 data in KV3Object
    let kv3_object = KV3Object { fields: parsed_kv3 };

    // Deserialize directly into the target struct
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
