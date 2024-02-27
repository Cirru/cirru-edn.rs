extern crate cirru_edn;

use cirru_edn::EdnRecordView;
use cirru_edn::{Edn, EdnListView, EdnTag};
use std::collections::HashMap;
use std::collections::HashSet;

#[test]
fn edn_parsing() {
  assert_eq!(Ok(Edn::Nil), cirru_edn::parse("do nil"));
  assert_eq!(Ok(Edn::Bool(true)), cirru_edn::parse("do true"));
  assert_eq!(Ok(Edn::Bool(false)), cirru_edn::parse("do false"));

  assert_eq!(Ok(Edn::sym("a")), cirru_edn::parse("do 'a"));
  assert_eq!(Ok(Edn::tag("k")), cirru_edn::parse("do :k"));
  assert_eq!(Ok(Edn::str("s")), cirru_edn::parse("do |s"));

  assert_eq!(Ok(Edn::str("a b\n c")), cirru_edn::parse(r#"do "|a b\n c""#));

  assert_eq!(Ok(Edn::Number(2.0)), cirru_edn::parse("do 2"));
  assert_eq!(Ok(Edn::Number(-2.2)), cirru_edn::parse("do -2.2"));

  assert_eq!(Ok(Edn::tuple(Edn::tag("a"), vec![])), cirru_edn::parse(":: :a"));

  assert_eq!(
    Ok(Edn::tuple(Edn::tag("a"), vec![Edn::Number(1.0)])),
    cirru_edn::parse(":: :a 1")
  );

  assert_eq!(
    Ok(Edn::tuple(Edn::tag("a"), vec![Edn::Number(1.0), Edn::Number(2.0)])),
    cirru_edn::parse(":: :a 1 2")
  );

  assert_eq!(
    Ok(Edn::tuple(
      Edn::tag("a"),
      vec![Edn::Number(1.0), Edn::Number(2.0), Edn::str("b")]
    )),
    cirru_edn::parse(":: :a 1 2 |b")
  );

  assert_eq!(Ok(Edn::str("中文")), cirru_edn::parse("do |中文"));

  assert_eq!(
    Ok(Edn::List(EdnListView(vec![Edn::Number(1.), Edn::Number(2.)]))),
    cirru_edn::parse("[] (; one) 1 (; two) 2 (; end)")
  );

  assert_eq!(Ok(Edn::Number(1.)), cirru_edn::parse("do (; number) 1 (; end)"));
}

#[test]
fn list_parsing() {
  assert_eq!(
    Ok(Edn::from(vec![Edn::Number(1.0), Edn::Number(2.0),])),
    cirru_edn::parse("[] 1 2")
  );
  assert_eq!(
    Ok(Edn::from(vec![
      Edn::Number(1.0),
      Edn::Number(2.0),
      Edn::from(vec![Edn::Number(3.0)])
    ])),
    cirru_edn::parse("[] 1 2 $ [] 3")
  );
}

#[test]
fn set_parsing() {
  let mut v: HashSet<Edn> = HashSet::new();
  v.insert(Edn::tag("a"));
  v.insert(Edn::tag("b"));
  v.insert(Edn::tag("c"));
  assert_eq!(Ok(Edn::from(v)), cirru_edn::parse("#{} :a :b :c"));
}

const ORDER_DEMO: &str = r#"
{} (:a 1) (:c 2)
  :b $ [] 1 2
"#;

#[test]
fn edn_formatting() -> Result<(), String> {
  assert_eq!(cirru_edn::format(&Edn::Nil, true)?, "\ndo nil\n");
  assert_eq!(cirru_edn::format(&Edn::Bool(true), true)?, "\ndo true\n");
  assert_eq!(cirru_edn::format(&Edn::Bool(false), true)?, "\ndo false\n");

  assert_eq!(cirru_edn::format(&Edn::Number(1.0), true)?, "\ndo 1\n");
  assert_eq!(cirru_edn::format(&Edn::Number(1.1), true)?, "\ndo 1.1\n");
  assert_eq!(cirru_edn::format(&Edn::Number(-1.1), true)?, "\ndo -1.1\n");

  assert_eq!(cirru_edn::format(&Edn::sym("a"), true)?, "\ndo 'a\n");
  assert_eq!(cirru_edn::format(&Edn::tag("a"), true)?, "\ndo :a\n");
  assert_eq!(cirru_edn::format(&Edn::str("a"), true)?, "\ndo |a\n");
  assert_eq!(cirru_edn::format(&Edn::str("a b"), true)?, "\ndo \"|a b\"\n");

  assert_eq!(
    cirru_edn::format(
      &Edn::from(HashMap::from([
        (Edn::tag("b"), Edn::from(vec![Edn::Number(1.0), Edn::Number(2.0)])),
        (Edn::tag("c"), Edn::Number(2.0)),
        (Edn::tag("a"), Edn::Number(1.0)),
      ])),
      true
    )?,
    ORDER_DEMO
  );

  assert_eq!(
    cirru_edn::format(&Edn::tuple(Edn::tag("a"), vec![Edn::Number(1.0)]), true)?,
    "\n:: :a 1\n"
  );

  assert_eq!(
    cirru_edn::format(&Edn::tuple(Edn::tag("a"), vec![]), true)?,
    "\n:: :a\n"
  );

  assert_eq!(
    cirru_edn::format(
      &Edn::tuple(Edn::tag("a"), vec![Edn::Number(1.0), Edn::tag("c"), Edn::Nil]),
      true
    )?,
    "\n:: :a 1 :c nil\n"
  );

  Ok(())
}

#[test]
fn list_writing() -> Result<(), String> {
  assert_eq!(
    cirru_edn::format(
      &Edn::from(vec![
        Edn::Number(1.0),
        Edn::Number(2.0),
        Edn::from(vec![Edn::Number(3.0)])
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
  v.insert(Edn::from(vec![Edn::Number(3.0)]));

  // TODO order is not stable
  let r = cirru_edn::format(&Edn::from(v), true)?;
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

const DICT_DEMO_COMMENT: &str = r#"
{}
  :b $ [] 2 3 4 (; "comment")
  :a 1
  ; "comment"
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
    Ok(Edn::Record(EdnRecordView {
      tag: EdnTag::new("Demo"),
      pairs: vec![
        (EdnTag::new("a"), Edn::Number(1.0),),
        (EdnTag::new("b"), Edn::Number(2.0)),
        (
          EdnTag::new("c"),
          Edn::from(vec![Edn::Number(1.0), Edn::Number(2.0), Edn::Number(3.0)])
        )
      ],
    }))
  );

  let v1 = cirru_edn::parse(DICT_DEMO).unwrap();
  let v2 = cirru_edn::parse(DICT_DEMO2).unwrap();
  let v_comment = cirru_edn::parse(DICT_DEMO_COMMENT).unwrap();
  assert_eq!(cirru_edn::parse(&cirru_edn::format(&v1, true)?), Ok(v1.to_owned()));
  assert_eq!(v1, v2);
  assert_eq!(v2, v_comment);

  assert_eq!(
    cirru_edn::format(
      &Edn::Record(EdnRecordView {
        tag: EdnTag::new("Demo"),
        pairs: vec![
          (EdnTag::new("a"), Edn::Number(1.0),),
          (EdnTag::new("b"), Edn::Number(2.0)),
          (
            EdnTag::new("c"),
            Edn::from(vec![Edn::Number(1.0), Edn::Number(2.0), Edn::Number(3.0)])
          )
        ],
      }),
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

  let empty: HashMap<Edn, Edn> = HashMap::new();
  assert_eq!(format!("{}", Edn::from(empty)), "({})");

  let mut singleton: HashMap<Edn, Edn> = HashMap::new();
  singleton.insert(Edn::tag("a"), Edn::str("b"));
  assert_eq!(format!("{}", Edn::from(singleton)), "({} (:a |b))");

  let mut singleton_set: HashSet<Edn> = HashSet::new();
  singleton_set.insert(Edn::sym("a"));
  assert_eq!(format!("{}", Edn::from(singleton_set)), "(#{} 'a)");

  let singleton_vec = vec![Edn::Bool(false)];
  assert_eq!(format!("{}", Edn::from(singleton_vec)), "([] false)");

  let code = Edn::from(vec![Edn::Quote(vec!["a", "b"].into())]);

  assert_eq!(format!("{}", code), "([] (quote (a b)))");
}

#[test]
fn test_reader() -> Result<(), String> {
  assert!(Edn::Bool(true).read_bool()?);
  assert_eq!(Edn::str("a").read_string()?, String::from("a"));
  assert_eq!(Edn::sym("a").read_symbol_string()?, String::from("a"));
  assert_eq!(Edn::tag("a").read_tag_str()?, "a".into());
  assert!((Edn::Number(1.1).read_number()? - 1.1).abs() < f64::EPSILON);
  assert_eq!(
    Edn::from(vec![Edn::Number(1.0)]).view_list()?.get_or_nil(0),
    Edn::Number(1.0)
  );
  assert_eq!(Edn::from(vec![Edn::Number(1.0)]).view_list()?.get_or_nil(1), Edn::Nil);

  let mut dict = HashMap::new();
  dict.insert(Edn::tag("k"), Edn::Number(1.1));
  assert!((Edn::from(dict.to_owned()).view_map()?.get_or_nil("k").read_number()? - 1.1).abs() < f64::EPSILON);
  assert_eq!(Edn::from(dict).view_map()?.get_or_nil("k2"), Edn::Nil);
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

#[test]
fn test_string_order() -> Result<(), String> {
  let mut data: HashMap<Edn, Edn> = HashMap::new();
  data.insert(Edn::tag("a"), Edn::Number(1.0));
  data.insert(Edn::tag("c"), Edn::Number(2.0));
  data.insert(Edn::tag("b"), Edn::Number(3.0));
  data.insert(Edn::tag("Z"), Edn::Number(4.0));
  assert_eq!(
    cirru_edn::format(&Edn::from(data), true).unwrap().trim(),
    "{} (:Z 4) (:a 1) (:b 3) (:c 2)".to_owned()
  );

  let mut data2: HashMap<Edn, Edn> = HashMap::new();
  data2.insert(Edn::str("a"), Edn::Number(1.0));
  data2.insert(Edn::str("c"), Edn::Number(2.0));
  data2.insert(Edn::str("b"), Edn::Number(3.0));
  data2.insert(Edn::str("Z"), Edn::Number(4.0));
  assert_eq!(
    cirru_edn::format(&Edn::from(data2), true).unwrap().trim(),
    "{} (|Z 4) (|a 1) (|b 3) (|c 2)".to_owned()
  );

  let mut v: HashSet<Edn> = HashSet::new();
  v.insert(Edn::tag("a"));
  v.insert(Edn::tag("1"));
  v.insert(Edn::tag("z"));
  assert_eq!(Ok(Edn::from(v)), cirru_edn::parse("#{} :z :a :1"));
  Ok(())
}

#[test]
fn test_format_record() -> Result<(), String> {
  let record = Edn::Record(EdnRecordView {
    tag: EdnTag::new("Demo"),
    pairs: vec![
      (EdnTag::new("a"), Edn::Number(1.0)),
      (
        EdnTag::new("c"),
        Edn::from(vec![Edn::Number(1.0), Edn::Number(2.0), Edn::Number(3.0)]),
      ),
      (EdnTag::new("b"), Edn::Number(2.0)),
      (EdnTag::new("d"), Edn::Number(3.0)),
    ],
  });

  assert_eq!(
    cirru_edn::format(&record, true)?,
    "\n%{} :Demo (:a 1) (:b 2) (:d 3)\n  :c $ [] 1 2 3\n"
  );

  Ok(())
}

#[test]
fn test_iter() -> Result<(), String> {
  let xs = vec![
    Edn::from(1u8),
    2u8.into(),
    3u8.into(),
    4u8.into(),
    5u8.into(),
    6u8.into(),
    7u8.into(),
    8u8.into(),
    9u8.into(),
  ];
  let data = Edn::from(xs);
  for item in &data.view_list()? {
    println!("{:?}", item);
  }
  Ok(())
}
