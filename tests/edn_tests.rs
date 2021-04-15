use cirru_edn::CirruEdn;
use cirru_edn::CirruEdn::*;
use cirru_edn::{parse_cirru_edn, write_cirru_edn};
use std::collections::HashSet;

#[test]
fn edn_parsing() {
  assert_eq!(Ok(CirruEdnNil), parse_cirru_edn(String::from("do nil")));
  assert_eq!(
    Ok(CirruEdnBool(true)),
    parse_cirru_edn(String::from("do true"))
  );
  assert_eq!(
    Ok(CirruEdnBool(false)),
    parse_cirru_edn(String::from("do false"))
  );

  assert_eq!(
    Ok(CirruEdnSymbol(String::from("a"))),
    parse_cirru_edn(String::from("do 'a"))
  );
  assert_eq!(
    Ok(CirruEdnKeyword(String::from("k"))),
    parse_cirru_edn(String::from("do :k"))
  );
  assert_eq!(
    Ok(CirruEdnString(String::from("s"))),
    parse_cirru_edn(String::from("do |s"))
  );

  assert_eq!(
    Ok(CirruEdnString(String::from("a b\n c"))),
    parse_cirru_edn(String::from(r#"do "|a b\n c""#))
  );

  assert_eq!(
    Ok(CirruEdnNumber(2.0)),
    parse_cirru_edn(String::from("do 2"))
  );
  assert_eq!(
    Ok(CirruEdnNumber(-2.2)),
    parse_cirru_edn(String::from("do -2.2"))
  );
}

#[test]
fn list_parsing() {
  assert_eq!(
    Ok(CirruEdnList(
      vec![CirruEdnNumber(1.0), CirruEdnNumber(2.0),]
    )),
    parse_cirru_edn(String::from("[] 1 2"))
  );
  assert_eq!(
    Ok(CirruEdnList(vec![
      CirruEdnNumber(1.0),
      CirruEdnNumber(2.0),
      CirruEdnList(vec![CirruEdnNumber(3.0)])
    ])),
    parse_cirru_edn(String::from("[] 1 2 $ [] 3"))
  );
}

#[test]
fn set_parsing() {
  let mut v: HashSet<CirruEdn> = HashSet::new();
  v.insert(CirruEdnKeyword(String::from("a")));
  v.insert(CirruEdnKeyword(String::from("b")));
  v.insert(CirruEdnKeyword(String::from("c")));
  assert_eq!(
    Ok(CirruEdnSet(v)),
    parse_cirru_edn(String::from("#{} :a :b :c"))
  );
}

#[test]
fn edn_formatting() {
  assert_eq!(write_cirru_edn(CirruEdnNil), "\ndo nil\n");
  assert_eq!(write_cirru_edn(CirruEdnBool(true)), "\ndo true\n");
  assert_eq!(write_cirru_edn(CirruEdnBool(false)), "\ndo false\n");

  assert_eq!(write_cirru_edn(CirruEdnNumber(1.0)), "\ndo 1\n");
  assert_eq!(write_cirru_edn(CirruEdnNumber(1.1)), "\ndo 1.1\n");
  assert_eq!(write_cirru_edn(CirruEdnNumber(-1.1)), "\ndo -1.1\n");

  assert_eq!(
    write_cirru_edn(CirruEdnSymbol(String::from("a"))),
    "\ndo 'a\n"
  );
  assert_eq!(
    write_cirru_edn(CirruEdnKeyword(String::from("a"))),
    "\ndo :a\n"
  );
  assert_eq!(
    write_cirru_edn(CirruEdnString(String::from("a"))),
    "\ndo |a\n"
  );
  assert_eq!(
    write_cirru_edn(CirruEdnString(String::from("a"))),
    "\ndo |a\n"
  );
}

#[test]
fn list_writing() {
  assert_eq!(
    write_cirru_edn(CirruEdnList(vec![
      CirruEdnNumber(1.0),
      CirruEdnNumber(2.0),
      CirruEdnList(vec![CirruEdnNumber(3.0)])
    ])),
    "\n[] 1 2 $ [] 3\n"
  );
}

#[test]
fn set_writing() {
  let mut v = HashSet::new();
  v.insert(CirruEdnNumber(1.0));
  v.insert(CirruEdnList(vec![CirruEdnNumber(3.0)]));

  // TODO order is not stable
  let r = write_cirru_edn(CirruEdnSet(v));
  let r1 = "\n#{} ([] 3) 1\n";
  let r2 = "\n#{} 1 $ [] 3\n";

  assert!(r == r1 || r == r2);
}

const RECORD_DEMO: &str = r#"
%{} Demo (a 1.0)
  b 2.0
  c $ [] 1.0 2.0 3.0
"#;

const DICT_DEMO: &str = r#"
{} (:a 1.0)
  :b $ [] 2.0 3.0 4.0
  :c $ {} (:d 4.0)
    :e true
    :f :g
    :h $ {} (|a 1.0)
      |b true
"#;

const DICT_DEMO2: &str = r#"
{}
  :b $ [] 2 3 4
  :a 1
  :c $ {}
    :h $ {} (|b true) (|a 1)
    :f :g
    :e true
    :d 4
"#;

#[test]
fn demo_parsing() {
  // println!("{:?}", parse_cirru_edn(String::from(RECORD_DEMO)));
  // println!("{:?}", parse_cirru_edn(String::from(DICT_DEMO)));

  assert_eq!(
    parse_cirru_edn(String::from(RECORD_DEMO)),
    Ok(CirruEdnRecord(
      String::from("Demo"),
      vec![String::from("a"), String::from("b"), String::from("c")],
      vec![
        CirruEdnNumber(1.0),
        CirruEdnNumber(2.0),
        CirruEdnList(vec![
          CirruEdnNumber(1.0),
          CirruEdnNumber(2.0),
          CirruEdnNumber(3.0)
        ])
      ]
    ))
  );

  let v1 = parse_cirru_edn(String::from(DICT_DEMO)).unwrap();
  let v2 = parse_cirru_edn(String::from(DICT_DEMO2)).unwrap();
  assert_eq!(parse_cirru_edn(write_cirru_edn(v1.clone())), Ok(v1.clone()));
  assert_eq!(v1, v2);
}
