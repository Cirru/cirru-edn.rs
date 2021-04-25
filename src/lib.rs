#[macro_use]
extern crate lazy_static;

mod primes;

use cirru_parser::{Cirru, CirruWriterOptions};
pub use primes::Edn;
use regex::Regex;
use std::collections::HashMap;
use std::collections::HashSet;

/// parse Cirru code into data
pub fn parse(s: &str) -> Result<Edn, String> {
  match cirru_parser::parse(&s) {
    Ok(nodes) => match nodes {
      Cirru::Leaf(_) => Err(String::from("Expected exprs")),
      Cirru::List(xs) => {
        if xs.len() == 1 {
          match xs[0] {
            Cirru::Leaf(_) => Err(String::from("Expected expr for data")),
            Cirru::List(_) => extract_cirru_edn(&xs[0]),
          }
        } else {
          Err(String::from("Expected 1 expr for edn"))
        }
      }
    },
    Err(e) => Err(e),
  }
}

fn extract_cirru_edn(node: &Cirru) -> Result<Edn, String> {
  match node {
    Cirru::Leaf(s) => match s.as_str() {
      "nil" => Ok(Edn::Nil),
      "true" => Ok(Edn::Bool(true)),
      "false" => Ok(Edn::Bool(false)),
      "" => Err(String::from("Empty string is invalid")),
      s1 => match s1.chars().next().unwrap() {
        '\'' => Ok(Edn::Symbol(String::from(&s1[1..]))),
        ':' => Ok(Edn::Keyword(String::from(&s1[1..]))),
        '"' | '|' => Ok(Edn::Str(String::from(&s1[1..]))),
        _ => {
          if matches_float(s1) {
            let f: f32 = s1.parse().unwrap();
            Ok(Edn::Number(f))
          } else {
            Err(format!("Unknown token: {:?}", s1))
          }
        }
      },
    },
    Cirru::List(xs) => {
      if xs.is_empty() {
        Err(String::from("empty expr is invalid"))
      } else {
        match &xs[0] {
          Cirru::Leaf(s) => match s.as_str() {
            "quote" => {
              if xs.len() == 2 {
                Ok(Edn::Quote(xs[1].clone()))
              } else {
                Err(String::from("Missing quote value"))
              }
            }
            "do" => {
              if xs.len() == 2 {
                extract_cirru_edn(&xs[1])
              } else {
                Err(String::from("Missing do value"))
              }
            }
            "[]" => {
              let mut ys: Vec<Edn> = vec![];
              for (idx, x) in xs.iter().enumerate() {
                if idx > 0 {
                  match extract_cirru_edn(x) {
                    Ok(v) => ys.push(v),
                    Err(v) => return Err(v),
                  }
                }
              }
              Ok(Edn::List(ys))
            }
            "#{}" => {
              let mut ys: HashSet<Edn> = HashSet::new();
              for (idx, x) in xs.iter().enumerate() {
                if idx > 0 {
                  match extract_cirru_edn(x) {
                    Ok(v) => {
                      ys.insert(v);
                    }
                    Err(v) => return Err(v),
                  }
                }
              }
              Ok(Edn::Set(ys))
            }
            "{}" => {
              let mut zs: HashMap<Edn, Edn> = HashMap::new();
              for (idx, x) in xs.iter().enumerate() {
                if idx > 0 {
                  match x {
                    Cirru::Leaf(s) => return Err(format!("Invalid map entry: {}", s)),
                    Cirru::List(ys) => {
                      if ys.len() == 2 {
                        match (extract_cirru_edn(&ys[0]), extract_cirru_edn(&ys[1])) {
                          (Ok(k), Ok(v)) => {
                            zs.insert(k, v);
                          }
                          (e1, e2) => return Err(format!("invalid map entry: {:?} {:?}", e1, e2)),
                        }
                      }
                    }
                  }
                }
              }
              Ok(Edn::Map(zs))
            }
            "%{}" => {
              if xs.len() >= 3 {
                let name = match xs[1].clone() {
                  Cirru::Leaf(s) => s,
                  Cirru::List(e) => return Err(format!("expected record name in string: {:?}", e)),
                };
                let mut fields: Vec<String> = vec![];
                let mut values: Vec<Edn> = vec![];
                for (idx, x) in xs.iter().enumerate() {
                  if idx > 1 {
                    match x {
                      Cirru::Leaf(s) => return Err(format!("Invalid record entry: {}", s)),
                      Cirru::List(ys) => {
                        if ys.len() == 2 {
                          match (&ys[0], extract_cirru_edn(&ys[1])) {
                            (Cirru::Leaf(s), Ok(v)) => {
                              fields.push(s.clone());
                              values.push(v);
                            }
                            (e1, e2) => {
                              return Err(format!("invalid map entry: {:?} {:?}", e1, e2))
                            }
                          }
                        }
                      }
                    }
                  }
                }
                Ok(Edn::Record(name, fields, values))
              } else {
                Err(String::from("Not enough items for record"))
              }
            }
            a => Err(format!("Invalid operator: {}", a)),
          },
          Cirru::List(a) => Err(format!("Invalid operator: {:?}", a)),
        }
      }
    }
  }
}

