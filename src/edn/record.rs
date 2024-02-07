// Record

/// Record interface for Edn::Record
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdnRecordView {
  pub tag: EdnTag,
  pub pairs: Vec<(EdnTag, Edn)>,
}

impl PartialOrd for EdnRecordView {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(std::cmp::Ord::cmp(self, other))
  }
}

impl Ord for EdnRecordView {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.tag.cmp(&other.tag).then_with(|| self.pairs.cmp(&other.pairs))
  }
}

impl TryFrom<Edn> for EdnRecordView {
  type Error = String;

  fn try_from(data: Edn) -> Result<Self, Self::Error> {
    match data {
      Edn::Record(EdnRecordView { tag: t, pairs }) => {
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
    Edn::Record(EdnRecordView {
      tag: x.tag,
      pairs: x.pairs,
    })
  }
}

use std::ops::Index;

use crate::{Edn, EdnTag};
impl Index<&str> for EdnRecordView {
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

impl EdnRecordView {
  pub fn new(tag: EdnTag) -> EdnRecordView {
    EdnRecordView { tag, pairs: vec![] }
  }

  pub fn has_key(&self, key: &str) -> bool {
    for pair in self.pairs.iter() {
      if key == &*pair.0.arc_str() {
        return true;
      }
    }
    false
  }

  /// quick hand for building record
  pub fn insert(&mut self, k: impl Into<EdnTag>, v: Edn) {
    self.pairs.push((k.into(), v))
  }
}
