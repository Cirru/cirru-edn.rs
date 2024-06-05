extern crate cirru_edn;

use cirru_edn::Edn;
use cirru_edn::EdnAnyRef;

#[test]
fn any_ref_values() {
  let a = Edn::AnyRef(EdnAnyRef::new(1));
  let b = Edn::AnyRef(EdnAnyRef::new(2));
  let c = Edn::AnyRef(EdnAnyRef::new("1"));
  let d = Edn::AnyRef(EdnAnyRef::new(1));

  assert_eq!(a, d);
  assert_ne!(a, b);
  assert_ne!(a, c);
}
