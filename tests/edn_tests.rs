use cirru_edn::Edn;
use cirru_parser::Cirru;
use std::collections::HashMap;
use std::collections::HashSet;

#[test]
fn edn_parsing() {
  assert_eq!(Ok(Edn::Nil), cirru_edn::parse("do nil"));
  assert_eq!(Ok(Edn::Bool(true)), cirru_edn::parse("do true"));
  assert_eq!(Ok(Edn::Bool(false)), cirru_edn::parse("do false"));

  assert_eq!(
    Ok(Edn::Symbol(String::from("a"))),
    cirru_edn::parse("do 'a")
  );
  assert_eq!(
    Ok(Edn::Keyword(String::from("k"))),
    cirru_edn::parse("do :k")
  );
  assert_eq!(Ok(Edn::Str(String::from("s"))), cirru_edn::parse("do |s"));

  assert_eq!(
    Ok(Edn::Str(String::from("a b\n c"))),
    cirru_edn::parse(r#"do "|a b\n c""#)
  );

  assert_eq!(Ok(Edn::Number(2.0)), cirru_edn::parse("do 2"));
  assert_eq!(Ok(Edn::Number(-2.2)), cirru_edn::parse("do -2.2"));
}

#[test]
fn list_parsing() {
  assert_eq!(
    Ok(Edn::List(vec![Edn::Number(1.0), Edn::Number(2.0),])),
    cirru_edn::parse("[] 1 2")
  );
  assert_eq!(
    Ok(Edn::List(vec![
      Edn::Number(1.0),
      Edn::Number(2.0),
      Edn::List(vec![Edn::Number(3.0)])
    ])),
    cirru_edn::parse("[] 1 2 $ [] 3")
  );
}

#[test]
fn set_parsing() {
  let mut v: HashSet<Edn> = HashSet::new();
  v.insert(Edn::Keyword(String::from("a")));
  v.insert(Edn::Keyword(String::from("b")));
  v.insert(Edn::Keyword(String::from("c")));
  assert_eq!(Ok(Edn::Set(v)), cirru_edn::parse("#{} :a :b :c"));
}

#[test]
fn edn_formatting() {
  assert_eq!(cirru_edn::format(&Edn::Nil), "\ndo nil\n");
  assert_eq!(cirru_edn::format(&Edn::Bool(true)), "\ndo true\n");
  assert_eq!(cirru_edn::format(&Edn::Bool(false)), "\ndo false\n");

  assert_eq!(cirru_edn::format(&Edn::Number(1.0)), "\ndo 1\n");
  assert_eq!(cirru_edn::format(&Edn::Number(1.1)), "\ndo 1.1\n");
  assert_eq!(cirru_edn::format(&Edn::Number(-1.1)), "\ndo -1.1\n");

  assert_eq!(
    cirru_edn::format(&Edn::Symbol(String::from("a"))),
    "\ndo 'a\n"
  );
  assert_eq!(
    cirru_edn::format(&Edn::Keyword(String::from("a"))),
    "\ndo :a\n"
  );
  assert_eq!(cirru_edn::format(&Edn::Str(String::from("a"))), "\ndo |a\n");
  assert_eq!(cirru_edn::format(&Edn::Str(String::from("a"))), "\ndo |a\n");
}

#[test]
fn list_writing() {
  assert_eq!(
    cirru_edn::format(&Edn::List(vec![
      Edn::Number(1.0),
      Edn::Number(2.0),
      Edn::List(vec![Edn::Number(3.0)])
    ])),
    "\n[] 1 2 $ [] 3\n"
  );
}

#[test]
fn set_writing() {
  let mut v = HashSet::new();
  v.insert(Edn::Number(1.0));
  v.insert(Edn::List(vec![Edn::Number(3.0)]));

  // TODO order is not stable
  let r = cirru_edn::format(&Edn::Set(v));
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
  // println!("{:?}", cirru_edn::parse(RECORD_DEMO));
  // println!("{:?}", cirru_edn::parse(DICT_DEMO));

  assert_eq!(
    cirru_edn::parse(RECORD_DEMO),
    Ok(Edn::Record(
      String::from("Demo"),
      vec![String::from("a"), String::from("b"), String::from("c")],
      vec![
        Edn::Number(1.0),
        Edn::Number(2.0),
        Edn::List(vec![Edn::Number(1.0), Edn::Number(2.0), Edn::Number(3.0)])
      ]
    ))
  );

  let v1 = cirru_edn::parse(DICT_DEMO).unwrap();
  let v2 = cirru_edn::parse(DICT_DEMO2).unwrap();
  assert_eq!(cirru_edn::parse(&cirru_edn::format(&v1)), Ok(v1.clone()));
  assert_eq!(v1, v2);
}

#[test]
fn debug_format() {
  // TODO order for hashmap is unstable

  // let DICT_INLINE2: &str =
  //   r#"({} (:a 1) (:b ([] 2 3 4)) (:c ({} (:h ({} (|b true) (|a 1)) (:e true) (:d 4) (:f :g))"#;

  // let data = cirru_edn::parse(String::from(DICT_DEMO2))?;
  // assert_eq!(format!("{}", data), DICT_INLINE2);

  let empty = HashMap::new();
  assert_eq!(format!("{}", Edn::Map(empty)), "({})");

  let mut singleton: HashMap<Edn, Edn> = HashMap::new();
  singleton.insert(Edn::Keyword(String::from("a")), Edn::Str(String::from("b")));
  assert_eq!(format!("{}", Edn::Map(singleton)), "({} (:a |b))");

  let mut singleton_set: HashSet<Edn> = HashSet::new();
  singleton_set.insert(Edn::Symbol(String::from("a")));
  assert_eq!(format!("{}", Edn::Set(singleton_set)), "(#{} 'a)");

  let singleton_vec = vec![Edn::Bool(false)];
  assert_eq!(format!("{}", Edn::List(singleton_vec)), "([] false)");

  let code = Edn::List(vec![Edn::Quote(Cirru::List(vec![
    Cirru::Leaf(String::from("a")),
    Cirru::Leaf(String::from("b")),
  ]))]);

  assert_eq!(format!("{}", code), "([] (quote (a b)))");
}
