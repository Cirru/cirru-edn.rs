//! Tag abstractions for EDN values.
//!
//! Tags (previously called "keywords") are named constants that can be used
//! as map keys, enum values, or identifiers. This module provides efficient
//! string reuse through Arc<str> to minimize memory allocation.

use std::{
  cmp::Eq,
  cmp::Ordering,
  fmt,
  hash::{Hash, Hasher},
  sync::Arc,
};

/// Tags across whole program with strings reused for efficiency.
///
/// A tag is similar to a keyword in other Lisp dialects - it's a
/// self-evaluating named constant. Tags are commonly used as:
/// - Map keys (like `:name`, `:age`)
/// - Enum-like values (like `:success`, `:error`)
/// - Type identifiers in records
///
/// # Examples
///
/// ```
/// use cirru_edn::EdnTag;
///
/// let tag1 = EdnTag::new("status");
/// let tag2 = EdnTag::from("active");
/// ```
#[derive(fmt::Debug, Clone)]
pub struct EdnTag(
  /// The tag string - there will be a practical limit on the count of all tags
  pub Arc<str>,
);

impl fmt::Display for EdnTag {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(&self.0)
  }
}

impl Hash for EdnTag {
  fn hash<H>(&self, _state: &mut H)
  where
    H: Hasher,
  {
    "EdnTag:".hash(_state);
    self.0.hash(_state);
  }
}

impl From<&str> for EdnTag {
  fn from(s: &str) -> Self {
    Self(Arc::from(s))
  }
}

impl EdnTag {
  /// Create a new tag from a string.
  ///
  /// # Examples
  ///
  /// ```
  /// use cirru_edn::EdnTag;
  ///
  /// let tag = EdnTag::new("my-tag");
  /// assert_eq!(tag.ref_str(), "my-tag");
  /// ```
  pub fn new<T: Into<Arc<str>>>(s: T) -> Self {
    EdnTag(s.into())
  }

  /// Get the inner Arc<str> reference.
  ///
  /// This provides access to the underlying string data without cloning.
  pub fn arc_str(&self) -> Arc<str> {
    (*self.0).into()
  }

  /// Get a string slice reference for comparison.
  ///
  /// This is the most efficient way to compare tag content.
  pub fn ref_str(&self) -> &str {
    &self.0
  }

  /// Check if the tag matches a string slice.
  ///
  /// This is more efficient than converting the tag to a string.
  ///
  /// # Examples
  ///
  /// ```
  /// use cirru_edn::EdnTag;
  ///
  /// let tag = EdnTag::new("status");
  /// assert!(tag.matches("status"));
  /// assert!(!tag.matches("other"));
  /// ```
  pub fn matches(&self, s: &str) -> bool {
    self.0.as_ref() == s
  }

  /// Get the length of the tag string.
  pub fn len(&self) -> usize {
    self.0.len()
  }

  /// Check if the tag is empty.
  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }
}

impl Ord for EdnTag {
  fn cmp(&self, other: &Self) -> Ordering {
    self.0.cmp(&other.0)
  }
}

impl PartialOrd for EdnTag {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Eq for EdnTag {}

impl PartialEq for EdnTag {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}
