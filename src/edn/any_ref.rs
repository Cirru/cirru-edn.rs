//! AnyRef is designed to hold any Rust data, which is used in Clacit FFIs.

use std::{
  any::Any,
  sync::{Arc, RwLock},
};

/// data inside any-ref is allowed to be mutable
#[derive(Debug, Clone)]
pub struct EdnAnyRef(pub Arc<RwLock<dyn Any>>);

/// cannot predict behavior yet, but to bypass type checking
unsafe impl Send for EdnAnyRef {}
/// cannot predict behavior yet, but to bypass type checking
unsafe impl Sync for EdnAnyRef {}

impl PartialEq for EdnAnyRef {
  fn eq(&self, other: &Self) -> bool {
    std::ptr::addr_eq(&self, &other)
  }
}

impl Eq for EdnAnyRef {}

impl EdnAnyRef {
  pub fn new<T: ToOwned + Any>(d: T) -> Self {
    EdnAnyRef(Arc::new(RwLock::new(d)))
  }
}
