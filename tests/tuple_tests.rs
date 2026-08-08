use cirru_edn::{Edn, format, parse};
use std::sync::Arc;

#[test]
fn test_enum_symbols_and_legacy_tags() {
  let value = Edn::typed_enum("e", "a", vec![Edn::Number(1.0)]);
  let expected_str = "\n%:: 'e 'a 1\n";
  assert_eq!(format(&value, true).unwrap(), expected_str);

  let parsed = parse("%:: :e :a 1").unwrap();
  assert_eq!(value, parsed);

  if let Edn::Enum(view) = parsed {
    assert_eq!(view.type_name, Some(Arc::from("e")));
    assert_eq!(view.variant, Arc::from("a"));
    assert_eq!(view.extra, vec![Edn::Number(1.0)]);
  } else {
    panic!("not an enum");
  }
}

#[test]
fn test_enum_serde() {
  let value = Edn::typed_enum("e", "a", vec![Edn::Number(1.0)]);
  let edn_encoded = cirru_edn::to_edn(&value).unwrap();
  let reconstructed: Edn = cirru_edn::from_edn(edn_encoded).unwrap();
  assert_eq!(value, reconstructed);
}

#[test]
fn test_enum_comparison() {
  let t1 = Edn::typed_enum("e1", "a", vec![]);
  let t2 = Edn::typed_enum("e2", "a", vec![]);
  assert!(t1 < t2);

  let t3 = Edn::enum_value("a", vec![]);
  // None < Some
  assert!(t3 < t1);
}
