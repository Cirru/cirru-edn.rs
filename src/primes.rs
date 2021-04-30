use cirru_parser::Cirru;
use core::cmp::Ord;
use regex::Regex;
use std::cmp::Eq;
use std::cmp::Ordering;
use std::cmp::Ordering::*;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};

/// Data format based on subset of EDN, but in Cirru syntax.
/// different parts are quote and Record.
#[derive(fmt::Debug, Clone)]
pub enum Edn {
  Nil,
  Bool(bool),
  Number(f64),
  Symbol(String),
  Keyword(String),
  Str(String), // name collision
  Quote(Cirru),
  List(Vec<Edn>),
  Set(HashSet<Edn>),
  Map(HashMap<Edn, Edn>),
  Record(String, Vec<String>, Vec<Edn>),
}

lazy_static! {
  static ref RE_SIMPLE_TOKEN: Regex = Regex::new("^[\\d\\w\\-\\?\\.\\$,]+$").unwrap();
}

impl fmt::Display for Edn {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Nil => f.write_str("nil"),
      Self::Bool(v) => f.write_str(&format!("{}", v)),
      Self::Number(n) => f.write_str(&format!("{}", n)),
      Self::Symbol(s) => f.write_str(&format!("'{}", s)),
      Self::Keyword(s) => f.write_str(&format!(":{}", s)),
      Self::Str(s) => {
        if RE_SIMPLE_TOKEN.is_match(s) {
          f.write_str(&format!("|{}", s))
        } else {
          f.write_str(&format!("\"|{}\"", s))
        }
      }
      Self::Quote(v) => f.write_str(&format!("(quote {})", v)),
      Self::List(xs) => {
        f.write_str("([]")?;
        for x in xs {
          f.write_str(&format!(" {}", x))?;
        }
        f.write_str(")")
      }
      Self::Set(xs) => {
        f.write_str("(#{}")?;
        for x in xs {
          f.write_str(&format!(" {}", x))?;
        }
        f.write_str(")")
      }
      Self::Map(xs) => {
        f.write_str("({}")?;
        for (k, v) in xs {
          f.write_str(&format!(" ({} {})", k, v))?;
        }
        f.write_str(")")
      }
      Self::Record(name, fields, values) => {
        f.write_str(&format!("(%{{}} {}", name))?;

        for idx in 0..fields.len() {
          f.write_str(&format!("({} {})", fields[idx], values[idx]))?;
        }

        f.write_str(")")
      }
    }
  }
}

impl Hash for Edn {
  fn hash<H>(&self, _state: &mut H)
  where
    H: Hasher,
  {
    match self {
      Self::Nil => "nil:".hash(_state),
      Self::Bool(v) => {
        "bool:".hash(_state);
        v.hash(_state);
      }
      Self::Number(n) => {
        "number:".hash(_state);
        (*n as usize).hash(_state) // TODO inaccurate solution
      }
      Self::Symbol(s) => {
        "symbol:".hash(_state);
        s.hash(_state);
      }
      Self::Keyword(s) => {
        "keyword:".hash(_state);
        s.hash(_state);
      }
      Self::Str(s) => {
        "string:".hash(_state);
        s.hash(_state);
      }
      Self::Quote(v) => {
        "quote:".hash(_state);
        v.hash(_state);
      }
      Self::List(v) => {
        "list:".hash(_state);
        v.hash(_state);
      }
      Self::Set(v) => {
        "set:".hash(_state);
        // TODO order for set is stable
        for x in v {
          x.hash(_state)
        }
      }
      Self::Map(v) => {
        "map:".hash(_state);
        // TODO order for map is not stable
        for x in v {
          x.hash(_state)
        }
      }
      Self::Record(name, fields, values) => {
        "record:".hash(_state);
        name.hash(_state);
        fields.hash(_state);
        values.hash(_state);
      }
    }
  }
}

impl Ord for Edn {
  fn cmp(&self, other: &Self) -> Ordering {
    match (self, other) {
      (Self::Nil, Self::Nil) => Equal,
      (Self::Nil, _) => Less,
      (_, Self::Nil) => Greater,

      (Self::Bool(a), Self::Bool(b)) => a.cmp(b),
      (Self::Bool(_), _) => Less,
      (_, Self::Bool(_)) => Greater,

      (Self::Number(a), Self::Number(b)) => {
        if a < b {
          Less
        } else if a > b {
          Greater
        } else {
          Equal
        }
      }
      (Self::Number(_), _) => Less,
      (_, Self::Number(_)) => Greater,

      (Self::Symbol(a), Self::Symbol(b)) => a.cmp(b),
      (Self::Symbol(_), _) => Less,
      (_, Self::Symbol(_)) => Greater,

      (Self::Keyword(a), Self::Keyword(b)) => a.cmp(b),
      (Self::Keyword(_), _) => Less,
      (_, Self::Keyword(_)) => Greater,

      (Self::Str(a), Self::Str(b)) => a.cmp(b),
      (Self::Str(_), _) => Less,
      (_, Self::Str(_)) => Greater,

      (Self::Quote(a), Self::Quote(b)) => a.cmp(b),
      (Self::Quote(_), _) => Less,
      (_, Self::Quote(_)) => Greater,

      (Self::List(a), Self::List(b)) => a.cmp(b),
      (Self::List(_), _) => Less,
      (_, Self::List(_)) => Greater,

      (Self::Set(a), Self::Set(b)) => match a.len().cmp(&b.len()) {
        Equal => unreachable!("TODO sets are not cmp ed"), // TODO
        a => a,
      },
      (Self::Set(_), _) => Less,
      (_, Self::Set(_)) => Greater,

      (Self::Map(a), Self::Map(b)) => {
        match a.len().cmp(&b.len()) {
          Equal => unreachable!(format!("TODO maps are not cmp ed {:?} {:?}", a, b)), // TODO
          a => a,
        }
      }
      (Self::Map(_), _) => Less,
      (_, Self::Map(_)) => Greater,

      (Self::Record(name1, fields1, values1), Self::Record(name2, fields2, values2)) => {
        match name1.cmp(name2) {
          Equal => match fields1.cmp(&fields2) {
            Equal => values1.cmp(&values2),
            a => a,
          },
          a => a,
        }
      }
    }
  }
}

