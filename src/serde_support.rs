//! Serde support for Edn data format.
//!
//! This module provides seamless integration with the serde ecosystem,
//! allowing easy conversion between Rust structs and Edn values.
//!
//! # Key Type Distinction
//!
//! **This implementation makes an important distinction between struct fields and map keys:**
//!
//! - **Struct fields** use `Tag` (`:field_name`) - representing finite, enumerable identifiers
//! - **Map keys** use `String` (`"key"`) - representing arbitrary string data
//!
//! This design preserves the semantic difference between:
//! - **Tags**: Named constants like struct field names, enum variants, or record keys
//! - **Strings**: Dynamic text data that can be any value
//!
//! # Basic Usage
//!
//! ```rust
//! use cirru_edn::{to_edn, from_edn};
//! use serde::{Serialize, Deserialize};
//! use std::collections::HashMap;
//!
//! #[derive(Serialize, Deserialize, Debug, PartialEq)]
//! struct Person {
//!     name: String,                    // Will become :name (Tag)
//!     age: u32,                        // Will become :age (Tag)
//!     metadata: HashMap<String, String>, // Keys will be "key" (String)
//! }
//!
//! let person = Person {
//!     name: "Alice".to_string(),
//!     age: 30,
//!     metadata: [("role".to_string(), "admin".to_string())].into_iter().collect(),
//! };
//!
//! // Serialize to Edn
//! let edn_value = to_edn(&person).unwrap();
//! // Results in: {:name "Alice", :age 30, :metadata {"role" "admin"}}
//! //              ^^^^^ Tag                          ^^^^^^ String
//!
//! // Deserialize from Edn
//! let recovered: Person = from_edn(edn_value).unwrap();
//! assert_eq!(person, recovered);
//! ```
//!
//! # Type Mapping
//!
//! - Rust `Option<T>` maps to either `Edn::Nil` or the contained value
//! - Rust `Vec<T>` maps to `Edn::List`
//! - Rust `HashMap<K, V>` maps to `Edn::Map` (keys become Strings)
//! - Rust `HashSet<T>` maps to `Edn::Set` (with special encoding)
//! - Primitive types map directly to their Edn equivalents
//! - Struct fields map to Tags in `Edn::Map`
//!
//! # Special Encodings
//!
//! Some Edn types that don't have direct serde equivalents are encoded as maps:
//! - `Symbol` -> `{"__edn_symbol": "value"}`
//! - `Tag` -> `{"__edn_tag": "value"}`
//! - `Set` -> `{"__edn_set": [items]}`
//! - `Buffer` -> `{"__edn_buffer": [bytes]}`
//! - `Tuple` -> `{"__edn_tuple_tag": tag, "__edn_tuple_extra": [values]}`

#![allow(clippy::mutable_key_type)]
#![allow(clippy::uninlined_format_args)]

use std::collections::HashMap;
use std::sync::Arc;

