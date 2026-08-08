use std::sync::Arc;

use crate::Edn;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdnEnumView {
  pub variant: Arc<str>,
  pub type_name: Option<Arc<str>>,
  pub extra: Vec<Edn>,
}

impl From<(Arc<str>, Vec<Edn>)> for EdnEnumView {
  fn from((variant, extra): (Arc<str>, Vec<Edn>)) -> EdnEnumView {
    EdnEnumView {
      variant,
      type_name: None,
      extra,
    }
  }
}

impl From<(Arc<str>, Arc<str>, Vec<Edn>)> for EdnEnumView {
  fn from((type_name, variant, extra): (Arc<str>, Arc<str>, Vec<Edn>)) -> EdnEnumView {
    EdnEnumView {
      variant,
      type_name: Some(type_name),
      extra,
    }
  }
}

impl From<EdnEnumView> for (Arc<str>, Vec<Edn>) {
  fn from(x: EdnEnumView) -> (Arc<str>, Vec<Edn>) {
    (x.variant, x.extra)
  }
}

impl TryFrom<Edn> for EdnEnumView {
  type Error = String;

  fn try_from(data: Edn) -> Result<Self, Self::Error> {
    match data {
      Edn::Enum(EdnEnumView {
        variant,
        type_name,
        extra,
      }) => Ok(EdnEnumView {
        variant,
        type_name,
        extra,
      }),
      value => Err(format!("data is not enum: {value}")),
    }
  }
}

impl From<EdnEnumView> for Edn {
  fn from(x: EdnEnumView) -> Edn {
    Edn::Enum(x)
  }
}

impl Ord for EdnEnumView {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self
      .type_name
      .cmp(&other.type_name)
      .then_with(|| self.variant.cmp(&other.variant))
      .then_with(|| self.extra.cmp(&other.extra))
  }
}

impl PartialOrd for EdnEnumView {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}
