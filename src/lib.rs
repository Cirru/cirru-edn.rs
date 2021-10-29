mod primes;

use cirru_parser::{Cirru, CirruWriterOptions};
pub use primes::Edn;
use std::collections::HashMap;
use std::collections::HashSet;

/// parse Cirru code into data
pub fn parse(s: &str) -> Result<Edn, String> {
  match cirru_parser::parse(s) {
    Ok(xs) => {
      if xs.len() == 1 {
        match &xs[0] {
          Cirru::Leaf(s) => Err(format!("expected expr for data, got leaf: {}", s)),
          Cirru::List(_) => extract_cirru_edn(&xs[0]),
        }
      } else {
        Err(format!("Expected 1 expr for edn, got length {}: {:?} ", xs.len(), xs))
      }
    }
    Err(e) => Err(e),
  }
}

fn extract_cirru_edn(node: &Cirru) -> Result<Edn, String> {
  match node {
    Cirru::Leaf(s) => match s.as_str() {
      "nil" => Ok(Edn::Nil),
      "true" => Ok(Edn::Bool(true)),
      "false" => Ok(Edn::Bool(false)),
      "" => Err(String::from("empty string is invalid for edn")),
      s1 => match s1.chars().next().unwrap() {
        '\'' => Ok(Edn::Symbol(s1[1..].to_owned())),
        ':' => Ok(Edn::Keyword(s1[1..].to_owned())),
        '"' | '|' => Ok(Edn::Str(s1[1..].to_owned())),
        _ => {
          if let Ok(f) = s1.trim().parse::<f64>() {
            Ok(Edn::Number(f))
          } else {
            Err(format!("unknown token for edn value: {:?}", s1))
          }
        }
      },
    },
    Cirru::List(xs) => {
      if xs.is_empty() {
        Err(String::from("empty expr is invalid for edn"))
      } else {
        match &xs[0] {
          Cirru::Leaf(s) => match s.as_str() {
            "quote" => {
              if xs.len() == 2 {
                Ok(Edn::Quote(xs[1].to_owned()))
              } else {
                Err(String::from("missing edn quote value"))
              }
            }
            "do" => {
              if xs.len() == 2 {
                extract_cirru_edn(&xs[1])
              } else {
                Err(String::from("missing edn do value"))
              }
            }
            "::" => {
              if xs.len() == 3 {
                Ok(Edn::Tuple(
                  Box::new(extract_cirru_edn(&xs[1])?),
                  Box::new(extract_cirru_edn(&xs[2])?),
                ))
              } else {
                Err(String::from("tuple expected 2 values"))
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
                    Cirru::Leaf(s) => return Err(format!("expected a pair, invalid map entry: {}", s)),
                    Cirru::List(ys) => {
                      if ys.len() == 2 {
                        match (extract_cirru_edn(&ys[0]), extract_cirru_edn(&ys[1])) {
                          (Ok(k), Ok(v)) => {
                            zs.insert(k, v);
                          }
                          (Err(e), _) => return Err(format!("invalid map entry `{}` from `{}`", e, &ys[0])),
                          (Ok(k), Err(e)) => return Err(format!("invalid map entry for `{}`, got {}", k, e)),
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
                let name = match xs[1].to_owned() {
                  Cirru::Leaf(s) => s.strip_prefix(':').unwrap_or(&s).to_owned(),
                  Cirru::List(e) => return Err(format!("expected record name in string: {:?}", e)),
                };
                let mut entries: Vec<(String, Edn)> = vec![];

                for (idx, x) in xs.iter().enumerate() {
                  if idx > 1 {
                    match x {
                      Cirru::Leaf(s) => return Err(format!("expected record, invalid record entry: {}", s)),
                      Cirru::List(ys) => {
                        if ys.len() == 2 {
                          match (&ys[0], extract_cirru_edn(&ys[1])) {
                            (Cirru::Leaf(s), Ok(v)) => {
                              entries.push((s.strip_prefix(':').unwrap_or(s).to_owned(), v));
                            }
                            (Cirru::Leaf(s), Err(e)) => {
                              return Err(format!("invalid record value for `{}`, got: {}", s, e))
                            }
                            (Cirru::List(zs), _) => return Err(format!("invalid list as record key: {:?}", zs)),
                          }
                        }
                      }
                    }
                  }
                }
                Ok(Edn::Record(name, entries))
              } else {
                Err(String::from("insufficient items for edn record"))
              }
            }
            "buf" => {
              let mut ys: Vec<u8> = vec![];
              for (idx, x) in xs.iter().enumerate() {
                if idx > 0 {
                  match x {
                    Cirru::Leaf(y) => {
                      if y.len() == 2 {
                        match hex::decode(y) {
                          Ok(b) => {
                            if b.len() == 1 {
                              ys.push(b[0])
                            } else {
                              return Err(format!("hex for buffer might be too large, got: {:?}", b));
                            }
                          }
                          Err(e) => return Err(format!("expected length 2 hex string in buffer, got: {} {}", y, e)),
                        }
                      } else {
                        return Err(format!("expected length 2 hex string in buffer, got: {}", y));
                      }
                    }
                    _ => return Err(format!("expected hex string in buffer, got: {}", x)),
                  }
                }
              }
              Ok(Edn::Buffer(ys))
            }
            a => Err(format!("invalid operator for edn: {}", a)),
          },
          Cirru::List(a) => Err(format!("invalid nodes for edn: {:?}", a)),
        }
      }
    }
  }
}

fn assemble_cirru_node(data: &Edn) -> Cirru {
  match data {
    Edn::Nil => Cirru::leaf("nil"),
    Edn::Bool(v) => Cirru::Leaf(v.to_string()),
    Edn::Number(n) => Cirru::Leaf(n.to_string()),
    Edn::Symbol(s) => Cirru::Leaf(format!("'{}", s)),
    Edn::Keyword(s) => Cirru::Leaf(format!(":{}", s)),
    Edn::Str(s) => Cirru::Leaf(format!("|{}", s)),
    Edn::Quote(v) => Cirru::List(vec![Cirru::leaf("quote"), (*v).to_owned()]),
    Edn::List(xs) => {
      let mut ys: Vec<Cirru> = vec![Cirru::leaf("[]")];
      for x in xs {
        ys.push(assemble_cirru_node(x));
      }
      Cirru::List(ys)
    }
    Edn::Set(xs) => {
      let mut ys: Vec<Cirru> = vec![Cirru::leaf("#{}")];
      for x in xs {
        ys.push(assemble_cirru_node(x));
      }
      Cirru::List(ys)
    }
    Edn::Map(xs) => {
      let mut ys: Vec<Cirru> = vec![Cirru::leaf("{}")];
      for (k, v) in xs {
        ys.push(Cirru::List(vec![assemble_cirru_node(k), assemble_cirru_node(v)]))
      }
      Cirru::List(ys)
    }
    Edn::Record(name, entries) => {
      let mut ys: Vec<Cirru> = vec![Cirru::leaf("%{}"), Cirru::Leaf(format!(":{}", name))];
      for entry in entries {
        let v = &entry.1;
        ys.push(Cirru::List(vec![
          Cirru::Leaf(format!(":{}", entry.0)),
          assemble_cirru_node(v),
        ]));
      }

      Cirru::List(ys)
    }
    Edn::Tuple(tag, v) => {
      let mut ys: Vec<Cirru> = vec![Cirru::leaf("::")];
      ys.push(assemble_cirru_node(&*tag.to_owned()));
      ys.push(assemble_cirru_node(&*v.to_owned()));
      Cirru::List(ys)
    }
    Edn::Buffer(buf) => {
      let mut ys: Vec<Cirru> = vec![Cirru::leaf("buf")];
      for b in buf {
        ys.push(Cirru::Leaf(hex::encode(vec![b.to_owned()])));
      }
      Cirru::List(ys)
    }
  }
}

/// generate string fro, Edn
pub fn format(data: &Edn, use_inline: bool) -> Result<String, String> {
  let options = CirruWriterOptions { use_inline };
  match assemble_cirru_node(data) {
    Cirru::Leaf(s) => cirru_parser::format(&[(Cirru::List(vec![Cirru::leaf("do"), Cirru::leaf(s)]))], options),
    Cirru::List(xs) => cirru_parser::format(&[(Cirru::List(xs))], options),
  }
}
