use crate::edn::Edn;

use std::{collections::HashSet, fmt};

// Set

#[derive(fmt::Debug, Clone, Default, PartialEq, Eq)]
pub struct EdnSetView(pub HashSet<Edn>);

impl TryFrom<Edn> for EdnSetView {
  type Error = String;

  fn try_from(data: Edn) -> Result<Self, Self::Error> {
    match data {
      Edn::Set(xs) => Ok(xs),
      Edn::Nil => Ok(EdnSetView(HashSet::new())),
      a => Err(format!("data is not set: {a}")),
    }
  }
}

impl From<HashSet<Edn>> for EdnSetView {
  fn from(xs: HashSet<Edn>) -> EdnSetView {
    EdnSetView(xs)
  }
}

impl From<EdnSetView> for Edn {
  fn from(x: EdnSetView) -> Edn {
    Edn::Set(EdnSetView(x.0))
  }
}

impl EdnSetView {
  pub fn contains(&self, x: &Edn) -> bool {
    self.0.contains(x)
  }

  pub fn insert(&mut self, x: Edn) {
    self.0.insert(x);
  }

  pub fn len(&self) -> usize {
    self.0.len()
  }

  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }
}
