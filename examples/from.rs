extern crate cirru_edn;

use std::convert::TryFrom;
use std::{collections::HashMap, convert::TryInto, iter::FromIterator};

use cirru_edn::{Edn, EdnKwd};

#[derive(Debug, Clone, PartialEq)]
struct Cat {
  name: String,
  category: EdnKwd,
  weight: f64,
  skills: Vec<EdnKwd>,
  counts: HashMap<String, i64>,
  owner: Option<String>,
}

impl TryFrom<Edn> for Cat {
  type Error = String;
  fn try_from(value: Edn) -> Result<Self, Self::Error> {
    let c = Cat {
      name: value.map_get_some("name")?.try_into()?,
      category: value.map_get_some("category")?.try_into()?,
      weight: value.map_get_some("weight")?.try_into()?,
      skills: value.map_get_some("skills")?.try_into()?,
      counts: value.map_get_some("counts")?.try_into()?,
      owner: {
        let v = value.map_get("owner")?;
        if v == Edn::Nil {
          None
        } else {
          Some(v.try_into()?)
        }
      },
    };
    Ok(c)
  }
}

impl From<Cat> for Edn {
  fn from(x: Cat) -> Edn {
    Edn::Map(HashMap::from_iter([
      ("name".into(), x.name.into()),
      ("category".into(), x.category.into()),
      ("weight".into(), x.weight.into()),
      ("skills".into(), x.skills.into()),
      ("counts".into(), x.counts.into()),
      ("owner".into(), x.owner.into()),
    ]))
  }
}

fn main() -> Result<(), String> {
  let data: Edn = Edn::Map(HashMap::from_iter([
    ("name".into(), Edn::str("Kii")),
    ("category".into(), Edn::kwd("ying")),
    // ("weight".into(), Edn::Number(1.0)),
    // (
    //   "skills".into(),
    //   Edn::List(vec![Edn::kwd("eating"), Edn::kwd("sleeping")]),
    // ),
    (
      "counts".into(),
      Edn::Map(HashMap::from_iter([("a".into(), Edn::Number(1.))])),
    ),
    // ("owner".into(), Edn::str("Kii")),
    ("owner".into(), Edn::Nil),
  ]));
  let cat: Cat = data.try_into()?;
  println!("new {:?}", cat);
  assert_eq!(cat.name, "Kii");
  let data2: Edn = cat.into();
  assert_eq!(data2.map_get("name")?, Edn::str("Kii"));
  Ok(())
}
