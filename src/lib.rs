mod primes;

use cirru_parser::{parse, write_cirru, CirruNode, CirruWriterOptions};
pub use primes::CirruEdn;
use primes::CirruEdn::*;
use regex::Regex;
use std::collections::HashMap;
use std::collections::HashSet;

use cirru_parser::CirruNode::*;

/// parse Cirru code into data
pub fn parse_cirru_edn(s: String) -> Result<CirruEdn, String> {
  match parse(s) {
    Ok(nodes) => match nodes {
      CirruLeaf(_) => Err(String::from("Expected exprs")),
      CirruList(xs) => {
        if xs.len() == 1 {
          match xs[0] {
            CirruLeaf(_) => Err(String::from("Expected expr for data")),
            CirruList(_) => extract_cirru_edn(&xs[0]),
          }
        } else {
          Err(String::from("Expected 1 expr for edn"))
        }
      }
    },
    Err(e) => Err(e),
  }
}

fn extract_cirru_edn(node: &CirruNode) -> Result<CirruEdn, String> {
  match node {
    CirruLeaf(s) => match s.as_str() {
      "nil" => Ok(CirruEdnNil),
      "true" => Ok(CirruEdnBool(true)),
      "false" => Ok(CirruEdnBool(false)),
      "" => Err(String::from("Empty string is invalid")),
      s1 => match s1.chars().nth(0).unwrap() {
        '\'' => Ok(CirruEdnSymbol(String::from(&s1[1..]))),
        ':' => Ok(CirruEdnKeyword(String::from(&s1[1..]))),
        '"' | '|' => Ok(CirruEdnString(String::from(&s1[1..]))),
        _ => {
          if matches_float(s1) {
            let f: f32 = s1.parse().unwrap();
            Ok(CirruEdnNumber(f))
          } else {
            Err(format!("Unknown token: {:?}", s1))
          }
        }
      },
    },
    CirruList(xs) => {
      if xs.len() == 0 {
        Err(String::from("empty expr is invalid"))
      } else {
        match &xs[0] {
          CirruLeaf(s) => match s.as_str() {
            "quote" => {
              if xs.len() == 2 {
                Ok(CirruEdnQuote(xs[1].clone()))
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
              let mut ys: Vec<CirruEdn> = vec![];
              for (idx, x) in xs.iter().enumerate() {
                if idx > 0 {
                  match extract_cirru_edn(x) {
                    Ok(v) => ys.push(v),
                    Err(v) => return Err(v),
                  }
                }
              }
              Ok(CirruEdnList(ys))
            }
            "#{}" => {
              let mut ys: HashSet<CirruEdn> = HashSet::new();
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
              Ok(CirruEdnSet(ys))
            }
            "{}" => {
              let mut zs: HashMap<CirruEdn, CirruEdn> = HashMap::new();
              for (idx, x) in xs.iter().enumerate() {
                if idx > 0 {
                  match x {
                    CirruLeaf(s) => return Err(format!("Invalid map entry: {}", s)),
                    CirruList(ys) => {
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
              Ok(CirruEdnMap(zs))
            }
            "%{}" => {
              if xs.len() >= 3 {
                let name = match xs[1].clone() {
                  CirruLeaf(s) => s,
                  CirruList(e) => return Err(format!("expected record name in string: {:?}", e)),
                };
                let mut fields: Vec<String> = vec![];
                let mut values: Vec<CirruEdn> = vec![];
                for (idx, x) in xs.iter().enumerate() {
                  if idx > 1 {
                    match x {
                      CirruLeaf(s) => return Err(format!("Invalid record entry: {}", s)),
                      CirruList(ys) => {
                        if ys.len() == 2 {
                          match (&ys[0], extract_cirru_edn(&ys[1])) {
                            (CirruLeaf(s), Ok(v)) => {
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
                Ok(CirruEdnRecord(name, fields, values))
              } else {
                Err(format!("Not enough items for record"))
              }
            }
            a => Err(format!("Invalid operator: {}", a)),
          },
          CirruList(a) => Err(format!("Invalid operator: {:?}", a)),
        }
      }
    }
  }
}

fn matches_float(x: &str) -> bool {
  let re = Regex::new("^-?[\\d]+(\\.[\\d]+)?$").unwrap(); // TODO special cases not handled
  re.is_match(x)
}

fn assemble_cirru_node(data: &CirruEdn) -> CirruNode {
  match data {
    CirruEdnNil => CirruLeaf(String::from("nil")),
    CirruEdnBool(v) => CirruLeaf(format!("{}", v)),
    CirruEdnNumber(n) => CirruLeaf(format!("{}", n)),
    CirruEdnSymbol(s) => CirruLeaf(format!("'{}", s)),
    CirruEdnKeyword(s) => CirruLeaf(format!(":{}", s)),
    CirruEdnString(s) => CirruLeaf(format!("|{}", s)),
    CirruEdnQuote(v) => CirruList(vec![CirruLeaf(String::from("quote")), (*v).clone()]),
    CirruEdnList(xs) => {
      let mut ys: Vec<CirruNode> = vec![CirruLeaf(String::from("[]"))];
      for x in xs {
        ys.push(assemble_cirru_node(x));
      }
      return CirruList(ys);
    }
    CirruEdnSet(xs) => {
      let mut ys: Vec<CirruNode> = vec![CirruLeaf(String::from("#{}"))];
      for x in xs {
        ys.push(assemble_cirru_node(x));
      }
      return CirruList(ys);
    }
    CirruEdnMap(xs) => {
      let mut ys: Vec<CirruNode> = vec![CirruLeaf(String::from("{}"))];
      for (k, v) in xs {
        ys.push(CirruList(vec![
          assemble_cirru_node(k),
          assemble_cirru_node(v),
        ]))
      }
      CirruList(ys)
    }
    CirruEdnRecord(name, fields, values) => {
      let mut ys: Vec<CirruNode> = vec![
        CirruLeaf(String::from("%{}")),
        CirruLeaf(String::from(name)),
      ];
      for idx in 0..fields.len() {
        let v = &values[idx];
        ys.push(CirruList(vec![
          CirruLeaf(fields[idx].clone()),
          assemble_cirru_node(v),
        ]));
      }

      CirruList(ys)
    }
  }
}

/// generate string fro, CirruEdn
pub fn write_cirru_edn(data: CirruEdn) -> String {
  let options = CirruWriterOptions { use_inline: true };
  match assemble_cirru_node(&data) {
    CirruLeaf(s) => write_cirru(
      CirruList(vec![
        (CirruList(vec![CirruLeaf(String::from("do")), CirruLeaf(s)])),
      ]),
      options,
    ),
    CirruList(xs) => write_cirru(CirruList(vec![(CirruList(xs))]), options),
  }
}
