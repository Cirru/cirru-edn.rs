extern crate cirru_edn;

use std::vec;

use cirru_edn::{Edn, EdnTag};

#[test]
fn display_data() {
  let r = Edn::Record(
    EdnTag::new("Demo"),
    vec![
      (EdnTag::new("a"), Edn::Number(1.0)),
      (EdnTag::new("a"), Edn::Number(2.0)),
    ],
  );

  assert_eq!(format!("{r}"), "(%{} :Demo (:a 1) (:a 2))");

  let t = Edn::Tuple(Box::new(Edn::tag("t")), vec![]);
  assert_eq!(format!("{t}"), "(:: :t)");

  let t2 = Edn::Tuple(Box::new(Edn::tag("t")), vec![Edn::Number(1.0), Edn::Number(2.0)]);
  assert_eq!(format!("{t2}"), "(:: :t 1 2)");
}
