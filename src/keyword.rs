//!
//! abstractions on keyword, trying to reused strings.
//! a keyword is actually represented with a number, and with a string associated.
//! and two copies of strings are saved, for fast lookup and reverse lookup.
//!
//! TODO: need more optimizations

use lazy_static::lazy_static;
use std::cmp::Eq;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::RwLock;

use std::sync::atomic;
use std::sync::atomic::AtomicUsize;

lazy_static! {
  /// use 2 maps for fast lookups
  static ref KEYWORDS_DICT: RwLock<HashMap<String, usize>> = RwLock::new(HashMap::new());
  static ref KEYWORDS_REVERSE_DICT: RwLock<HashMap<usize, String>> = RwLock::new(HashMap::new());
}

static KEYWORD_ID: AtomicUsize = AtomicUsize::new(0);

/// keywords across whole program with strings reused
#[derive(fmt::Debug, Clone)]
pub struct EdnKwd {
  /// which means there will be a limit of the count of all keywords
  id: usize,
}

impl fmt::Display for EdnKwd {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(&lookup_order_kwd_str(&self.id))
  }
}

impl Hash for EdnKwd {
  fn hash<H>(&self, _state: &mut H)
  where
    H: Hasher,
  {
    "EdnKwd:".hash(_state);
    self.id.hash(_state);
  }
}

impl EdnKwd {
  pub fn from(s: &str) -> Self {
    EdnKwd { id: load_order_key(s) }
  }
}

impl Ord for EdnKwd {
  fn cmp(&self, other: &Self) -> Ordering {
    self.id.cmp(&other.id)
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
    self.id == other.id
  }
}

/// lookup from maps, record new keywords
fn load_order_key(s: &str) -> usize {
  let mut ret: usize = 0;
  let existed = {
    let read_dict = KEYWORDS_DICT.read().unwrap();
    if read_dict.contains_key(s) {
      ret = read_dict[s].to_owned();
      true
    } else {
      false
    }
  };
  // boring logic to make sure reading lock released
  if !existed {
    let mut dict = KEYWORDS_DICT.write().unwrap();
    let mut reverse_dict = KEYWORDS_REVERSE_DICT.write().unwrap();
    ret = KEYWORD_ID.fetch_add(1, atomic::Ordering::SeqCst);

    (*dict).insert(s.to_owned(), ret);
    (*reverse_dict).insert(ret, s.to_owned());
  }
  ret
}

fn lookup_order_kwd_str(i: &usize) -> String {
  let reverse_dict = KEYWORDS_REVERSE_DICT.read().unwrap();
  reverse_dict[i].to_owned()
}
