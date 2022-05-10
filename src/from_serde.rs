use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeMap, SerializeSeq, SerializeTuple};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;

use crate::Edn;

#[derive(Serialize, Deserialize)]
enum EdnU8Kind {
  Keyword,
  Symbol,
  Tuple,
  Quote,
  Set,
  Record,
}

impl Serialize for Edn {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    match self {
      Edn::Nil => serializer.serialize_unit(),
      Edn::Bool(b) => serializer.serialize_bool(*b),
      Edn::Number(n) => serializer.serialize_f64(*n),
      Edn::Str(s) => serializer.collect_str(&*s),
      Edn::Keyword(k) => {
        // let mut s = serializer.serialize_tuple_struct("Keyword", 1)?;
        // s.serialize_field(&*k.to_str())?;
        // s.end()

        let mut t = serializer.serialize_tuple(2)?;
        t.serialize_element(&EdnU8Kind::Keyword)?;
        t.serialize_element(&*k.to_str())?;
        t.end()
      }
      Edn::Symbol(s) => {
        let mut t = serializer.serialize_tuple(2)?;
        t.serialize_element(&EdnU8Kind::Symbol)?;
        t.serialize_element(&**s)?;
        t.end()
      }

      Edn::Buffer(buf) => serializer.serialize_bytes(buf),

      Edn::Tuple(pair) => {
        // Edn only supports tuple of 2, so it can be a special case here
        let mut t = serializer.serialize_tuple(3)?;
        t.serialize_element(&EdnU8Kind::Tuple)?;
        t.serialize_element(&pair.0)?;
        t.serialize_element(&pair.1)?;
        t.end()
      }
      Edn::Quote(c) => {
        let mut t = serializer.serialize_tuple(2)?;
        t.serialize_element(&EdnU8Kind::Quote)?;
        t.serialize_element(&*c)?;
        t.end()
      }
      Edn::List(xs) => {
        let mut seq = serializer.serialize_seq(Some(xs.len()))?;
        for e in xs {
          seq.serialize_element(e)?;
        }
        seq.end()
      }
      Edn::Set(xs) => {
        serializer.serialize_newtype_struct("Set", xs)
        // let mut t = serializer.serialize_tuple(xs.len())?;
        // t.serialize_element(&EdnU8Kind::Set)?;
        // t.serialize_element(xs)?;
        // t.end()
      }
      Edn::Map(m) => {
        let mut seq = serializer.serialize_map(Some(m.len()))?;
        for (k, v) in m {
          seq.serialize_entry(k, v)?;
        }
        seq.end()
      }

      Edn::Record(name, dict) => {
        let mut seq = serializer.serialize_tuple(3)?;
        seq.serialize_element(&EdnU8Kind::Record)?;
        seq.serialize_element(&*name)?;
        seq.serialize_element(dict)?;
        seq.end()

        // let mut s = serializer.serialize_struct(&name.to_str(), dict.len())?;
        // for (k, v) in dict {
        //   let field = k.to_str().to_string();
        //   s.serialize_field(&field, v)?;
        // }
        // s.end()
      }
    }
  }
}

pub struct EdnVisitor;

impl<'de> Visitor<'de> for EdnVisitor {
  type Value = Edn;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("invalid data for EDN")
  }

  fn visit_unit<E>(self) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Edn::Nil)
  }

  fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Edn::Bool(v))
  }

  fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Edn::Number(v as f64))
  }

  fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Edn::Number(v))
  }

  fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Edn::Number(v as f64))
  }

  fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Edn::Number(v as f64))
  }

  fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Edn::Number(v as f64))
  }

  fn visit_str<E>(self, s: &str) -> Result<Self::Value, E> {
    Ok(Edn::Str(s.into()))
  }

  fn visit_string<E>(self, s: String) -> Result<Self::Value, E> {
    Ok(Edn::Str(s.into()))
  }

  fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Edn::Buffer(v.to_vec()))
  }

  fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Edn::Buffer(v))
  }

  fn visit_none<E>(self) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Edn::Nil)
  }

  fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_any(self)
  }

  fn visit_seq<M>(self, mut access: M) -> Result<Self::Value, M::Error>
  where
    M: SeqAccess<'de>,
  {
    let mut seq = Vec::with_capacity(access.size_hint().unwrap_or(0));
    while let Some(el) = access.next_element()? {
      seq.push(el);
    }

    println!("seq {:?}", seq);
    Ok(Edn::List(seq))
  }

  fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
  where
    M: MapAccess<'de>,
  {
    let mut map = HashMap::new();
    while let Some((k, v)) = access.next_entry()? {
      map.insert(k, v);
    }
    Ok(Edn::Map(map))
  }

  fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Edn::Str(v.to_string().into()))
  }

  fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Edn::Str(v.into()))
  }

  fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Edn::Buffer(v.to_vec()))
  }

  // fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  // where
  //   D: Deserializer<'de>,
  // {
  // }

  /// TODO
  fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
  where
    A: serde::de::EnumAccess<'de>,
  {
    match data.variant()? {
      (EdnU8Kind::Keyword, _k) => Ok(Edn::Nil),
      (EdnU8Kind::Symbol, _k) => Ok(Edn::Nil),
      (EdnU8Kind::Quote, _k) => Ok(Edn::Nil),
      (EdnU8Kind::Tuple, _k) => Ok(Edn::Nil),
      (EdnU8Kind::Set, _k) => Ok(Edn::Nil),
      (EdnU8Kind::Record, _k) => Ok(Edn::Nil),
    }
  }
}

impl<'de> Deserialize<'de> for Edn {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_any(EdnVisitor {})
  }
}

// impl<'de> Deserializer<'de> for Edn {
//   type Error = String;

//   fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//   where
//     V: Visitor<'de>,
//   {
//     self.deserialize_any(EdnVisitor {})
//   }

//   forward_to_deserialize_any! {
//       bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
//       bytes byte_buf option unit unit_struct newtype_struct seq tuple
//       tuple_struct map struct enum identifier ignored_any
//   }
// }
