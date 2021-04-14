use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(PartialOrd, PartialEq, fmt::Debug)]
pub enum CirruEdn {
  CirruEdnNil,
  CirruEdnBool(bool),
  CirruEdnNumber(f32),
  CirruEdnSymbol(String),
  CirruEdnKeyword(String),
  CirruEdnString(String),
  CirruEdnList(Vec<CirruEdn>),
  // CirruEdnSet(HashSet<CirruEdn>),
  // TODO
}

impl Hash for CirruEdn {
  fn hash<H>(&self, _state: &mut H)
  where
    H: Hasher,
  {
    // TODO
  }
}
