// Map

use std::collections::HashMap;

use crate::{Edn, EdnTag};

/// Map interface for Edn::Map
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EdnMapView(pub HashMap<Edn, Edn>);

impl TryFrom<Edn> for EdnMapView {
  type Error = String;

  fn try_from(data: Edn) -> Result<Self, Self::Error> {
    match data {
      Edn::Map(xs) => Ok(xs),
      Edn::Nil => Ok(EdnMapView(HashMap::new())),
      a => Err(format!("data is not map: {}", a)),
    }
  }
}

impl From<HashMap<Edn, Edn>> for EdnMapView {
  fn from(xs: HashMap<Edn, Edn>) -> EdnMapView {
    EdnMapView(xs)
  }
}

impl From<EdnMapView> for HashMap<Edn, Edn> {
  fn from(x: EdnMapView) -> HashMap<Edn, Edn> {
    x.0
  }
}

impl From<EdnMapView> for Edn {
  fn from(x: EdnMapView) -> Edn {
    Edn::Map(EdnMapView(x.0))
  }
}

impl EdnMapView {
  /// get reference of element
  pub fn get(&self, key: &str) -> Option<&Edn> {
    self.0.get(&Edn::str(key))
  }

  /// regardless of key in string or tag
  pub fn get_or_nil(&self, key: &str) -> Edn {
    self
      .0
      .get(&Edn::str(key))
      .cloned()
      .or_else(|| self.0.get(&Edn::tag(key)).cloned())
      .unwrap_or(Edn::Nil)
  }

  pub fn contains_key(&self, key: &str) -> bool {
    self.0.contains_key(&Edn::str(key)) || self.0.contains_key(&Edn::tag(key))
  }

  pub fn insert(&mut self, k: Edn, v: Edn) {
    self.0.insert(k, v);
  }

  /// takes k that impl Into<EdnTag>
  pub fn insert_key(&mut self, k: impl Into<EdnTag>, v: Edn) {
    self.0.insert(k.into().into(), v);
  }

  pub fn len(&self) -> usize {
    self.0.len()
  }

  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }
}
