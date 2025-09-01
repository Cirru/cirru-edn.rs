extern crate cirru_edn;

use std::convert::TryFrom;
use std::{collections::HashMap, convert::TryInto, iter::FromIterator};

use cirru_edn::{Edn, EdnMapView, EdnTag};

struct Cat {
  name: String,
  category: EdnTag,
  weight: f64,
  skills: Vec<EdnTag>,
  counts: HashMap<String, i64>,
  injection_times: u8,
  owner: Option<String>,
}

impl TryFrom<Edn> for Cat {
  type Error = String;
  fn try_from(value: Edn) -> Result<Self, Self::Error> {
    let c = Cat {
      name: value.view_map()?.get_or_nil("name").try_into()?,
      category: value.view_map()?.get_or_nil("category").try_into()?,
      weight: value.view_map()?.get_or_nil("weight").try_into()?,
      skills: value.view_map()?.get_or_nil("skills").try_into()?,
      counts: value.view_map()?.get_or_nil("counts").try_into()?,
      injection_times: value.view_map()?.get_or_nil("injection_times").try_into()?,
      owner: {
        let v = value.view_map()?.get_or_nil("owner");
        if v == Edn::Nil { None } else { Some(v.try_into()?) }
      },
    };
    Ok(c)
  }
}

impl From<Cat> for Edn {
  fn from(x: Cat) -> Edn {
    Edn::Map(EdnMapView(HashMap::from_iter([
      ("name".into(), x.name.into()),
      ("category".into(), x.category.into()),
      ("weight".into(), x.weight.into()),
      ("skills".into(), x.skills.into()),
      ("counts".into(), x.counts.into()),
      ("injection_times".into(), x.injection_times.into()),
      ("owner".into(), x.owner.into()),
    ])))
  }
}

#[test]
fn from_to_test() -> Result<(), String> {
  let data: Edn = Edn::Map(EdnMapView(HashMap::from_iter([
    ("name".into(), Edn::str("Kii")),
    ("category".into(), Edn::tag("ying")),
    ("weight".into(), Edn::Number(1.0)),
    (
      "skills".into(),
      Edn::from(vec![Edn::tag("eating"), Edn::tag("sleeping")]),
    ),
    (
      "counts".into(),
      Edn::Map(EdnMapView(HashMap::from_iter([("a".into(), Edn::Number(1.))]))),
    ),
    ("injection_times".into(), Edn::Number(10.0)),
    // ("owner".into(), Edn::str("Kii")),
    ("owner".into(), Edn::Nil),
  ])));
  let cat: Cat = data.try_into()?;
  assert_eq!(cat.name, "Kii");
  let data2: Edn = cat.into();
  assert_eq!(data2.view_map()?.get_or_nil("name"), Edn::str("Kii"));
  Ok(())
}
