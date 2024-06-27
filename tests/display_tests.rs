extern crate cirru_edn;

use std::{sync::Arc, vec};

use cirru_edn::{Edn, EdnListView, EdnRecordView, EdnTag};

#[test]
fn display_data() {
  let r = Edn::Record(EdnRecordView {
    tag: EdnTag::new("Demo"),
    pairs: vec![
      (EdnTag::new("a"), Edn::Number(1.0)),
      (EdnTag::new("a"), Edn::Number(2.0)),
    ],
  });

  assert_eq!(format!("{r}"), "(%{} :Demo (:a 1) (:a 2))");

  let t = Edn::from((Arc::new(Edn::tag("t")), vec![]));
  assert_eq!(format!("{t}"), "(:: :t)");

  let t2 = Edn::from((Arc::new(Edn::tag("t")), vec![Edn::Number(1.0), Edn::Number(2.0)]));
  assert_eq!(format!("{t2}"), "(:: :t 1 2)");
}

#[test]
fn display_with_cjk() {
  let r = Edn::List(EdnListView(vec![Edn::str("你好"), Edn::str("世界"), Edn::str("海 洋")]));

  assert_eq!(format!("{r}"), "([] |你好 |世界 \"|海 洋\")");
}
