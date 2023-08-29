use crate::primes::Edn;

use std::{
  collections::{HashMap, HashSet},
  fmt,
};

// List

#[derive(fmt::Debug, Clone)]
pub struct EdnListView {
  data: Edn,
}

impl From<Edn> for EdnListView {
  fn from(data: Edn) -> Self {
    EdnListView { data }
  }
}
impl From<EdnListView> for Edn {
  fn from(x: EdnListView) -> Edn {
    x.data
  }
}

impl EdnListView {
  pub fn read(self) -> Vec<Edn> {
    match self.data {
      Edn::List(xs) => xs,
      a => unreachable!("data is not list: {}", a),
    }
  }

  pub fn get_or_nil(&self, index: usize) -> Edn {
    match &self.data {
      Edn::List(xs) => {
        if index < xs.len() {
          xs[index].clone()
        } else {
          Edn::Nil
        }
      }
      a => unreachable!("data is not list: {}", a),
    }
  }
}

// Map

#[derive(fmt::Debug, Clone)]
pub struct EdnMapView {
  data: Edn,
}

impl From<Edn> for EdnMapView {
  fn from(data: Edn) -> Self {
    EdnMapView { data }
  }
}

impl From<EdnMapView> for Edn {
  fn from(x: EdnMapView) -> Edn {
    x.data
  }
}

impl EdnMapView {
  pub fn read(self) -> HashMap<Edn, Edn> {
    match self.data {
      Edn::Map(xs) => xs,
      a => unreachable!("data is not a map: {}", a),
    }
  }

  /// regardless of key in string or tag
  pub fn get_or_nil(&self, key: &str) -> Edn {
    match &self.data {
      Edn::Map(xs) => xs
        .get(&Edn::str(key))
        .cloned()
        .or_else(|| xs.get(&Edn::tag(key)).cloned())
        .unwrap_or(Edn::Nil),
      a => unreachable!("data is not a map: {}", a),
    }
  }
}

// Record

#[derive(fmt::Debug, Clone)]
pub struct EdnRecordView {
  data: Edn,
}

impl From<Edn> for EdnRecordView {
  fn from(data: Edn) -> Self {
    EdnRecordView { data }
  }
}

impl From<EdnRecordView> for Edn {
  fn from(x: EdnRecordView) -> Edn {
    x.data
  }
}

use std::ops::Index;
impl Index<&str> for EdnRecordView {
  type Output = Edn;

  fn index(&self, index: &str) -> &Self::Output {
    match &self.data {
      Edn::Record(_t, pairs) => {
        for pair in pairs {
          if index == &*pair.0.to_str() {
            return &pair.1;
          }
        }
        unreachable!("missing key in record: {}", index)
      }
      a => unreachable!("data is not a record: {}", a),
    }
  }
}

impl EdnRecordView {}

// Set

#[derive(fmt::Debug, Clone)]
pub struct EdnSetView {
  data: Edn,
}

impl From<Edn> for EdnSetView {
  fn from(data: Edn) -> Self {
    EdnSetView { data }
  }
}

impl From<EdnSetView> for Edn {
  fn from(x: EdnSetView) -> Edn {
    x.data
  }
}

impl EdnSetView {
  pub fn read(self) -> HashSet<Edn> {
    match self.data {
      Edn::Set(xs) => xs,
      a => unreachable!("failed to convert to set: {}", a),
    }
  }
}
