extern crate cirru_edn;

use std::collections::HashSet;

use cirru_edn::{Edn, EdnListView, EdnMapView, EdnRecordView, EdnSetView, EdnTag};

#[test]
fn building_record() {
  let mut record = EdnRecordView::new(EdnTag::new("a"));
  record.insert("b", Edn::Number(1.0));
  record.insert("c", Edn::Number(2.0));

  assert_eq!(
    Edn::Record(EdnRecordView {
      tag: EdnTag::new("a"),
      pairs: vec![
        (EdnTag::new("b"), Edn::Number(1.0)),
        (EdnTag::new("c"), Edn::Number(2.0)),
      ],
    }),
    Edn::from(record)
  );
}

#[test]
fn building_map() {
  let mut map = EdnMapView::default();
  map.insert_key("a", Edn::Number(1.0));
  map.insert_key("b", Edn::Number(2.0));

  assert_eq!(
    Edn::map_from_iter(vec![
      (Edn::tag("a"), Edn::Number(1.0)),
      (Edn::tag("b"), Edn::Number(2.0)),
    ],),
    Edn::from(map)
  );
}

#[test]
fn building_map_with_str_fields() {
  let mut map = EdnMapView::default();
  map.insert(Edn::str("a"), Edn::Number(1.0));
  map.insert(Edn::str("b"), Edn::Number(2.0));

  assert_eq!(
    Edn::map_from_iter(vec![
      (Edn::str("a"), Edn::Number(1.0)),
      (Edn::str("b"), Edn::Number(2.0)),
    ],),
    Edn::from(map)
  );
}

#[test]
fn building_list() {
  let mut list = EdnListView::default();
  list.push(Edn::Number(1.0));
  list.push(Edn::Number(2.0));

  assert_eq!(Edn::from(vec![Edn::Number(1.0), Edn::Number(2.0)]), Edn::List(list));
}

#[test]
fn building_set() {
  let mut set = EdnSetView::default();
  set.insert(Edn::Number(1.0));
  set.insert(Edn::Number(2.0));

  let mut s = HashSet::new();
  s.insert(Edn::Number(1.0));
  s.insert(Edn::Number(2.0));
  assert_eq!(Edn::from(s), Edn::from(set));
}
