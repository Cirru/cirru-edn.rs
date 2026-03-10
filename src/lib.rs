//! # Cirru EDN
//!
//! Extensible Data Notation (EDN) implementation using Cirru syntax.
//!
//! This crate provides a data format similar to EDN but using Cirru's syntax instead of
//! traditional s-expressions. It supports rich data types including primitives, collections,
//! and special constructs like records and tuples.
//!
//! ## Features
//!
//! - **Rich data types**: nil, boolean, number, string, symbol, tag, list, set, map, record, tuple, buffer, atom
//! - **Serde integration**: Seamless serialization/deserialization with Rust structs
//! - **Efficient string handling**: Uses `Arc<str>` for string deduplication
//! - **Runtime references**: Support for arbitrary Rust data via `AnyRef`
//! - **Type-safe API**: Strong typing with convenient conversion methods
//!
//! ## Basic Usage
//!
//! ```rust
//! use cirru_edn::{parse, format, Edn};
//!
//! // Parse Cirru EDN from string
//! let data = parse("[] 1 2 3").unwrap();
//!
//! // Create EDN values programmatically
//! let map = Edn::map_from_iter([
//!     (Edn::tag("name"), Edn::str("Alice")),
//!     (Edn::tag("age"), Edn::Number(30.0)),
//! ]);
//!
//! // Format back to string
//! let formatted = format(&map, true).unwrap();
//! ```
//!
//! ## Type Checking and Conversion
//!
//! The library provides type-safe methods for checking and converting values:
//!
//! ```rust
//! use cirru_edn::Edn;
//!
//! let value = Edn::Number(42.0);
//!
//! // Type checking
//! assert!(value.is_number());
//! assert!(!value.is_string());
//!
//! // Safe conversion
//! let number: f64 = value.read_number().unwrap();
//! assert_eq!(number, 42.0);
//!
//! // Get type name for debugging
//! assert_eq!(value.type_name(), "number");
//! ```
//!
//! ## Working with Collections
//!
//! ```rust
//! use cirru_edn::Edn;
//!
//! // Create and access lists
//! let list = Edn::List(vec![
//!     Edn::Number(1.0),
//!     Edn::str("hello"),
//!     Edn::Bool(true)
//! ].into());
//!
//! if let Some(first) = list.get_list_item(0) {
//!     assert_eq!(first.read_number().unwrap(), 1.0);
//! }
//!
//! // Create and access maps
//! let map = Edn::map_from_iter([
//!     (Edn::tag("name"), Edn::str("Bob")),
//!     (Edn::tag("age"), Edn::Number(25.0)),
//! ]);
//!
//! if let Some(name) = map.get_map_value(&Edn::tag("name")) {
//!     assert_eq!(name.read_string().unwrap(), "Bob");
//! }
//! ```
//!
//! ## Serde Integration
//!
//! The crate includes built-in serde support for seamless serialization:
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
//! let person = Person { name: "Bob".to_string(), age: 25 };
//! let edn_value = to_edn(&person).unwrap();
//! let recovered: Person = from_edn(edn_value).unwrap();
//! ```

mod edn;
mod error;
mod tag;

pub mod serde_support;

use std::cmp::Ordering::*;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::sync::Arc;
use std::vec;

use cirru_parser::Cirru;

pub use edn::{
  DynEq, Edn, EdnAnyRef, EdnListView, EdnMapView, EdnRecordView, EdnSetView, EdnTupleView, is_simple_char,
};
pub use error::{EdnError, EdnResult, Position};
pub use tag::EdnTag;

// Backward compatible type alias
#[deprecated(since = "0.7.0", note = "Use EdnError instead")]
pub type EdnResultString<T> = Result<T, String>;

// Convenience type aliases for common patterns
pub type EdnList = EdnListView;
pub type EdnMap = EdnMapView;
pub type EdnSet = EdnSetView;
pub type EdnRecord = EdnRecordView;
pub type EdnTuple = EdnTupleView;

// Common constants for convenience
impl Edn {
  /// Predefined nil constant for convenience
  pub const NIL: Edn = Edn::Nil;

  /// Predefined true constant for convenience
  pub const TRUE: Edn = Edn::Bool(true);

  /// Predefined false constant for convenience
  pub const FALSE: Edn = Edn::Bool(false);
}

pub use serde_support::{from_edn, to_edn};