lazy_static! {
    static ref RE_FLOAT: Regex = Regex::new("^-?[\\d]+(\\.[\\d]+)?$").unwrap(); // TODO special cases not handled
}

fn matches_float(x: &str) -> bool {
  RE_FLOAT.is_match(x)
}

fn assemble_cirru_node(data: &Edn) -> Cirru {
  match data {
    Edn::Nil => Cirru::Leaf(String::from("nil")),
    Edn::Bool(v) => {
      let mut leaf = String::from("");
      leaf.push_str(&v.to_string());
      Cirru::Leaf(leaf)
    }
    Edn::Number(n) => {
      let mut leaf = String::from("");
      leaf.push_str(&n.to_string());
      Cirru::Leaf(leaf)
    }
    Edn::Symbol(s) => {
      let mut leaf = String::from("'");
      leaf.push_str(&s);
      Cirru::Leaf(leaf)
    }
    Edn::Keyword(s) => {
      let mut leaf = String::from(":");
      leaf.push_str(&s);
      Cirru::Leaf(leaf)
    }
    Edn::Str(s) => {
      let mut leaf = String::from("|");
      leaf.push_str(&s);
      Cirru::Leaf(leaf)
    }
    Edn::Quote(v) => Cirru::List(vec![Cirru::Leaf(String::from("quote")), (*v).clone()]),
    Edn::List(xs) => {
      let mut ys: Vec<Cirru> = vec![Cirru::Leaf(String::from("[]"))];
      for x in xs {
        ys.push(assemble_cirru_node(x));
      }
      Cirru::List(ys)
    }
    Edn::Set(xs) => {
      let mut ys: Vec<Cirru> = vec![Cirru::Leaf(String::from("#{}"))];
      for x in xs {
        ys.push(assemble_cirru_node(x));
      }
      Cirru::List(ys)
    }
    Edn::Map(xs) => {
      let mut ys: Vec<Cirru> = vec![Cirru::Leaf(String::from("{}"))];
      for (k, v) in xs {
        ys.push(Cirru::List(vec![
          assemble_cirru_node(k),
          assemble_cirru_node(v),
        ]))
      }
      Cirru::List(ys)
    }
    Edn::Record(name, fields, values) => {
      let mut ys: Vec<Cirru> = vec![
        Cirru::Leaf(String::from("%{}")),
        Cirru::Leaf(String::from(name)),
      ];
      for idx in 0..fields.len() {
        let v = &values[idx];
        ys.push(Cirru::List(vec![
          Cirru::Leaf(fields[idx].clone()),
          assemble_cirru_node(v),
        ]));
      }

      Cirru::List(ys)
    }
  }
}

/// generate string fro, Edn
pub fn format(data: &Edn, use_inline: bool) -> String {
  let options = CirruWriterOptions { use_inline };
  match assemble_cirru_node(&data) {
    Cirru::Leaf(s) => cirru_parser::format(
      &Cirru::List(vec![
        (Cirru::List(vec![Cirru::Leaf(String::from("do")), Cirru::Leaf(s)])),
      ]),
      options,
    ),
    Cirru::List(xs) => cirru_parser::format(&Cirru::List(vec![(Cirru::List(xs))]), options),
  }
}
