use std::{
  cmp::{Eq, Ordering, Ordering::*},
  collections::{HashMap, HashSet},
  convert::TryInto,
  fmt,
  hash::{Hash, Hasher},
};

use cirru_parser::Cirru;

use crate::keyword::EdnKwd;

/// Data format based on subset of EDN, but in Cirru syntax.
/// different parts are quote and Record.
#[derive(fmt::Debug, Clone)]
pub enum Edn {
  Nil,
  Bool(bool),
  Number(f64),
  Symbol(Box<str>),
  Keyword(EdnKwd),
  Str(Box<str>), // name collision
  Quote(Cirru),
  Tuple(Box<(Edn, Edn)>),
  List(Vec<Edn>),
  Set(HashSet<Edn>),
  Map(HashMap<Edn, Edn>),
  Record(EdnKwd, Vec<(EdnKwd, Edn)>),
  Buffer(Vec<u8>),
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
        if is_simple_token(s) {
          f.write_str(&format!("|{}", s))
        } else {
          f.write_str(&format!("\"|{}\"", s))
        }
      }
      Self::Quote(v) => f.write_str(&format!("(quote {})", v)),
      Self::Tuple(pair) => f.write_str(&format!("(:: {} {})", pair.0, pair.1)),
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
      Self::Record(name, entries) => {
        f.write_str(&format!("(%{{}} {}", name))?;

        for entry in entries {
          f.write_str(&format!("({} {})", Edn::Keyword(entry.0.to_owned()), entry.1))?;
        }

        f.write_str(")")
      }
      Self::Buffer(buf) => {
        f.write_str("(buf")?;
        for b in buf {
          f.write_str(" ")?;
          f.write_str(&hex::encode(vec![b.to_owned()]))?;
        }
        f.write_str(")")
      }
    }
  }
}

fn is_simple_token(tok: &str) -> bool {
  for s in tok.chars() {
    if !matches!(s, '0'..='9' | 'A'..='Z'| 'a'..='z'|  '-' | '?' | '.'| '$' | ',') {
      return false;
    }
  }
  true
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
      Self::Tuple(pair) => {
        "tuple".hash(_state);
        pair.0.hash(_state);
        pair.1.hash(_state);
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
      Self::Record(name, entries) => {
        "record:".hash(_state);
        name.hash(_state);
        entries.hash(_state);
      }
      Self::Buffer(buf) => {
        "buffer:".hash(_state);
        for b in buf {
          b.hash(_state);
        }
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

      (Self::Tuple(a), Self::Tuple(b)) => match a.0.cmp(&b.0) {
        Less => Less,
        Greater => Greater,
        Equal => a.1.cmp(&b.1),
      },
      (Self::Tuple(..), _) => Less,
      (_, Self::Tuple(..)) => Greater,

      (Self::List(a), Self::List(b)) => a.cmp(b),
      (Self::List(_), _) => Less,
      (_, Self::List(_)) => Greater,

      (Self::Buffer(a), Self::Buffer(b)) => a.cmp(b),
      (Self::Buffer(_), _) => Less,
      (_, Self::Buffer(_)) => Greater,

      (Self::Set(a), Self::Set(b)) => match a.len().cmp(&b.len()) {
        Equal => unreachable!("TODO sets are not cmp ed"), // TODO
        a => a,
      },
      (Self::Set(_), _) => Less,
      (_, Self::Set(_)) => Greater,

      (Self::Map(a), Self::Map(b)) => {
        match a.len().cmp(&b.len()) {
          Equal => unreachable!("TODO maps are not cmp ed {:?} {:?}", a, b), // TODO
          a => a,
        }
      }
      (Self::Map(_), _) => Less,
      (_, Self::Map(_)) => Greater,

      (Self::Record(name1, entries1), Self::Record(name2, entries2)) => match name1.cmp(name2) {
        Equal => entries1.cmp(entries2),
        a => a,
      },
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
      (Self::Number(a), Self::Number(b)) => (a - b).abs() < f64::EPSILON,
      (Self::Symbol(a), Self::Symbol(b)) => a == b,
      (Self::Keyword(a), Self::Keyword(b)) => a == b,
      (Self::Str(a), Self::Str(b)) => a == b,
      (Self::Quote(a), Self::Quote(b)) => a == b,
      (Self::Tuple(a), Self::Tuple(b)) => a.0 == b.0 && a.1 == b.1,
      (Self::List(a), Self::List(b)) => a == b,
      (Self::Buffer(a), Self::Buffer(b)) => a == b,
      (Self::Set(a), Self::Set(b)) => a == b,
      (Self::Map(a), Self::Map(b)) => a == b,
      (Self::Record(name1, entries1), Self::Record(name2, entries2)) => name1 == name2 && entries1 == entries2,
      (_, _) => false,
    }
  }
}