/// Parse Cirru code into Edn data.
///
/// This function takes a string containing Cirru syntax and converts it into an Edn value.
/// The input must contain exactly one expression.
///
/// # Arguments
///
/// * `s` - A string slice containing the Cirru code to parse
///
/// # Returns
///
/// * `Result<Edn, String>` - Returns the parsed Edn value on success, or an error message on failure
///
/// # Examples
///
/// ```
/// use cirru_edn::parse;
///
/// // Parse a simple number
/// let result = parse("do 42").unwrap();
///
/// // Parse a list
/// let result = parse("[] 1 2 3").unwrap();
///
/// // Parse a map
/// let result = parse("{} (:name |Alice) (:age 30)").unwrap();
///
/// // Parse nested structures
/// let result = parse("{} (:items $ [] 1 2 3) (:meta $ {} (:version 1))").unwrap();
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The input contains no expressions or more than one expression
/// - The syntax is invalid
/// - The input contains unsupported constructs
pub fn parse(s: &str) -> EdnResult<Edn> {
  let xs = cirru_parser::parse(s).map_err(|e| EdnError::from_parse_error_detailed(e, s))?;
  if xs.len() == 1 {
    match &xs[0] {
      Cirru::Leaf(s) => Err(EdnError::structure(
        format!("expected expr for data, got leaf: {s}"),
        vec![],
        Some(&xs[0]),
      )),
      Cirru::List(_) => extract_cirru_edn_with_path(&xs[0], vec![]),
    }
  } else {
    Err(EdnError::structure(
      format!("Expected 1 expr for edn, got length {}: {:?} ", xs.len(), xs),
      vec![],
      None,
    ))
  }
}

