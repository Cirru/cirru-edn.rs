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
  Number(f32),
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
        Equal => {
          unreachable!("TODO sets are not cmp ed") // TODO
        }
        a => a,
      },
      (Self::Set(_), _) => Less,
      (_, Self::Set(_)) => Greater,

      (Self::Map(a), Self::Map(b)) => {
        unreachable!(format!("TODO maps are not cmp ed {:?} {:?}", a, b)) // TODO
      }
      (Self::Map(_), _) => Less,
      (_, Self::Map(_)) => Greater,

      (Self::Record(_name1, _fields1, _values1), Self::Record(_name2, _fields2, _values2)) => {
        unreachable!("TODO records are not cmp ed") // TODO
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