use serde::{
  de::{self, MapAccess, SeqAccess, Visitor},
  ser::{
    self, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple, SerializeTupleStruct,
    SerializeTupleVariant,
  },
  Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{Edn, EdnListView, EdnMapView, EdnRecordView, EdnSetView, EdnTag, EdnTupleView};
use cirru_parser::Cirru;

impl Serialize for Edn {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    match self {
      Edn::Nil => serializer.serialize_unit(),
      Edn::Bool(b) => serializer.serialize_bool(*b),
      Edn::Number(n) => serializer.serialize_f64(*n),
      Edn::Symbol(s) => {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("__edn_symbol", s.as_ref())?;
        map.end()
      }
      Edn::Tag(tag) => {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("__edn_tag", &tag.to_string())?;
        map.end()
      }
      Edn::Str(s) => serializer.serialize_str(s),
      Edn::Quote(cirru) => {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("__edn_quote", cirru)?;
        map.end()
      }
      Edn::Tuple(EdnTupleView { tag, extra }) => {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("__edn_tuple_tag", tag.as_ref())?;
        map.serialize_entry("__edn_tuple_extra", extra)?;
        map.end()
      }
      Edn::List(EdnListView(list)) => {
        let mut seq = serializer.serialize_seq(Some(list.len()))?;
        for item in list {
          seq.serialize_element(item)?;
        }
        seq.end()
      }
      Edn::Set(EdnSetView(set)) => {
        let mut map = serializer.serialize_map(Some(1))?;
        let items: Vec<&Edn> = set.iter().collect();
        map.serialize_entry("__edn_set", &items)?;
        map.end()
      }
      Edn::Map(EdnMapView(map)) => {
        let mut ser_map = serializer.serialize_map(Some(map.len()))?;
        for (k, v) in map {
          // For simple string keys, serialize directly
          if let Edn::Str(s) = k {
            ser_map.serialize_entry(s.as_ref(), v)?;
          } else {
            // For complex keys, convert to string representation
            let key_str = format!("{}", k);
            ser_map.serialize_entry(&key_str, v)?;
          }
        }
        ser_map.end()
      }
      Edn::Record(EdnRecordView { tag, pairs }) => {
        let mut map = serializer.serialize_map(Some(pairs.len() + 1))?;
        map.serialize_entry("__edn_record_tag", &tag.to_string())?;
        for (key, value) in pairs {
          map.serialize_entry(&key.to_string(), value)?;
        }
        map.end()
      }
      Edn::Buffer(buf) => {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("__edn_buffer", buf)?;
        map.end()
      }
      Edn::AnyRef(_) => Err(ser::Error::custom("AnyRef type cannot be serialized")),
      Edn::Atom(boxed) => {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("__edn_atom", boxed.as_ref())?;
        map.end()
      }
    }
  }
}

impl<'de> Deserialize<'de> for Edn {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    struct EdnVisitor;

    impl<'de> Visitor<'de> for EdnVisitor {
      type Value = Edn;

      fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("any valid Edn value")
      }

      fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(Edn::Bool(value))
      }

      fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(Edn::Number(value as f64))
      }

      fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(Edn::Number(value as f64))
      }

      fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E> {
        Ok(Edn::Number(value as f64))
      }

      fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E> {
        Ok(Edn::Number(value as f64))
      }

      fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E> {
        Ok(Edn::Number(value))
      }

      fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(Edn::Str(value.into()))
      }

      fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(Edn::Str(value.into()))
      }

      fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(Edn::Nil)
      }

      fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(Edn::Nil)
      }

      fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
      where
        D: Deserializer<'de>,
      {
        Edn::deserialize(deserializer)
      }

      fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
      where
        A: SeqAccess<'de>,
      {
        let mut list = Vec::new();
        while let Some(value) = seq.next_element()? {
          list.push(value);
        }
        Ok(Edn::List(EdnListView(list)))
      }

      fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
      where
        A: MapAccess<'de>,
      {
        let mut result_map = HashMap::new();
        let mut is_special = false;
        let mut special_type: Option<String> = None;
        let mut special_data = HashMap::new();

        while let Some(key) = map.next_key::<String>()? {
          if key.starts_with("__edn_") {
            is_special = true;
            if special_type.is_none() {
              special_type = Some(key.clone());
            }
            let value: Edn = map.next_value()?;
            special_data.insert(key, value);
          } else {
            let value: Edn = map.next_value()?;
            result_map.insert(Edn::Str(key.into()), value);
          }
        }

        if is_special {
          if let Some(ref stype) = special_type {
            match stype.as_str() {
              "__edn_symbol" => {
                if let Some(Edn::Str(s)) = special_data.get("__edn_symbol") {
                  Ok(Edn::Symbol(s.clone()))
                } else {
                  Err(de::Error::custom("Invalid symbol data"))
                }
              }
              "__edn_tag" => {
                if let Some(Edn::Str(s)) = special_data.get("__edn_tag") {
                  Ok(Edn::Tag(EdnTag::new(s.as_ref())))
                } else {
                  Err(de::Error::custom("Invalid tag data"))
                }
              }
              "__edn_tuple_tag" => {
                if let (Some(tag), Some(extra)) = (
                  special_data.get("__edn_tuple_tag"),
                  special_data.get("__edn_tuple_extra"),
                ) {
                  if let Edn::List(EdnListView(extra_vec)) = extra {
                    Ok(Edn::Tuple(EdnTupleView {
                      tag: Arc::new(tag.clone()),
                      extra: extra_vec.clone(),
                    }))
                  } else {
                    Err(de::Error::custom("Invalid tuple extra data"))
                  }
                } else {
                  Err(de::Error::custom("Invalid tuple data"))
                }
              }
              "__edn_set" => {
                if let Some(Edn::List(EdnListView(items))) = special_data.get("__edn_set") {
                  let set = items.iter().cloned().collect();
                  Ok(Edn::Set(EdnSetView(set)))
                } else {
                  Err(de::Error::custom("Invalid set data"))
                }
              }
              "__edn_buffer" => {
                if let Some(Edn::List(EdnListView(items))) = special_data.get("__edn_buffer") {
                  let mut buffer = Vec::new();
                  for item in items {
                    if let Edn::Number(n) = item {
                      buffer.push(*n as u8);
                    } else {
                      return Err(de::Error::custom("Invalid buffer data"));
                    }
                  }
                  Ok(Edn::Buffer(buffer))
                } else {
                  Err(de::Error::custom("Invalid buffer data"))
                }
              }
              "__edn_record_tag" => {
                if let Some(Edn::Str(tag_str)) = special_data.get("__edn_record_tag") {
                  let tag = EdnTag::new(tag_str.as_ref());
                  let mut pairs = Vec::new();
                  for (k, v) in &result_map {
                    if let Edn::Str(key_str) = k {
                      pairs.push((EdnTag::new(key_str.as_ref()), v.clone()));
                    }
                  }
                  Ok(Edn::Record(EdnRecordView { tag, pairs }))
                } else {
                  Err(de::Error::custom("Invalid record tag"))
                }
              }
              "__edn_atom" => {
                if let Some(value) = special_data.get("__edn_atom") {
                  Ok(Edn::Atom(Box::new(value.clone())))
                } else {
                  Err(de::Error::custom("Invalid atom data"))
                }
              }
              "__edn_quote" => {
                if let Some(cirru_data) = special_data.get("__edn_quote") {
                  // 直接从Edn反序列化为Cirru，而不是通过自定义转换函数
                  let cirru = from_edn::<Cirru>(cirru_data.clone())
                    .map_err(|e| de::Error::custom(format!("Invalid quote data: {}", e)))?;
                  Ok(Edn::Quote(cirru))
                } else {
                  Err(de::Error::custom("Invalid quote data"))
                }
              }
              _ => Err(de::Error::custom(format!("Unknown special type: {}", stype))),
            }
          } else {
            Err(de::Error::custom("No special type found"))
          }
        } else {
          Ok(Edn::Map(EdnMapView(result_map)))
        }
      }
    }

    deserializer.deserialize_any(EdnVisitor)
  }
}

/// Convert a `T` where `T` implements `Serialize` to `Edn`.
///
/// This function directly serializes any serializable type to the `Edn` format.
/// Struct fields become Tags (`:field_name`) and map keys become Strings (`"key"`).
///
/// # Examples
///
/// ```
/// use serde::Serialize;
/// use cirru_edn::to_edn;
///
/// #[derive(Serialize)]
/// struct Config {
///     debug: bool,
///     port: u16,
/// }
///
/// let config = Config { debug: true, port: 8080 };
/// let edn_value = to_edn(&config).unwrap();
/// // Results in: {:debug true, :port 8080}
/// ```
pub fn to_edn<T>(value: T) -> Result<Edn, String>
where
  T: Serialize,
{
  // Serialize directly to Edn using custom serializer
  value.serialize(EdnSerializer).map_err(|e| e.to_string())
}

/// Convert an `Edn` to a `T` where `T` implements `Deserialize`.
///
/// This function directly deserializes an `Edn` value to any deserializable type.
/// Tags (`:field_name`) are used for struct fields and Strings (`"key"`) for map keys.
///
/// # Examples
///
/// ```
/// use serde::Deserialize;
/// use cirru_edn::{from_edn, Edn, EdnTag, EdnMapView};
/// use std::collections::HashMap;
///
/// #[derive(Deserialize)]
/// struct Config {
///     debug: bool,
///     port: u16,
/// }
///
/// // Create EDN manually
/// let mut map = HashMap::new();
/// map.insert(Edn::Tag(EdnTag::new("debug")), true.into());
/// map.insert(Edn::Tag(EdnTag::new("port")), 8080.into());
/// let edn_map = Edn::Map(EdnMapView(map));
///
/// let config: Config = from_edn(edn_map).unwrap();
/// ```
pub fn from_edn<T>(value: Edn) -> Result<T, String>
where
  T: for<'de> Deserialize<'de>,
{
  // Deserialize directly from Edn using custom deserializer
  T::deserialize(EdnDeserializer::new(value)).map_err(|e| e.to_string())
}

