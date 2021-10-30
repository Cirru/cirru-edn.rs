use cirru_edn::{Edn, EdnKwd};
use cirru_parser::Cirru;
use std::collections::HashMap;
use std::collections::HashSet;

#[test]
fn edn_parsing() {
  assert_eq!(Ok(Edn::Nil), cirru_edn::parse("do nil"));
  assert_eq!(Ok(Edn::Bool(true)), cirru_edn::parse("do true"));
  assert_eq!(Ok(Edn::Bool(false)), cirru_edn::parse("do false"));

  assert_eq!(Ok(Edn::sym("a")), cirru_edn::parse("do 'a"));
  assert_eq!(Ok(Edn::kwd("k")), cirru_edn::parse("do :k"));
  assert_eq!(Ok(Edn::str("s")), cirru_edn::parse("do |s"));

  assert_eq!(Ok(Edn::str("a b\n c")), cirru_edn::parse(r#"do "|a b\n c""#));

  assert_eq!(Ok(Edn::Number(2.0)), cirru_edn::parse("do 2"));
  assert_eq!(Ok(Edn::Number(-2.2)), cirru_edn::parse("do -2.2"));

  assert_eq!(
    Ok(Edn::tuple(Edn::kwd("a"), Edn::Number(1.0))),
    cirru_edn::parse(":: :a 1")
  );
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
  v.insert(Edn::kwd("a"));
  v.insert(Edn::kwd("b"));
  v.insert(Edn::kwd("c"));
  assert_eq!(Ok(Edn::Set(v)), cirru_edn::parse("#{} :a :b :c"));
}

#[test]
fn edn_formatting() -> Result<(), String> {
  assert_eq!(cirru_edn::format(&Edn::Nil, true)?, "\ndo nil\n");
  assert_eq!(cirru_edn::format(&Edn::Bool(true), true)?, "\ndo true\n");
  assert_eq!(cirru_edn::format(&Edn::Bool(false), true)?, "\ndo false\n");

  assert_eq!(cirru_edn::format(&Edn::Number(1.0), true)?, "\ndo 1\n");
  assert_eq!(cirru_edn::format(&Edn::Number(1.1), true)?, "\ndo 1.1\n");
  assert_eq!(cirru_edn::format(&Edn::Number(-1.1), true)?, "\ndo -1.1\n");

  assert_eq!(cirru_edn::format(&Edn::sym("a"), true)?, "\ndo 'a\n");
  assert_eq!(cirru_edn::format(&Edn::kwd("a"), true)?, "\ndo :a\n");
  assert_eq!(cirru_edn::format(&Edn::str("a"), true)?, "\ndo |a\n");
  assert_eq!(cirru_edn::format(&Edn::str("a b"), true)?, "\ndo \"|a b\"\n");

  assert_eq!(
    cirru_edn::format(&Edn::tuple(Edn::kwd("a"), Edn::Number(1.0)), true)?,
    "\n:: :a 1\n"
  );

  Ok(())
}

#[test]
fn list_writing() -> Result<(), String> {
  assert_eq!(
    cirru_edn::format(
      &Edn::List(vec![
        Edn::Number(1.0),
        Edn::Number(2.0),
        Edn::List(vec![Edn::Number(3.0)])
      ]),
      true
    )?,
    "\n[] 1 2 $ [] 3\n"
  );

  Ok(())
}

#[test]
fn set_writing() -> Result<(), String> {
  let mut v = HashSet::new();
  v.insert(Edn::Number(1.0));
  v.insert(Edn::List(vec![Edn::Number(3.0)]));

  // TODO order is not stable
  let r = cirru_edn::format(&Edn::Set(v), true)?;
  let r1 = "\n#{} ([] 3) 1\n";
  let r2 = "\n#{} 1 $ [] 3\n";

  assert!(r == r1 || r == r2);
  Ok(())
}

const RECORD_DEMO: &str = r#"
%{} :Demo (:a 1)
  :b 2
  :c $ [] 1 2 3
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
fn demo_parsing() -> Result<(), String> {
  // println!("{:?}", cirru_edn::parse(RECORD_DEMO));
  // println!("{:?}", cirru_edn::parse(DICT_DEMO));

  assert_eq!(
    cirru_edn::parse(RECORD_DEMO),
    Ok(Edn::Record(
      EdnKwd::from("Demo"),
      vec![
        (EdnKwd::from("a"), Edn::Number(1.0),),
        (EdnKwd::from("b"), Edn::Number(2.0)),
        (
          EdnKwd::from("c"),
          Edn::List(vec![Edn::Number(1.0), Edn::Number(2.0), Edn::Number(3.0)])
        )
      ],
    ))
  );

  let v1 = cirru_edn::parse(DICT_DEMO).unwrap();
  let v2 = cirru_edn::parse(DICT_DEMO2).unwrap();
  assert_eq!(cirru_edn::parse(&cirru_edn::format(&v1, true)?), Ok(v1.clone()));
  assert_eq!(v1, v2);

  assert_eq!(
    cirru_edn::format(
      &Edn::Record(
        EdnKwd::from("Demo"),
        vec![
          (EdnKwd::from("a"), Edn::Number(1.0),),
          (EdnKwd::from("b"), Edn::Number(2.0)),
          (
            EdnKwd::from("c"),
            Edn::List(vec![Edn::Number(1.0), Edn::Number(2.0), Edn::Number(3.0)])
          )
        ],
      ),
      false
    ),
    Ok(String::from(RECORD_DEMO))
  );

  Ok(())
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
  singleton.insert(Edn::kwd("a"), Edn::str("b"));
  assert_eq!(format!("{}", Edn::Map(singleton)), "({} (:a |b))");

  let mut singleton_set: HashSet<Edn> = HashSet::new();
  singleton_set.insert(Edn::sym("a"));
  assert_eq!(format!("{}", Edn::Set(singleton_set)), "(#{} 'a)");

  let singleton_vec = vec![Edn::Bool(false)];
  assert_eq!(format!("{}", Edn::List(singleton_vec)), "([] false)");

  let code = Edn::List(vec![Edn::Quote(Cirru::List(vec![Cirru::leaf("a"), Cirru::leaf("b")]))]);

  assert_eq!(format!("{}", code), "([] (quote (a b)))");
}

#[test]
fn test_reader() -> Result<(), String> {
  assert!(Edn::Bool(true).read_bool()?);
  assert_eq!(Edn::str("a").read_string()?, String::from("a"));
  assert_eq!(Edn::sym("a").read_symbol_string()?, String::from("a"));
  assert_eq!(Edn::kwd("a").read_keyword_string()?, String::from("a"));
  assert!((Edn::Number(1.1).read_number()? - 1.1).abs() < f64::EPSILON);
  assert_eq!(Edn::List(vec![Edn::Number(1.0)]).vec_get(0)?, Edn::Number(1.0));
  assert_eq!(Edn::List(vec![Edn::Number(1.0)]).vec_get(1)?, Edn::Nil);

  let mut dict = HashMap::new();
  dict.insert(Edn::kwd("k"), Edn::Number(1.1));
  assert!((Edn::Map(dict.to_owned()).map_get("k")?.read_number()? - 1.1).abs() < f64::EPSILON);
  assert_eq!(Edn::Map(dict).map_get("k2")?, Edn::Nil);
  Ok(())
}

#[test]
fn test_buffer() -> Result<(), String> {
  assert_eq!(Edn::Buffer(vec![]), cirru_edn::parse("buf").unwrap());
  assert_eq!(Edn::Buffer(vec![1]), cirru_edn::parse("buf 01").unwrap());
  assert_eq!(Edn::Buffer(vec![255]), cirru_edn::parse("buf ff").unwrap());
  assert_eq!(Edn::Buffer(vec![10]), cirru_edn::parse("buf 0a").unwrap());

  assert_eq!(
    cirru_edn::format(&Edn::Buffer(vec![]), true).unwrap().trim(),
    String::from("buf")
  );
  assert_eq!(
    cirru_edn::format(&Edn::Buffer(vec![1]), true).unwrap().trim(),
    String::from("buf 01")
  );
  assert_eq!(
    cirru_edn::format(&Edn::Buffer(vec![255]), true).unwrap().trim(),
    String::from("buf ff")
  );
  assert_eq!(
    cirru_edn::format(&Edn::Buffer(vec![10]), true).unwrap().trim(),
    String::from("buf 0a")
  );

  Ok(())
}