fn extract_cirru_edn_with_path(node: &Cirru, path: Vec<usize>) -> EdnResult<Edn> {
  match node {
    Cirru::Leaf(s) => match &**s {
      "nil" => Ok(Edn::Nil),
      "true" => Ok(Edn::Bool(true)),
      "false" => Ok(Edn::Bool(false)),
      "" => Err(EdnError::value(
        "empty string is invalid for edn",
        path.clone(),
        Some(node),
      )),
      s1 => match s1.chars().next().unwrap() {
        '\'' => Ok(Edn::Symbol(s1[1..].into())),
        ':' => Ok(Edn::tag(&s1[1..])),
        '"' | '|' => Ok(Edn::Str(s1[1..].into())),
        _ => {
          if let Ok(f) = s1.trim().parse::<f64>() {
            Ok(Edn::Number(f))
          } else {
            Err(EdnError::value(
              format!("unknown token for edn value: {s1:?}"),
              path.clone(),
              Some(node),
            ))
          }
        }
      },
    },
    Cirru::List(xs) => {
      if xs.is_empty() {
        Err(EdnError::structure(
          "empty expr is invalid for edn",
          path.clone(),
          Some(node),
        ))
      } else {
        match &xs[0] {
          Cirru::Leaf(s) => match &**s {
            "quote" => {
              if xs.len() == 2 {
                Ok(Edn::Quote(xs[1].to_owned()))
              } else {
                Err(EdnError::structure("missing edn quote value", path.clone(), Some(node)))
              }
            }
            "do" => {
              let mut ret: Option<Edn> = None;

              for (i, x) in xs.iter().enumerate().skip(1) {
                if is_comment(x) {
                  continue;
                }
                if ret.is_some() {
                  return Err(EdnError::structure("multiple values in do", path.clone(), Some(node)));
                }
                let mut child_path = path.clone();
                child_path.push(i);
                ret = Some(extract_cirru_edn_with_path(x, child_path)?);
              }
              if ret.is_none() {
                return Err(EdnError::structure("missing edn do value", path.clone(), Some(node)));
              }
              ret.ok_or_else(|| EdnError::structure("missing edn do value", path.clone(), Some(node)))
            }
            "::" => {
              let mut tag: Option<Edn> = None;
              let mut extra: Vec<Edn> = vec![];
              for (i, x) in xs.iter().enumerate().skip(1) {
                if is_comment(x) {
                  continue;
                }
                let mut child_path = path.clone();
                child_path.push(i);
                if tag.is_some() {
                  extra.push(extract_cirru_edn_with_path(x, child_path)?);
                  continue;
                } else {
                  tag = Some(extract_cirru_edn_with_path(x, child_path)?);
                }
              }
              if let Some(x0) = tag {
                Ok(Edn::Tuple(EdnTupleView {
                  tag: Arc::new(x0),
                  enum_tag: None,
                  extra,
                }))
              } else {
                Err(EdnError::structure(
                  "missing edn :: fst value",
                  path.clone(),
                  Some(node),
                ))
              }
            }
            "%::" => {
              let mut enum_tag: Option<Edn> = None;
              let mut tag: Option<Edn> = None;
              let mut extra: Vec<Edn> = vec![];
              for (i, x) in xs.iter().enumerate().skip(1) {
                if is_comment(x) {
                  continue;
                }
                let mut child_path = path.clone();
                child_path.push(i);
                if enum_tag.is_none() {
                  enum_tag = Some(extract_cirru_edn_with_path(x, child_path)?);
                } else if tag.is_none() {
                  tag = Some(extract_cirru_edn_with_path(x, child_path)?);
                } else {
                  extra.push(extract_cirru_edn_with_path(x, child_path)?);
                }
              }
              if let (Some(e0), Some(x0)) = (enum_tag, tag) {
                Ok(Edn::Tuple(EdnTupleView {
                  tag: Arc::new(x0),
                  enum_tag: Some(Arc::new(e0)),
                  extra,
                }))
              } else {
                Err(EdnError::structure(
                  "missing edn %:: enum_tag or tag value",
                  path.clone(),
                  Some(node),
                ))
              }
            }
            "[]" => {
              let mut ys: Vec<Edn> = Vec::with_capacity(xs.len() - 1);
              for (i, x) in xs.iter().enumerate().skip(1) {
                if is_comment(x) {
                  continue;
                }
                let mut child_path = path.clone();
                child_path.push(i);
                match extract_cirru_edn_with_path(x, child_path) {
                  Ok(v) => ys.push(v),
                  Err(v) => return Err(v),
                }
              }
              Ok(Edn::List(EdnListView(ys)))
            }
            "#{}" => {
              #[allow(clippy::mutable_key_type)]
              let mut ys: HashSet<Edn> = HashSet::new();
              for (i, x) in xs.iter().enumerate().skip(1) {
                if is_comment(x) {
                  continue;
                }
                let mut child_path = path.clone();
                child_path.push(i);
                match extract_cirru_edn_with_path(x, child_path) {
                  Ok(v) => {
                    ys.insert(v);
                  }
                  Err(v) => return Err(v),
                }
              }
              Ok(Edn::Set(EdnSetView(ys)))
            }
            "{}" => {
              #[allow(clippy::mutable_key_type)]
              let mut zs: HashMap<Edn, Edn> = HashMap::new();
              for (i, x) in xs.iter().enumerate().skip(1) {
                if is_comment(x) {
                  continue;
                }
                let mut child_path = path.clone();
                child_path.push(i);
                match x {
                  Cirru::Leaf(s) => {
                    return Err(EdnError::structure(
                      format!("expected a pair, invalid map entry: {s}"),
                      child_path,
                      Some(node),
                    ));
                  }
                  Cirru::List(ys) => {
                    if ys.len() == 2 {
                      let mut k_path = child_path.clone();
                      k_path.push(0);
                      let mut v_path = child_path.clone();
                      v_path.push(1);
                      match (
                        extract_cirru_edn_with_path(&ys[0], k_path.clone()),
                        extract_cirru_edn_with_path(&ys[1], v_path.clone()),
                      ) {
                        (Ok(k), Ok(v)) => {
                          zs.insert(k, v);
                        }
                        (Err(e), _) => {
                          return Err(EdnError::structure(
                            format!("invalid map entry `{}` from `{}`", e, &ys[0]),
                            k_path,
                            Some(node),
                          ));
                        }
                        (Ok(k), Err(e)) => {
                          return Err(EdnError::structure(
                            format!("invalid map entry for `{k}`, got {e}"),
                            v_path,
                            Some(node),
                          ));
                        }
                      }
                    }
                  }
                }
              }
              Ok(Edn::Map(EdnMapView(zs)))
            }
            "%{}" => {
              if xs.len() >= 3 {
                let name = match &xs[1] {
                  Cirru::Leaf(s) => EdnTag::new(s.strip_prefix(':').unwrap_or(s)),
                  Cirru::List(e) => {
                    let mut name_path = path.clone();
                    name_path.push(1);
                    return Err(EdnError::structure(
                      format!("expected record name in string: {e:?}"),
                      name_path,
                      Some(node),
                    ));
                  }
                };
                let mut entries: Vec<(EdnTag, Edn)> = Vec::with_capacity(xs.len() - 1);

                for (i, x) in xs.iter().enumerate().skip(2) {
                  if is_comment(x) {
                    continue;
                  }
                  let mut child_path = path.clone();
                  child_path.push(i);
                  match x {
                    Cirru::Leaf(s) => {
                      return Err(EdnError::structure(
                        format!("expected record, invalid record entry: {s}"),
                        child_path,
                        Some(node),
                      ));
                    }
                    Cirru::List(ys) => {
                      if ys.len() == 2 {
                        let mut v_path = child_path.clone();
                        v_path.push(1);
                        match (&ys[0], extract_cirru_edn_with_path(&ys[1], v_path.clone())) {
                          (Cirru::Leaf(s), Ok(v)) => {
                            entries.push((EdnTag::new(s.strip_prefix(':').unwrap_or(s)), v));
                          }
                          (Cirru::Leaf(s), Err(e)) => {
                            return Err(EdnError::structure(
                              format!("invalid record value for `{s}`, got: {e}"),
                              v_path,
                              Some(node),
                            ));
                          }
                          (Cirru::List(zs), _) => {
                            let mut k_path = child_path.clone();
                            k_path.push(0);
                            return Err(EdnError::structure(
                              format!("invalid list as record key: {zs:?}"),
                              k_path,
                              Some(node),
                            ));
                          }
                        }
                      } else {
                        return Err(EdnError::structure(
                          format!("expected pair of 2: {ys:?}"),
                          child_path,
                          Some(node),
                        ));
                      }
                    }
                  }
                }
                if entries.is_empty() {
                  return Err(EdnError::structure("empty record is invalid", path.clone(), Some(node)));
                }
                Ok(Edn::Record(EdnRecordView {
                  tag: name,
                  pairs: entries,
                }))
              } else {
                Err(EdnError::structure(
                  "insufficient items for edn record",
                  path.clone(),
                  Some(node),
                ))
              }
            }
            "buf" => {
              let mut ys: Vec<u8> = Vec::with_capacity(xs.len() - 1);
              for (i, x) in xs.iter().enumerate().skip(1) {
                if is_comment(x) {
                  continue;
                }
                let mut child_path = path.clone();
                child_path.push(i);
                match x {
                  Cirru::Leaf(y) => {
                    if y.len() == 2 {
                      match hex::decode(&(**y)) {
                        Ok(b) => {
                          if b.len() == 1 {
                            ys.push(b[0])
                          } else {
                            return Err(EdnError::value(
                              format!("hex for buffer might be too large, got: {b:?}"),
                              child_path,
                              Some(node),
                            ));
                          }
                        }
                        Err(e) => {
                          return Err(EdnError::value(
                            format!("expected length 2 hex string in buffer, got: {y} {e}"),
                            child_path,
                            Some(node),
                          ));
                        }
                      }
                    } else {
                      return Err(EdnError::value(
                        format!("expected length 2 hex string in buffer, got: {y}"),
                        child_path,
                        Some(node),
                      ));
                    }
                  }
                  _ => {
                    return Err(EdnError::value(
                      format!("expected hex string in buffer, got: {x}"),
                      child_path,
                      Some(node),
                    ));
                  }
                }
              }
              Ok(Edn::Buffer(ys))
            }
            "atom" => {
              if xs.len() == 2 {
                let mut child_path = path.clone();
                child_path.push(1);
                Ok(Edn::Atom(Box::new(extract_cirru_edn_with_path(&xs[1], child_path)?)))
              } else {
                Err(EdnError::structure("missing edn atom value", path.clone(), Some(node)))
              }
            }
            a => Err(EdnError::structure(
              format!("invalid operator for edn: {a}"),
              path.clone(),
              Some(node),
            )),
          },
          Cirru::List(a) => Err(EdnError::structure(
            format!("invalid nodes for edn: {a:?}"),
            path.clone(),
            Some(node),
          )),
        }
      }
    }
  }
}

