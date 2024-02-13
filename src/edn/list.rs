use crate::Edn;

// List

/// List interface for Edn::List
#[derive(Debug, Clone, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EdnListView(pub Vec<Edn>);

impl From<Vec<Edn>> for EdnListView {
  fn from(xs: Vec<Edn>) -> EdnListView {
    EdnListView(xs)
  }
}

impl From<EdnListView> for Vec<Edn> {
  fn from(x: EdnListView) -> Vec<Edn> {
    x.0
  }
}

impl From<&[Edn]> for EdnListView {
  fn from(xs: &[Edn]) -> EdnListView {
    EdnListView(xs.to_vec())
  }
}

impl From<&Vec<Edn>> for EdnListView {
  fn from(xs: &Vec<Edn>) -> EdnListView {
    EdnListView(xs.clone())
  }
}

impl TryFrom<Edn> for EdnListView {
  type Error = String;
  fn try_from(value: Edn) -> Result<Self, Self::Error> {
    match value {
      Edn::List(xs) => Ok(xs),
      _ => Err(format!("expecting list, got: {}", value)),
    }
  }
}

impl From<EdnListView> for Edn {
  fn from(x: EdnListView) -> Edn {
    Edn::List(x)
  }
}

impl EdnListView {
  /// get reference of element
  pub fn get(&self, index: usize) -> Option<&Edn> {
    self.0.get(index)
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

  pub fn push(&mut self, x: Edn) {
    self.0.push(x)
  }
}