impl PartialOrd for Edn {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Eq for Edn {}

impl PartialEq for Edn {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Nil, Self::Nil) => true,
      (Self::Bool(a), Self::Bool(b)) => a == b,
      (Self::Number(a), Self::Number(b)) => a == b,
      (Self::Symbol(a), Self::Symbol(b)) => a == b,
      (Self::Keyword(a), Self::Keyword(b)) => a == b,
      (Self::Str(a), Self::Str(b)) => a == b,
      (Self::Quote(a), Self::Quote(b)) => a == b,
      (Self::List(a), Self::List(b)) => a == b,
      (Self::Set(a), Self::Set(b)) => a == b,
      (Self::Map(a), Self::Map(b)) => a == b,
      (Self::Record(name1, fields1, values1), Self::Record(name2, fields2, values2)) => {
        name1 == name2 && fields1 == fields2 && values1 == values2
      }
      (_, _) => false,
    }
  }
}

/// Support reading from EDN
impl Edn {
  pub fn read_string(&self) -> Result<String, String> {
    match self {
      Edn::Str(s) => Ok(s.to_owned()),
      a => Err(format!("failed to convert to string: {}", a)),
    }
  }
  pub fn read_symbol_string(&self) -> Result<String, String> {
    match self {
      Edn::Symbol(s) => Ok(s.to_owned()),
      a => Err(format!("failed to convert to symbol: {}", a)),
    }
  }
  pub fn read_keyword_string(&self) -> Result<String, String> {
    match self {
      Edn::Keyword(s) => Ok(s.to_owned()),
      a => Err(format!("failed to convert to keyword: {}", a)),
    }
  }

  pub fn read_bool(&self) -> Result<bool, String> {
    match self {
      Edn::Bool(b) => Ok(b.to_owned()),
      a => Err(format!("failed to convert to bool: {}", a)),
    }
  }

  pub fn read_number(&self) -> Result<f64, String> {
    match self {
      Edn::Number(n) => Ok(n.to_owned()),
      a => Err(format!("failed to convert to number: {}", a)),
    }
  }

  pub fn read_quoted_cirru(&self) -> Result<Cirru, String> {
    match self {
      Edn::Quote(c) => Ok(c.to_owned()),
      a => Err(format!("failed to convert to cirru code: {}", a)),
    }
  }

  pub fn read_list(&self) -> Result<Vec<Edn>, String> {
    match self {
      Edn::List(xs) => Ok(xs.to_owned()),
      Edn::Nil => Err(String::from("cannot get from nil")),
      a => Err(format!("failed to convert to vec: {}", a)),
    }
  }

  pub fn read_set(&self) -> Result<HashSet<Edn>, String> {
    match self {
      Edn::Set(xs) => Ok(xs.to_owned()),
      Edn::Nil => Err(String::from("cannot get set from nil")),
      a => Err(format!("failed to convert to set: {}", a)),
    }
  }

  // as_set, but allow nil
  pub fn read_set_or_nil(&self) -> Result<HashSet<Edn>, String> {
    match self {
      Edn::Set(xs) => Ok(xs.to_owned()),
      Edn::Nil => Ok(HashSet::new()),
      a => Err(format!("failed to convert to set: {}", a)),
    }
  }

  pub fn read_map(&self) -> Result<HashMap<Edn, Edn>, String> {
    match self {
      Edn::Map(xs) => Ok(xs.to_owned()),
      Edn::Nil => Err(String::from("cannot map get from nil")),
      a => Err(format!("failed to convert to map: {}", a)),
    }
  }

  // as_map, but allow nil being treated as empty map
  pub fn read_map_or_nil(&self) -> Result<HashMap<Edn, Edn>, String> {
    match self {
      Edn::Map(xs) => Ok(xs.to_owned()),
      Edn::Nil => Ok(HashMap::new()),
      a => Err(format!("failed to convert to map: {}", a)),
    }
  }

  /// detects by index
  pub fn vec_get(&self, idx: usize) -> Result<Edn, String> {
    match self {
      Edn::List(xs) => {
        if idx < xs.len() {
          Ok(xs[idx].to_owned())
        } else {
          Ok(Edn::Nil)
        }
      }
      a => Err(format!("target is not vec: {}", a)),
    }
  }

  /// detects by keyword then string, return nil if not found
  pub fn map_get(&self, k: &str) -> Result<Edn, String> {
    let key: String = k.to_owned();
    match self {
      Edn::Map(xs) => {
        if xs.contains_key(&Edn::Keyword(key.clone())) {
          Ok(xs[&Edn::Keyword(key)].clone())
        } else if xs.contains_key(&Edn::Str(key.clone())) {
          Ok(xs[&Edn::Str(key)].clone())
        } else {
          Ok(Edn::Nil)
        }
      }
      a => Err(format!("target is not map: {}", a)),
    }
  }
}
