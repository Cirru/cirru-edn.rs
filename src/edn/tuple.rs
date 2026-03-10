use std::sync::Arc;

use crate::Edn;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdnTupleView {
  pub tag: Arc<Edn>,
  pub enum_tag: Option<Arc<Edn>>,
  pub extra: Vec<Edn>,
}

impl From<(Arc<Edn>, Vec<Edn>)> for EdnTupleView {
  fn from((tag, extra): (Arc<Edn>, Vec<Edn>)) -> EdnTupleView {
    EdnTupleView {
      tag,
      enum_tag: None,
      extra,
    }
  }
}

impl From<(Arc<Edn>, Arc<Edn>, Vec<Edn>)> for EdnTupleView {
  fn from((enum_tag, tag, extra): (Arc<Edn>, Arc<Edn>, Vec<Edn>)) -> EdnTupleView {
    EdnTupleView {
      tag,
      enum_tag: Some(enum_tag),
      extra,
    }
  }
}

impl From<EdnTupleView> for (Arc<Edn>, Vec<Edn>) {
  fn from(x: EdnTupleView) -> (Arc<Edn>, Vec<Edn>) {
    (x.tag, x.extra)
  }
}

impl TryFrom<Edn> for EdnTupleView {
  type Error = String;

  fn try_from(data: Edn) -> Result<Self, Self::Error> {
    match data {
      Edn::Tuple(EdnTupleView { tag, enum_tag, extra }) => Ok(EdnTupleView { tag, enum_tag, extra }),
      a => Err(format!("data is not tuple: {a}")),
    }
  }
}

impl From<EdnTupleView> for Edn {
  fn from(x: EdnTupleView) -> Edn {
    Edn::Tuple(x)
  }
}

impl Ord for EdnTupleView {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self
      .enum_tag
      .cmp(&other.enum_tag)
      .then_with(|| self.tag.cmp(&other.tag))
      .then_with(|| self.extra.cmp(&other.extra))
  }
}

impl PartialOrd for EdnTupleView {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}
