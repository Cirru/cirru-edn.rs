//!
//! abstractions on tags(previously called "keyword"), trying to reused strings.
//! a tag is represented with a number, and with a string associated.
//! and two copies of strings are saved, for fast lookup and reverse lookup.
//!
//! TODO: need more optimizations

use std::{
  cmp::Eq,
  cmp::Ordering,
  fmt,
  hash::{Hash, Hasher},
  sync::Arc,
};

/// tags across whole program with strings reused
#[derive(fmt::Debug, Clone)]
pub struct EdnTag(
  /// which means there will be a limit of the count of all tags
  pub Arc<str>,
);

impl fmt::Display for EdnTag {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(&self.0)
  }
}

impl Hash for EdnTag {
  fn hash<H>(&self, _state: &mut H)
  where
    H: Hasher,
  {
    "EdnTag:".hash(_state);
    self.0.hash(_state);
  }
}

impl From<&str> for EdnTag {
  fn from(s: &str) -> Self {
    Self(Arc::from(s))
  }
}

impl EdnTag {
  pub fn new<T: Into<Arc<str>>>(s: T) -> Self {
    EdnTag(s.into())
  }

  /// get Arc<str> from inside
  pub fn arc_str(&self) -> Arc<str> {
    (*self.0).into()
  }

  /// use &str for comparison
  pub fn ref_str(&self) -> &str {
    &self.0
  }
}

impl Ord for EdnTag {
  fn cmp(&self, other: &Self) -> Ordering {
    self.0.cmp(&other.0)
  }
}

impl PartialOrd for EdnTag {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Eq for EdnTag {}

impl PartialEq for EdnTag {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}