// Custom Edn Serializer
struct EdnSerializer;

#[derive(Debug)]
struct EdnSerializerError(String);

impl std::fmt::Display for EdnSerializerError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl std::error::Error for EdnSerializerError {}

impl ser::Error for EdnSerializerError {
  fn custom<T: std::fmt::Display>(msg: T) -> Self {
    EdnSerializerError(msg.to_string())
  }
}

impl Serializer for EdnSerializer {
  type Ok = Edn;
  type Error = EdnSerializerError;

  type SerializeSeq = EdnSeqSerializer;
  type SerializeTuple = EdnSeqSerializer;
  type SerializeTupleStruct = EdnSeqSerializer;
  type SerializeTupleVariant = EdnSeqSerializer;
  type SerializeMap = EdnMapSerializer;
  type SerializeStruct = EdnMapSerializer;
  type SerializeStructVariant = EdnMapSerializer;

  fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
    Ok(Edn::Bool(v))
  }

  fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
    Ok(Edn::Number(v as f64))
  }

  fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
    Ok(Edn::Number(v as f64))
  }

  fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
    Ok(Edn::Number(v as f64))
  }

  fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
    Ok(Edn::Number(v as f64))
  }

  fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
    Ok(Edn::Number(v as f64))
  }

  fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
    Ok(Edn::Number(v as f64))
  }

  fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
    Ok(Edn::Number(v as f64))
  }

  fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
    Ok(Edn::Number(v as f64))
  }

  fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
    Ok(Edn::Number(v as f64))
  }

  fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
    Ok(Edn::Number(v))
  }

  fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
    Ok(Edn::Str(v.to_string().into()))
  }

  fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
    Ok(Edn::Str(v.into()))
  }

  fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
    Ok(Edn::Buffer(v.to_vec()))
  }

  fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
    Ok(Edn::Nil)
  }

  fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
  where
    T: ?Sized + Serialize,
  {
    value.serialize(self)
  }

  fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
    Ok(Edn::Nil)
  }

  fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
    Ok(Edn::Nil)
  }

  fn serialize_unit_variant(
    self,
    _name: &'static str,
    _variant_index: u32,
    variant: &'static str,
  ) -> Result<Self::Ok, Self::Error> {
    Ok(Edn::Str(variant.into()))
  }

  fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
  where
    T: ?Sized + Serialize,
  {
    value.serialize(self)
  }

  fn serialize_newtype_variant<T>(
    self,
    _name: &'static str,
    _variant_index: u32,
    variant: &'static str,
    value: &T,
  ) -> Result<Self::Ok, Self::Error>
  where
    T: ?Sized + Serialize,
  {
    let mut map = HashMap::new();
    map.insert(Edn::Str(variant.into()), value.serialize(self)?);
    Ok(Edn::Map(EdnMapView(map)))
  }

  fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
    Ok(EdnSeqSerializer {
      items: Vec::with_capacity(len.unwrap_or(0)),
    })
  }

  fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
    self.serialize_seq(Some(len))
  }

  fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeTupleStruct, Self::Error> {
    self.serialize_seq(Some(len))
  }

  fn serialize_tuple_variant(
    self,
    _name: &'static str,
    _variant_index: u32,
    _variant: &'static str,
    len: usize,
  ) -> Result<Self::SerializeTupleVariant, Self::Error> {
    // For tuple variants, we'll create a map with the variant name as key
    Ok(EdnSeqSerializer {
      items: Vec::with_capacity(len + 1),
    })
  }

  fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
    Ok(EdnMapSerializer {
      map: HashMap::with_capacity(len.unwrap_or(0)),
      next_key: None,
    })
  }

  fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct, Self::Error> {
    self.serialize_map(Some(len))
  }

  fn serialize_struct_variant(
    self,
    _name: &'static str,
    _variant_index: u32,
    variant: &'static str,
    len: usize,
  ) -> Result<Self::SerializeStructVariant, Self::Error> {
    let mut serializer = self.serialize_map(Some(len + 1))?;
    serializer
      .map
      .insert(Edn::Str("__variant".into()), Edn::Str(variant.into()));
    Ok(serializer)
  }
}