fn is_comment(node: &Cirru) -> bool {
  match node {
    Cirru::Leaf(_) => false,
    Cirru::List(xs) => xs.first() == Some(&Cirru::Leaf(";".into())),
  }
}

fn assemble_cirru_node(data: &Edn) -> Cirru {
  match data {
    Edn::Nil => "nil".into(),
    Edn::Bool(v) => v.to_string().as_str().into(),
    Edn::Number(n) => n.to_string().as_str().into(),
    Edn::Symbol(s) => format!("'{s}").as_str().into(),
    Edn::Tag(s) => format!(":{s}").as_str().into(),
    Edn::Str(s) => format!("|{s}").as_str().into(),
    Edn::Quote(v) => Cirru::List(vec!["quote".into(), (*v).to_owned()]),
    Edn::List(xs) => {
      let mut ys: Vec<Cirru> = Vec::with_capacity(xs.len() + 1);
      ys.push("[]".into());
      for x in xs {
        ys.push(assemble_cirru_node(x));
      }
      Cirru::List(ys)
    }
    Edn::Set(xs) => {
      let mut ys: Vec<Cirru> = Vec::with_capacity(xs.len() + 1);
      ys.push("#{}".into());
      let mut items = xs.0.iter().collect::<Vec<_>>();
      items.sort();
      for x in items {
        ys.push(assemble_cirru_node(x));
      }
      Cirru::List(ys)
    }
    Edn::Map(xs) => {
      let mut ys: Vec<Cirru> = Vec::with_capacity(xs.len() + 1);
      ys.push("{}".into());
      let mut items = Vec::from_iter(xs.0.iter());
      items.sort_by(|(a1, a2): &(&Edn, &Edn), (b1, b2): &(&Edn, &Edn)| {
        match (a1.is_literal(), b1.is_literal(), a2.is_literal(), b2.is_literal()) {
          (true, true, true, false) => Less,
          (true, true, false, true) => Greater,
          (true, false, ..) => Less,
          (false, true, ..) => Greater,
          _ => a1.cmp(b1),
        }
      });
      for (k, v) in items {
        ys.push(Cirru::List(vec![assemble_cirru_node(k), assemble_cirru_node(v)]))
      }
      Cirru::List(ys)
    }
    Edn::Record(EdnRecordView {
      tag: name,
      pairs: entries,
    }) => {
      let mut ys: Vec<Cirru> = Vec::with_capacity(entries.len() + 2);
      ys.push("%{}".into());
      ys.push(format!(":{name}").as_str().into());
      let mut ordered_entries = entries.to_owned();
      ordered_entries.sort_by(|(a1, a2), (b1, b2)| match (a2.is_literal(), b2.is_literal()) {
        (true, false) => Less,
        (false, true) => Greater,
        _ => a1.cmp(b1),
      });
      for entry in ordered_entries {
        let v = &entry.1;
        ys.push(Cirru::List(vec![
          format!(":{}", entry.0).as_str().into(),
          assemble_cirru_node(v),
        ]));
      }

      Cirru::List(ys)
    }
    Edn::Tuple(EdnTupleView { tag, enum_tag, extra }) => {
      let mut ys: Vec<Cirru> = if let Some(et) = enum_tag {
        vec!["%::".into(), assemble_cirru_node(et), assemble_cirru_node(tag)]
      } else {
        vec!["::".into(), assemble_cirru_node(tag)]
      };
      for item in extra {
        ys.push(assemble_cirru_node(item))
      }
      Cirru::List(ys)
    }
    Edn::Buffer(buf) => {
      let mut ys: Vec<Cirru> = Vec::with_capacity(buf.len() + 1);
      ys.push("buf".into());
      for b in buf {
        ys.push(hex::encode(vec![b.to_owned()]).as_str().into());
      }
      Cirru::List(ys)
    }
    Edn::AnyRef(..) => unreachable!("AnyRef is not serializable"),
    Edn::Atom(v) => {
      let ys = vec!["atom".into(), assemble_cirru_node(v)];
      Cirru::List(ys)
    }
  }
}