/// Support reading from EDN
impl Edn {
  /// create new string
  pub fn str<T: Into<String>>(s: T) -> Self {
    Edn::Str(s.into().into_boxed_str())
  }
  /// create new keyword
  pub fn kwd(s: &str) -> Self {
    Edn::Keyword(EdnKwd::from(s))
  }
  /// create new symbol
  pub fn sym<T: Into<String>>(s: T) -> Self {
    Edn::Symbol(s.into().into_boxed_str())
  }
  /// create new tuple
  pub fn tuple(a: Self, b: Self) -> Self {
    Edn::Tuple(Box::new((a, b)))
  }
  pub fn read_string(&self) -> Result<String, String> {
    match self {
      Edn::Str(s) => Ok((&**s).to_owned()),
      a => Err(format!("failed to convert to string: {}", a)),
    }
  }
  pub fn read_symbol_string(&self) -> Result<String, String> {
    match self {
      Edn::Symbol(s) => Ok((&**s).to_owned()),
      a => Err(format!("failed to convert to symbol: {}", a)),
    }
  }
  pub fn read_keyword_string(&self) -> Result<String, String> {
    match self {
      Edn::Keyword(s) => Ok(s.to_string()),
      a => Err(format!("failed to convert to keyword: {}", a)),
    }
  }
  pub fn read_str(&self) -> Result<Box<str>, String> {
    match self {
      Edn::Str(s) => Ok(s.to_owned()),
      a => Err(format!("failed to convert to string: {}", a)),
    }
  }
  pub fn read_symbol_str(&self) -> Result<Box<str>, String> {
    match self {
      Edn::Symbol(s) => Ok(s.to_owned()),
      a => Err(format!("failed to convert to symbol: {}", a)),
    }
  }
  pub fn read_kwd_str(&self) -> Result<Box<str>, String> {
    match self {
      Edn::Keyword(s) => Ok(s.to_str()),
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
      Edn::Nil => Err(String::from("cannot read list from nil")),
      a => Err(format!("failed to convert to vec: {}", a)),
    }
  }

  pub fn read_list_or_nil(&self) -> Result<Vec<Edn>, String> {
    match self {
      Edn::List(xs) => Ok(xs.to_owned()),
      Edn::Nil => Ok(vec![]),
      a => Err(format!("failed to convert to vec: {}", a)),
    }
  }

  pub fn read_set(&self) -> Result<HashSet<Edn>, String> {
    match self {
      Edn::Set(xs) => Ok(xs.to_owned()),
      Edn::Nil => Err(String::from("cannot read set from nil")),
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
      Edn::Nil => Err(String::from("cannot read map from nil")),
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
    match self {
      Edn::Map(xs) => {
        if xs.contains_key(&Edn::kwd(k)) {
          Ok(xs[&Edn::kwd(k)].to_owned())
        } else if xs.contains_key(&Edn::Str(k.to_owned().into_boxed_str())) {
          Ok(xs[&Edn::Str(k.into())].to_owned())
        } else {
          Ok(Edn::Nil)
        }
      }
      a => Err(format!("target is not map: {}", a)),
    }
  }
}

impl TryInto<String> for Edn {
  type Error = String;
  fn try_into(self) -> Result<String, Self::Error> {
    match self {
      Edn::Str(s) => Ok((&*s).to_owned()),
      a => Err(format!("failed to convert to string: {}", a)),
    }
  }
}

impl TryInto<Box<str>> for Edn {
  type Error = String;
  fn try_into(self) -> Result<Box<str>, Self::Error> {
    match self {
      Edn::Str(s) => Ok((&*s).into()),
      a => Err(format!("failed to convert to box str: {}", a)),
    }
  }
}

impl TryInto<bool> for Edn {
  type Error = String;
  fn try_into(self) -> Result<bool, Self::Error> {
    match self {
      Edn::Bool(s) => Ok(s),
      a => Err(format!("failed to convert to bool: {}", a)),
    }
  }
}

impl TryInto<f64> for Edn {
  type Error = String;
  fn try_into(self) -> Result<f64, Self::Error> {
    match self {
      Edn::Number(s) => Ok(s),
      a => Err(format!("failed to convert to number: {}", a)),
    }
  }
}

impl TryInto<Cirru> for Edn {
  type Error = String;
  fn try_into(self) -> Result<Cirru, Self::Error> {
    match self {
      Edn::Quote(s) => Ok(s),
      a => Err(format!("failed to convert to cirru code: {}", a)),
    }
  }
}

impl TryInto<Vec<Edn>> for Edn {
  type Error = String;
  fn try_into(self) -> Result<Vec<Edn>, Self::Error> {
    match self {
      Edn::List(s) => Ok(s),
      Edn::Nil => Ok(vec![]),
      a => Err(format!("failed to convert to vec: {}", a)),
    }
  }
}

impl TryInto<HashSet<Edn>> for Edn {
  type Error = String;
  fn try_into(self) -> Result<HashSet<Edn>, Self::Error> {
    match self {
      Edn::Set(s) => Ok(s),
      Edn::Nil => Ok(HashSet::new()),
      a => Err(format!("failed to convert to vec: {}", a)),
    }
  }
}

impl TryInto<HashMap<Edn, Edn>> for Edn {
  type Error = String;
  fn try_into(self) -> Result<HashMap<Edn, Edn>, Self::Error> {
    match self {
      Edn::Map(s) => Ok(s),
      Edn::Nil => Ok(HashMap::new()),
      a => Err(format!("failed to convert to vec: {}", a)),
    }
  }
}