struct EdnSeqSerializer {
  items: Vec<Edn>,
}

impl SerializeSeq for EdnSeqSerializer {
  type Ok = Edn;
  type Error = EdnSerializerError;

  fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
  where
    T: ?Sized + Serialize,
  {
    self.items.push(value.serialize(EdnSerializer)?);
    Ok(())
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    Ok(Edn::List(EdnListView(self.items)))
  }
}

impl SerializeTuple for EdnSeqSerializer {
  type Ok = Edn;
  type Error = EdnSerializerError;

  fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
  where
    T: ?Sized + Serialize,
  {
    SerializeSeq::serialize_element(self, value)
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    SerializeSeq::end(self)
  }
}

impl SerializeTupleStruct for EdnSeqSerializer {
  type Ok = Edn;
  type Error = EdnSerializerError;

  fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
  where
    T: ?Sized + Serialize,
  {
    SerializeSeq::serialize_element(self, value)
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    SerializeSeq::end(self)
  }
}

impl SerializeTupleVariant for EdnSeqSerializer {
  type Ok = Edn;
  type Error = EdnSerializerError;

  fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
  where
    T: ?Sized + Serialize,
  {
    SerializeSeq::serialize_element(self, value)
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    SerializeSeq::end(self)
  }
}

struct EdnMapSerializer {
  map: HashMap<Edn, Edn>,
  next_key: Option<Edn>,
}

impl SerializeMap for EdnMapSerializer {
  type Ok = Edn;
  type Error = EdnSerializerError;

  fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
  where
    T: ?Sized + Serialize,
  {
    self.next_key = Some(key.serialize(EdnSerializer)?);
    Ok(())
  }

  fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
  where
    T: ?Sized + Serialize,
  {
    let key = self
      .next_key
      .take()
      .ok_or_else(|| EdnSerializerError("serialize_value called before serialize_key".to_string()))?;
    self.map.insert(key, value.serialize(EdnSerializer)?);
    Ok(())
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    Ok(Edn::Map(EdnMapView(self.map)))
  }
}

impl SerializeStruct for EdnMapSerializer {
  type Ok = Edn;
  type Error = EdnSerializerError;

  fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
  where
    T: ?Sized + Serialize,
  {
    // Use Tag for struct field keys to distinguish from Map string keys
    self
      .map
      .insert(Edn::Tag(EdnTag::new(key)), value.serialize(EdnSerializer)?);
    Ok(())
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    Ok(Edn::Map(EdnMapView(self.map)))
  }
}

impl SerializeStructVariant for EdnMapSerializer {
  type Ok = Edn;
  type Error = EdnSerializerError;

  fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
  where
    T: ?Sized + Serialize,
  {
    SerializeStruct::serialize_field(self, key, value)
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    SerializeStruct::end(self)
  }
}

// Custom Edn Deserializer
struct EdnDeserializer {
  value: Edn,
}

impl EdnDeserializer {
  fn new(value: Edn) -> Self {
    EdnDeserializer { value }
  }
}

#[derive(Debug)]
struct EdnDeserializerError(String);

impl std::fmt::Display for EdnDeserializerError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl std::error::Error for EdnDeserializerError {}

impl de::Error for EdnDeserializerError {
  fn custom<T: std::fmt::Display>(msg: T) -> Self {
    EdnDeserializerError(msg.to_string())
  }
}

impl<'de> Deserializer<'de> for EdnDeserializer {
  type Error = EdnDeserializerError;

  fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    match self.value {
      Edn::Nil => visitor.visit_unit(),
      Edn::Bool(b) => visitor.visit_bool(b),
      Edn::Number(n) => {
        if n.fract().abs() < f64::EPSILON && n >= i64::MIN as f64 && n <= i64::MAX as f64 {
          visitor.visit_i64(n as i64)
        } else {
          visitor.visit_f64(n)
        }
      }
      Edn::Str(s) => visitor.visit_str(s.as_ref()),
      Edn::List(EdnListView(items)) => visitor.visit_seq(EdnSeqDeserializer::new(items.into_iter())),
      Edn::Map(EdnMapView(map)) => visitor.visit_map(EdnMapDeserializer::new(map.into_iter())),
      _ => Err(EdnDeserializerError(format!(
        "Cannot deserialize Edn type: {:?}",
        self.value
      ))),
    }
  }

  fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    match self.value {
      Edn::Bool(b) => visitor.visit_bool(b),
      _ => Err(EdnDeserializerError("Expected boolean".to_string())),
    }
  }

  fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    self.deserialize_i64(visitor)
  }

  fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    self.deserialize_i64(visitor)
  }

  fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    self.deserialize_i64(visitor)
  }

  fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    match self.value {
      Edn::Number(n) => visitor.visit_i64(n as i64),
      _ => Err(EdnDeserializerError("Expected number".to_string())),
    }
  }

  fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    self.deserialize_u64(visitor)
  }

  fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    self.deserialize_u64(visitor)
  }

  fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    self.deserialize_u64(visitor)
  }

  fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    match self.value {
      Edn::Number(n) => visitor.visit_u64(n as u64),
      _ => Err(EdnDeserializerError("Expected number".to_string())),
    }
  }

  fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    self.deserialize_f64(visitor)
  }

  fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    match self.value {
      Edn::Number(n) => visitor.visit_f64(n),
      _ => Err(EdnDeserializerError("Expected number".to_string())),
    }
  }

  fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    self.deserialize_str(visitor)
  }

  fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    match self.value {
      Edn::Str(s) => visitor.visit_str(s.as_ref()),
      // Support Tag as string for struct field keys
      Edn::Tag(tag) => visitor.visit_str(tag.0.as_ref()),
      _ => Err(EdnDeserializerError("Expected string or tag".to_string())),
    }
  }

  fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    self.deserialize_str(visitor)
  }

  fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    match self.value {
      Edn::Buffer(buf) => visitor.visit_bytes(&buf),
      _ => Err(EdnDeserializerError("Expected buffer".to_string())),
    }
  }

  fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    self.deserialize_bytes(visitor)
  }

  fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    match self.value {
      Edn::Nil => visitor.visit_none(),
      _ => visitor.visit_some(self),
    }
  }

  fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    match self.value {
      Edn::Nil => visitor.visit_unit(),
      _ => Err(EdnDeserializerError("Expected nil".to_string())),
    }
  }

  fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    self.deserialize_unit(visitor)
  }

  fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    visitor.visit_newtype_struct(self)
  }

  fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    match self.value {
      Edn::List(EdnListView(items)) => visitor.visit_seq(EdnSeqDeserializer::new(items.into_iter())),
      _ => Err(EdnDeserializerError("Expected list".to_string())),
    }
  }

  fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    self.deserialize_seq(visitor)
  }

  fn deserialize_tuple_struct<V>(self, _name: &'static str, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    self.deserialize_seq(visitor)
  }

  fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    match self.value {
      Edn::Map(EdnMapView(map)) => visitor.visit_map(EdnMapDeserializer::new(map.into_iter())),
      _ => Err(EdnDeserializerError("Expected map".to_string())),
    }
  }

  fn deserialize_struct<V>(
    self,
    _name: &'static str,
    _fields: &'static [&'static str],
    visitor: V,
  ) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    self.deserialize_map(visitor)
  }

  fn deserialize_enum<V>(
    self,
    _name: &'static str,
    _variants: &'static [&'static str],
    visitor: V,
  ) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    match self.value {
      Edn::Str(s) => visitor.visit_enum(EdnEnumDeserializer::new(s.as_ref().to_string(), None)),
      Edn::Map(EdnMapView(map)) => {
        if map.len() == 1 {
          let (key, value) = map.into_iter().next().unwrap();
          if let Edn::Str(variant_name) = key {
            visitor.visit_enum(EdnEnumDeserializer::new(variant_name.as_ref().to_string(), Some(value)))
          } else {
            Err(EdnDeserializerError("Expected string key for enum variant".to_string()))
          }
        } else {
          Err(EdnDeserializerError("Expected single-entry map for enum".to_string()))
        }
      }
      _ => Err(EdnDeserializerError("Expected string or map for enum".to_string())),
    }
  }

  fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    self.deserialize_str(visitor)
  }

  fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    visitor.visit_unit()
  }
}

