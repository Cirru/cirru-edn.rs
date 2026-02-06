use cirru_edn::{Edn, parse, format};
use std::sync::Arc;

#[test]
fn test_tuple_enum() {
  let tuple = Edn::enum_tuple(Edn::tag("e"), Edn::tag("a"), vec![Edn::Number(1.0)]);
  let expected_str = "\n%:: :e :a 1\n";
  assert_eq!(format(&tuple, true).unwrap(), expected_str);

  let parsed = parse("%:: :e :a 1").unwrap();
  assert_eq!(tuple, parsed);

  if let Edn::Tuple(view) = parsed {
    assert_eq!(view.enum_tag, Some(Arc::new(Edn::tag("e"))));
    assert_eq!(view.tag, Arc::new(Edn::tag("a")));
    assert_eq!(view.extra, vec![Edn::Number(1.0)]);
  } else {
    panic!("not a tuple");
  }
}

#[test]
fn test_tuple_serde() {
  let tuple = Edn::enum_tuple(Edn::tag("e"), Edn::tag("a"), vec![Edn::Number(1.0)]);
  let edn_encoded = cirru_edn::to_edn(&tuple).unwrap();
  let reconstructed: Edn = cirru_edn::from_edn(edn_encoded).unwrap();
  assert_eq!(tuple, reconstructed);
}

#[test]
fn test_tuple_comparison() {
  let t1 = Edn::enum_tuple(Edn::tag("e1"), Edn::tag("a"), vec![]);
  let t2 = Edn::enum_tuple(Edn::tag("e2"), Edn::tag("a"), vec![]);
  assert!(t1 < t2);

  let t3 = Edn::tuple(Edn::tag("a"), vec![]);
  // None < Some
  assert!(t3 < t1);
}
