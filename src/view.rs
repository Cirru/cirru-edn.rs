use crate::{primes::Edn, EdnTag};

use std::{
  collections::{HashMap, HashSet},
  fmt,
};

// List

#[derive(fmt::Debug, Clone)]
pub struct EdnListView(pub Vec<Edn>);

impl TryFrom<Edn> for EdnListView {
  type Error = String;

  fn try_from(data: Edn) -> Result<Self, Self::Error> {
    match data {
      Edn::List(xs) => Ok(EdnListView(xs)),
      Edn::Nil => Ok(EdnListView(vec![])),
      a => Err(format!("data is not list: {}", a)),
    }
  }
}

impl From<Vec<Edn>> for EdnListView {
  fn from(xs: Vec<Edn>) -> EdnListView {
    EdnListView(xs)
  }
}

impl From<EdnListView> for Edn {
  fn from(x: EdnListView) -> Edn {
    Edn::List(x.0)
  }
}

impl EdnListView {
  pub fn new() -> EdnListView {
    EdnListView(vec![])
  }

  pub fn get_or_nil(&self, index: usize) -> Edn {
    if index >= self.0.len() {
      return Edn::Nil;
    }
    self.0[index].clone()
  }

  pub fn len(&self) -> usize {
    self.0.len()
  }

  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }
}

// Map

#[derive(fmt::Debug, Clone)]
pub struct EdnMapView(pub HashMap<Edn, Edn>);

impl TryFrom<Edn> for EdnMapView {
  type Error = String;

  fn try_from(data: Edn) -> Result<Self, Self::Error> {
    match data {
      Edn::Map(xs) => Ok(EdnMapView(xs)),
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

impl From<EdnMapView> for Edn {
  fn from(x: EdnMapView) -> Edn {
    Edn::Map(x.0)
  }
}

impl EdnMapView {
  pub fn new() -> EdnMapView {
    EdnMapView(HashMap::new())
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
}

// Record

#[derive(fmt::Debug, Clone)]
pub struct EdnRecordView {
  pub tag: EdnTag,
  pub pairs: Vec<(EdnTag, Edn)>,
}

impl TryFrom<Edn> for EdnRecordView {
  type Error = String;

  fn try_from(data: Edn) -> Result<Self, Self::Error> {
    match data {
      Edn::Record(t, pairs) => {
        let mut buf = vec![];
        for pair in pairs {
          buf.push((pair.0, pair.1));
        }
        Ok(EdnRecordView { tag: t, pairs: buf })
      }
      a => Err(format!("data is not record: {}", a)),
    }
  }
}

impl From<EdnRecordView> for Edn {
  fn from(x: EdnRecordView) -> Edn {
    Edn::Record(x.tag, x.pairs)
  }
}

use std::ops::Index;
impl Index<&str> for EdnRecordView {
  type Output = Edn;

  fn index(&self, index: &str) -> &Self::Output {
    for pair in self.pairs.iter() {
      if index == &*pair.0.to_str() {
        return &pair.1;
      }
    }
    unreachable!("failed to get field: {}", index)
  }
}

impl EdnRecordView {
  pub fn new(tag: EdnTag) -> EdnRecordView {
    EdnRecordView { tag, pairs: vec![] }
  }

  pub fn has_key(&self, key: &str) -> bool {
    for pair in self.pairs.iter() {
      if key == &*pair.0.to_str() {
        return true;
      }
    }
    false
  }
}

// Set

#[derive(fmt::Debug, Clone)]
pub struct EdnSetView(pub HashSet<Edn>);

impl TryFrom<Edn> for EdnSetView {
  type Error = String;

  fn try_from(data: Edn) -> Result<Self, Self::Error> {
    match data {
      Edn::Set(xs) => Ok(EdnSetView(xs)),
      Edn::Nil => Ok(EdnSetView(HashSet::new())),
      a => Err(format!("data is not set: {}", a)),
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
    Edn::Set(x.0)
  }
}

impl EdnSetView {
  pub fn new() -> EdnSetView {
    EdnSetView(HashSet::new())
  }

  pub fn contains(&self, x: &Edn) -> bool {
    self.0.contains(x)
  }
}