/// Generate formatted string from Edn data.
///
/// This function converts an Edn value into its Cirru syntax representation.
///
/// # Arguments
///
/// * `data` - The Edn value to format
/// * `use_inline` - Whether to use inline formatting (more compact) or multiline formatting
///
/// # Returns
///
/// * `Result<String, String>` - Returns the formatted string on success, or an error message on failure
///
/// # Examples
///
/// ```
/// use cirru_edn::{Edn, format};
///
/// let data = Edn::Number(42.0);
/// let result = format(&data, true).unwrap();
/// assert_eq!(result.trim(), "do 42");
///
/// // Format a list with inline style
/// let data = Edn::List(vec![
///     Edn::Number(1.0),
///     Edn::Number(2.0),
///     Edn::Number(3.0),
/// ].into());
/// let result = format(&data, true).unwrap();
/// // Output: ([] 1 2 3)
///
/// // Format a map with multiline style
/// let data = Edn::map_from_iter([
///     (Edn::tag("name"), Edn::str("Alice")),
///     (Edn::tag("age"), Edn::Number(30.0)),
/// ]);
/// let result = format(&data, false).unwrap();
/// ```
///
/// # Notes
///
/// - AnyRef values cannot be formatted and will cause an error
/// - The function automatically wraps single literals in `do` expressions
/// - Inline formatting produces more compact output, while multiline formatting is more readable
pub fn format(data: &Edn, use_inline: bool) -> Result<String, String> {
  match assemble_cirru_node(data) {
    Cirru::Leaf(s) => cirru_parser::format(&[vec!["do", &*s].into()], use_inline.into()),
    Cirru::List(xs) => cirru_parser::format(&[(Cirru::List(xs))], use_inline.into()),
  }
}
