//! AnyRef is designed to hold any Rust data, which is used in Clacit FFIs.

use std::{
  any::Any,
  fmt::Debug,
  sync::{Arc, RwLock},
};

/// https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=c39e1eef6c8c10e973fa629103b4a0b1
pub trait DynEq: Debug {
  fn as_any(&self) -> &dyn Any;
  fn do_eq(&self, rhs: &dyn DynEq) -> bool;
}

impl<T> DynEq for T
where
  T: PartialEq + Debug + 'static,
{
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn do_eq(&self, rhs: &dyn DynEq) -> bool {
    if let Some(rhs_concrete) = rhs.as_any().downcast_ref::<Self>() {
      self == rhs_concrete
    } else {
      false
    }
  }
}

impl PartialEq for dyn DynEq {
  fn eq(&self, rhs: &Self) -> bool {
    self.do_eq(rhs)
  }
}

/// data inside any-ref is allowed to be mutable
#[derive(Debug, Clone)]
pub struct EdnAnyRef(pub Arc<RwLock<dyn DynEq>>);

/// cannot predict behavior yet, but to bypass type checking
unsafe impl Send for EdnAnyRef {}
/// cannot predict behavior yet, but to bypass type checking
unsafe impl Sync for EdnAnyRef {}

impl PartialEq for EdnAnyRef {
  fn eq(&self, other: &Self) -> bool {
    if std::ptr::addr_eq(&self, &other) {
      true
    } else {
      let a = self.0.read().expect("read any-ref");
      let b = other.0.read().expect("read any-ref");
      a.do_eq(&*b)
    }
  }
}

impl Eq for EdnAnyRef {}

impl EdnAnyRef {
  pub fn new<T: ToOwned + DynEq + 'static>(d: T) -> Self {
    EdnAnyRef(Arc::new(RwLock::new(d)))
  }
}
