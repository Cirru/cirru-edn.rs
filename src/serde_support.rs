//! Serde support for Edn data format.
//!
//! This module provides seamless integration with the serde ecosystem,
//! allowing easy conversion between Rust structs and Edn values.
//!
//! **Note**: This module is only available when the `serde` feature is enabled.
//! Add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! cirru_edn = { version = "0.6", features = ["serde"] }
//! ```
//!
//! # Usage
//!
//! ```rust
//! use cirru_edn::{to_edn, from_edn};
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct Person {
//!     name: String,
//!     age: u32,
//! }
//!
//! let person = Person { name: "Alice".to_string(), age: 30 };
//!
//! // Serialize to Edn
//! let edn_value = to_edn(&person).unwrap();
//!
//! // Deserialize from Edn
//! let recovered: Person = from_edn(edn_value).unwrap();
//! ```
//!
//! # Type Mapping
//!
//! - Rust `Option<T>` maps to either `Edn::Nil` or the contained value
//! - Rust `Vec<T>` maps to `Edn::List`
//! - Rust `HashMap<K, V>` maps to `Edn::Map`
//! - Rust `HashSet<T>` maps to `Edn::Set` (with special encoding)
//! - Primitive types map directly to their Edn equivalents
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
  ser::{self, SerializeMap, SerializeSeq},
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
        map.serialize_entry("__edn_quote", &cirru_to_serializable(cirru))?;
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
                  let cirru = serializable_to_cirru(cirru_data)
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
/// This is similar to `serde_json::to_value`.
///
/// # Examples
///
/// ```
/// use serde::Serialize;
/// use cirru_edn::{to_edn, Edn};
///
/// #[derive(Serialize)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
///
/// let person = Person {
///     name: "Alice".to_string(),
///     age: 30,
/// };
///
/// let edn_value = to_edn(&person).unwrap();
/// ```
pub fn to_edn<T>(value: T) -> Result<Edn, String>
where
  T: Serialize,
{
  // First serialize to serde_json::Value, then convert to Edn
  let json_value = serde_json::to_value(value).map_err(|e| e.to_string())?;
  json_value_to_edn(json_value)
}

/// Convert an `Edn` to a `T` where `T` implements `Deserialize`.
///
/// This is similar to `serde_json::from_value`.
///
/// # Examples
///
/// ```
/// use serde::Deserialize;
/// use cirru_edn::{from_edn, Edn};
///
/// #[derive(Deserialize)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
///
/// let edn_map = Edn::map_from_iter([
///     ("name".into(), "Alice".into()),
///     ("age".into(), 30.into()),
/// ]);
///
/// let person: Person = from_edn(edn_map).unwrap();
/// ```
pub fn from_edn<T>(value: Edn) -> Result<T, String>
where
  T: for<'de> Deserialize<'de>,
{
  // Convert Edn to serde_json::Value, then deserialize
  let json_value = edn_to_json_value(value)?;
  serde_json::from_value(json_value).map_err(|e| e.to_string())
}

fn json_value_to_edn(value: serde_json::Value) -> Result<Edn, String> {
  match value {
    serde_json::Value::Null => Ok(Edn::Nil),
    serde_json::Value::Bool(b) => Ok(Edn::Bool(b)),
    serde_json::Value::Number(n) => {
      if let Some(f) = n.as_f64() {
        Ok(Edn::Number(f))
      } else {
        Err("Invalid number format".to_string())
      }
    }
    serde_json::Value::String(s) => Ok(Edn::Str(s.into())),
    serde_json::Value::Array(arr) => {
      let mut edn_list = Vec::new();
      for item in arr {
        edn_list.push(json_value_to_edn(item)?);
      }
      Ok(Edn::List(EdnListView(edn_list)))
    }
    serde_json::Value::Object(obj) => {
      let mut edn_map = HashMap::new();
      for (k, v) in obj {
        edn_map.insert(Edn::Str(k.into()), json_value_to_edn(v)?);
      }
      Ok(Edn::Map(EdnMapView(edn_map)))
    }
  }
}

fn edn_to_json_value(value: Edn) -> Result<serde_json::Value, String> {
  match value {
    Edn::Nil => Ok(serde_json::Value::Null),
    Edn::Bool(b) => Ok(serde_json::Value::Bool(b)),
    Edn::Number(n) => {
      // If the number is a whole number, try to represent it as an integer
      if n.fract().abs() < f64::EPSILON {
        // Check if it fits in i64 range
        if n >= i64::MIN as f64 && n <= i64::MAX as f64 {
          let int_val = n as i64;
          Ok(serde_json::Value::Number(serde_json::Number::from(int_val)))
        } else {
          // Fall back to f64
          serde_json::Number::from_f64(n)
            .map(serde_json::Value::Number)
            .ok_or_else(|| "Invalid number".to_string())
        }
      } else {
        // It's a fractional number, use f64
        serde_json::Number::from_f64(n)
          .map(serde_json::Value::Number)
          .ok_or_else(|| "Invalid number".to_string())
      }
    }
    Edn::Str(s) => Ok(serde_json::Value::String((*s).to_string())),
    Edn::List(EdnListView(list)) => {
      let mut json_array = Vec::new();
      for item in list {
        json_array.push(edn_to_json_value(item)?);
      }
      Ok(serde_json::Value::Array(json_array))
    }
    Edn::Map(EdnMapView(map)) => {
      let mut json_obj = serde_json::Map::new();
      for (k, v) in map {
        if let Edn::Str(key_str) = k {
          json_obj.insert((*key_str).to_string(), edn_to_json_value(v)?);
        } else {
          return Err("Map keys must be strings for JSON conversion".to_string());
        }
      }
      Ok(serde_json::Value::Object(json_obj))
    }
    _ => Err(format!("Unsupported Edn type for JSON conversion: {:?}", value)),
  }
}

/// Convert a Cirru value to a serializable representation (nested arrays and strings)
fn cirru_to_serializable(cirru: &Cirru) -> serde_json::Value {
  match cirru {
    Cirru::Leaf(s) => serde_json::Value::String(s.as_ref().to_string()),
    Cirru::List(items) => {
      let serialized_items: Vec<serde_json::Value> = items.iter().map(cirru_to_serializable).collect();
      serde_json::Value::Array(serialized_items)
    }
  }
}

/// Convert a serializable representation back to Cirru
fn serializable_to_cirru(value: &Edn) -> Result<Cirru, String> {
  match value {
    Edn::Str(s) => Ok(Cirru::Leaf(s.as_ref().into())),
    Edn::List(EdnListView(items)) => {
      let mut cirru_items = Vec::new();
      for item in items {
        cirru_items.push(serializable_to_cirru(item)?);
      }
      Ok(Cirru::List(cirru_items))
    }
    _ => Err(format!("Invalid Cirru representation: {:?}", value)),
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

    // Verify the conversion
    if let Edn::Map(map) = edn_value {
      assert!(map.0.contains_key(&Edn::Str("name".into())));
      assert!(map.0.contains_key(&Edn::Str("age".into())));
    } else {
      panic!("Expected Edn::Map");
    }
  }

  #[test]
  fn test_from_edn() {
    let edn_map = Edn::map_from_iter([
      ("name".into(), "Bob".into()),
      ("age".into(), 25.into()),
      ("is_active".into(), false.into()),
      ("scores".into(), vec![90.0, 88.5].into()),
      ("metadata".into(), {
        let mut meta = HashMap::new();
        meta.insert(Edn::Str("role".into()), Edn::Str("user".into()));
        Edn::Map(EdnMapView(meta))
      }),
    ]);

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