struct EdnSeqDeserializer {
  iter: std::vec::IntoIter<Edn>,
}

impl EdnSeqDeserializer {
  fn new(iter: std::vec::IntoIter<Edn>) -> Self {
    EdnSeqDeserializer { iter }
  }
}

impl<'de> SeqAccess<'de> for EdnSeqDeserializer {
  type Error = EdnDeserializerError;

  fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
  where
    T: de::DeserializeSeed<'de>,
  {
    match self.iter.next() {
      Some(value) => seed.deserialize(EdnDeserializer::new(value)).map(Some),
      None => Ok(None),
    }
  }
}

struct EdnMapDeserializer {
  iter: std::collections::hash_map::IntoIter<Edn, Edn>,
  current_value: Option<Edn>,
}

impl EdnMapDeserializer {
  fn new(iter: std::collections::hash_map::IntoIter<Edn, Edn>) -> Self {
    EdnMapDeserializer {
      iter,
      current_value: None,
    }
  }
}

impl<'de> MapAccess<'de> for EdnMapDeserializer {
  type Error = EdnDeserializerError;

  fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
  where
    K: de::DeserializeSeed<'de>,
  {
    match self.iter.next() {
      Some((key, value)) => {
        self.current_value = Some(value);
        seed.deserialize(EdnDeserializer::new(key)).map(Some)
      }
      None => Ok(None),
    }
  }

  fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
  where
    V: de::DeserializeSeed<'de>,
  {
    match self.current_value.take() {
      Some(value) => seed.deserialize(EdnDeserializer::new(value)),
      None => Err(EdnDeserializerError(
        "next_value_seed called before next_key_seed".to_string(),
      )),
    }
  }
}

struct EdnEnumDeserializer {
  variant: String,
  value: Option<Edn>,
}

impl EdnEnumDeserializer {
  fn new(variant: String, value: Option<Edn>) -> Self {
    EdnEnumDeserializer { variant, value }
  }
}

impl<'de> de::EnumAccess<'de> for EdnEnumDeserializer {
  type Error = EdnDeserializerError;
  type Variant = EdnVariantDeserializer;

  fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
  where
    V: de::DeserializeSeed<'de>,
  {
    let variant_deserializer = EdnDeserializer::new(Edn::Str(self.variant.into()));
    let variant = seed.deserialize(variant_deserializer)?;
    Ok((variant, EdnVariantDeserializer { value: self.value }))
  }
}

struct EdnVariantDeserializer {
  value: Option<Edn>,
}

impl<'de> de::VariantAccess<'de> for EdnVariantDeserializer {
  type Error = EdnDeserializerError;

  fn unit_variant(self) -> Result<(), Self::Error> {
    match self.value {
      Some(_) => Err(EdnDeserializerError("Expected unit variant".to_string())),
      None => Ok(()),
    }
  }

  fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
  where
    T: de::DeserializeSeed<'de>,
  {
    match self.value {
      Some(value) => seed.deserialize(EdnDeserializer::new(value)),
      None => Err(EdnDeserializerError("Expected newtype variant".to_string())),
    }
  }

  fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    match self.value {
      Some(Edn::List(EdnListView(items))) => visitor.visit_seq(EdnSeqDeserializer::new(items.into_iter())),
      Some(_) => Err(EdnDeserializerError("Expected list for tuple variant".to_string())),
      None => Err(EdnDeserializerError("Expected tuple variant".to_string())),
    }
  }

  fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    match self.value {
      Some(Edn::Map(EdnMapView(map))) => visitor.visit_map(EdnMapDeserializer::new(map.into_iter())),
      Some(_) => Err(EdnDeserializerError("Expected map for struct variant".to_string())),
      None => Err(EdnDeserializerError("Expected struct variant".to_string())),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde::{Deserialize, Serialize};
  use std::collections::HashMap;

  #[derive(Debug, Serialize, Deserialize, PartialEq)]
  struct TestStruct {
    name: String,
    age: u32,
    is_active: bool,
    scores: Vec<f64>,
    metadata: HashMap<String, String>,
  }

  #[test]
  fn test_to_edn() {
    let test_data = TestStruct {
      name: "Alice".to_string(),
      age: 30,
      is_active: true,
      scores: vec![85.5, 92.0, 78.5],
      metadata: [("role".to_string(), "admin".to_string())].into_iter().collect(),
    };

    let edn_value = to_edn(&test_data).unwrap();

    // Verify the conversion - struct fields should now use Tags
    if let Edn::Map(map) = edn_value {
      assert!(map.0.contains_key(&Edn::Tag(EdnTag::new("name"))));
      assert!(map.0.contains_key(&Edn::Tag(EdnTag::new("age"))));
    } else {
      panic!("Expected Edn::Map");
    }
  }

  #[test]
  fn test_from_edn() {
    // Use Tags for struct field keys instead of Strings
    let edn_map = Edn::Map(EdnMapView({
      let mut map = HashMap::new();
      map.insert(Edn::Tag(EdnTag::new("name")), "Bob".into());
      map.insert(Edn::Tag(EdnTag::new("age")), 25.into());
      map.insert(Edn::Tag(EdnTag::new("is_active")), false.into());
      map.insert(Edn::Tag(EdnTag::new("scores")), vec![90.0, 88.5].into());
      map.insert(Edn::Tag(EdnTag::new("metadata")), {
        let mut meta = HashMap::new();
        meta.insert(Edn::Str("role".into()), Edn::Str("user".into()));
        Edn::Map(EdnMapView(meta))
      });
      map
    }));

    let result: Result<TestStruct, _> = from_edn(edn_map);
    assert!(result.is_ok());

    let test_struct = result.unwrap();
    assert_eq!(test_struct.name, "Bob");
    assert_eq!(test_struct.age, 25);
    assert!(!test_struct.is_active);
  }

  #[test]
  fn test_round_trip() {
    let original = TestStruct {
      name: "Charlie".to_string(),
      age: 35,
      is_active: true,
      scores: vec![95.0, 87.5, 91.0],
      metadata: [("department".to_string(), "engineering".to_string())]
        .into_iter()
        .collect(),
    };

    let edn_value = to_edn(&original).unwrap();
    let reconstructed: TestStruct = from_edn(edn_value).unwrap();

    assert_eq!(original, reconstructed);
  }

  #[test]
  fn test_quote_serialization() {
    use cirru_parser::Cirru;

    // Test simple quoted string
    let quote_str = Edn::Quote(Cirru::Leaf("hello".into()));
    let serialized = to_edn(&quote_str).unwrap();
    let deserialized: Edn = from_edn(serialized).unwrap();

    if let Edn::Quote(cirru) = &deserialized {
      if let Cirru::Leaf(s) = cirru {
        assert_eq!(s.as_ref(), "hello");
      } else {
        panic!("Expected Cirru::Leaf");
      }
    } else {
      panic!("Expected Edn::Quote");
    }

    // Test quoted list structure
    let quote_list = Edn::Quote(Cirru::List(vec![
      Cirru::Leaf("fn".into()),
      Cirru::Leaf("add".into()),
      Cirru::List(vec![Cirru::Leaf("a".into()), Cirru::Leaf("b".into())]),
      Cirru::List(vec![
        Cirru::Leaf("+".into()),
        Cirru::Leaf("a".into()),
        Cirru::Leaf("b".into()),
      ]),
    ]));

    let serialized = to_edn(&quote_list).unwrap();
    let deserialized: Edn = from_edn(serialized).unwrap();

    if let Edn::Quote(cirru) = &deserialized {
      if let Cirru::List(items) = cirru {
        assert_eq!(items.len(), 4);
        if let Cirru::Leaf(s) = &items[0] {
          assert_eq!(s.as_ref(), "fn");
        } else {
          panic!("Expected first item to be Cirru::Leaf");
        }
      } else {
        panic!("Expected Cirru::List");
      }
    } else {
      panic!("Expected Edn::Quote");
    }
  }
}
