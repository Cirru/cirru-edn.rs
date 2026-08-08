// Struct

/// Struct interface for Edn::Struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdnStructView {
  pub name: Arc<str>,
  pub pairs: Vec<(EdnTag, Edn)>,
}

impl PartialOrd for EdnStructView {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(std::cmp::Ord::cmp(self, other))
  }
}

impl Ord for EdnStructView {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.name.cmp(&other.name).then_with(|| self.pairs.cmp(&other.pairs))
  }
}

impl TryFrom<Edn> for EdnStructView {
  type Error = String;

  fn try_from(data: Edn) -> Result<Self, Self::Error> {
    match data {
      Edn::Struct(EdnStructView { name, pairs }) => {
        let mut buf = vec![];
        for pair in pairs {
          buf.push((pair.0, pair.1));
        }
        Ok(EdnStructView { name, pairs: buf })
      }
      value => Err(format!("data is not struct: {value}")),
    }
  }
}

impl From<EdnStructView> for Edn {
  fn from(x: EdnStructView) -> Edn {
    Edn::Struct(EdnStructView {
      name: x.name,
      pairs: x.pairs,
    })
  }
}

use std::ops::Index;

use crate::{Edn, EdnTag};
use std::sync::Arc;
impl Index<&str> for EdnStructView {
  type Output = Edn;

  fn index(&self, index: &str) -> &Self::Output {
    for pair in self.pairs.iter() {
      if index == &*pair.0.arc_str() {
        return &pair.1;
      }
    }
    unreachable!("failed to get field: {}", index)
  }
}

impl EdnStructView {
  pub fn new(name: impl Into<Arc<str>>) -> EdnStructView {
    EdnStructView {
      name: name.into(),
      pairs: vec![],
    }
  }

  pub fn has_key(&self, key: &str) -> bool {
    for pair in self.pairs.iter() {
      if key == &*pair.0.arc_str() {
        return true;
      }
    }
    false
  }

  /// Quick helper for building a struct.
  pub fn insert(&mut self, k: impl Into<EdnTag>, v: Edn) {
    self.pairs.push((k.into(), v))
  }
}
