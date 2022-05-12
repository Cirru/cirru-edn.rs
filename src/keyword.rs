//!
//! abstractions on keyword, trying to reused strings.
//! a keyword is actually represented with a number, and with a string associated.
//! and two copies of strings are saved, for fast lookup and reverse lookup.
//!
//! TODO: need more optimizations

use std::{
  cmp::Eq,
  cmp::Ordering,
  fmt,
  hash::{Hash, Hasher},
};

/// keywords across whole program with strings reused
#[derive(fmt::Debug, Clone)]
pub struct EdnKwd(
  /// which means there will be a limit of the count of all keywords
  Box<str>,
);

impl fmt::Display for EdnKwd {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(&self.0)
  }
}

impl Hash for EdnKwd {
  fn hash<H>(&self, _state: &mut H)
  where
    H: Hasher,
  {
    "EdnKwd:".hash(_state);
    self.0.hash(_state);
  }
}

impl From<&str> for EdnKwd {
  fn from(s: &str) -> Self {
    Self(Box::from(s))
  }
}

impl EdnKwd {
  pub fn new(s: &str) -> Self {
    EdnKwd(s.into())
  }

  /// get Box<str> from inside
  pub fn to_str(&self) -> Box<str> {
    self.0.to_owned()
  }
}

impl Ord for EdnKwd {
  fn cmp(&self, other: &Self) -> Ordering {
    self.0.cmp(&other.0)
  }
}

impl PartialOrd for EdnKwd {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Eq for EdnKwd {}

impl PartialEq for EdnKwd {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}
