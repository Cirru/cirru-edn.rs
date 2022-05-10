extern crate cirru_edn;

use cirru_edn::Edn;
use serde::Deserialize;

use std::{collections::HashSet, io::Error};

fn main() -> Result<(), Error> {
  let data = Edn::List(vec![Edn::kwd("aa"), Edn::Set(HashSet::from([Edn::str("a")]))]);

  let content = serde_json::to_value(data)?;

  println!("content: {}", content);

  let j = serde_json::json!({"a": 1.0});
  println!("edn: {}", Edn::deserialize(j)?);

  Ok(())
}
