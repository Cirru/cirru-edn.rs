mod edn;
mod tag;

use std::cmp::Ordering::*;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::sync::Arc;
use std::vec;

use cirru_parser::{Cirru, CirruWriterOptions};

pub use edn::{Edn, EdnListView, EdnMapView, EdnRecordView, EdnSetView, EdnTupleView};
pub use tag::EdnTag;

/// parse Cirru code into data
pub fn parse(s: &str) -> Result<Edn, String> {
  let xs = cirru_parser::parse(s)?;
  if xs.len() == 1 {
    match &xs[0] {
      Cirru::Leaf(s) => Err(format!("expected expr for data, got leaf: {}", s)),
      Cirru::List(_) => extract_cirru_edn(&xs[0]),
    }
  } else {
    Err(format!("Expected 1 expr for edn, got length {}: {:?} ", xs.len(), xs))
  }
}

fn extract_cirru_edn(node: &Cirru) -> Result<Edn, String> {
  match node {
    Cirru::Leaf(s) => match &**s {
      "nil" => Ok(Edn::Nil),
      "true" => Ok(Edn::Bool(true)),
      "false" => Ok(Edn::Bool(false)),
      "" => Err(String::from("empty string is invalid for edn")),
      s1 => match s1.chars().next().unwrap() {
        '\'' => Ok(Edn::Symbol(s1[1..].into())),
        ':' => Ok(Edn::tag(&s1[1..])),
        '"' | '|' => Ok(Edn::Str(s1[1..].into())),
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
          Cirru::Leaf(s) => match &**s {
            "quote" => {
              if xs.len() == 2 {
                Ok(Edn::Quote(xs[1].to_owned()))
              } else {
                Err(String::from("missing edn quote value"))
              }
            }
            "do" => {
              let mut ret: Option<Edn> = None;

              for x in xs.iter().skip(1) {
                if is_comment(x) {
                  continue;
                }
                if ret.is_some() {
                  return Err(String::from("multiple values in do"));
                }
                ret = Some(extract_cirru_edn(x)?);
              }
              if ret.is_none() {
                return Err(String::from("missing edn do value"));
              }
              ret.ok_or_else(|| String::from("missing edn do value"))
            }
            "::" => {
              let mut tag: Option<Edn> = None;
              let mut extra: Vec<Edn> = vec![];
              for x in xs.iter().skip(1) {
                if is_comment(x) {
                  continue;
                }
                if tag.is_some() {
                  extra.push(extract_cirru_edn(x)?);
                  continue;
                } else {
                  tag = Some(extract_cirru_edn(x)?);
                }
              }
              if let Some(x0) = tag {
                Ok(Edn::Tuple(EdnTupleView {
                  tag: Arc::new(x0),
                  extra,
                }))
              } else {
                Err(String::from("missing edn :: fst value"))
              }
            }
            "[]" => {
              let mut ys: Vec<Edn> = Vec::with_capacity(xs.len() - 1);
              for x in xs.iter().skip(1) {
                if is_comment(x) {
                  continue;
                }
                match extract_cirru_edn(x) {
                  Ok(v) => ys.push(v),
                  Err(v) => return Err(v),
                }
              }
              Ok(Edn::List(EdnListView(ys)))
            }
            "#{}" => {
              let mut ys: HashSet<Edn> = HashSet::new();
              for x in xs.iter().skip(1) {
                if is_comment(x) {
                  continue;
                }
                match extract_cirru_edn(x) {
                  Ok(v) => {
                    ys.insert(v);
                  }
                  Err(v) => return Err(v),
                }
              }
              Ok(Edn::Set(EdnSetView(ys)))
            }
            "{}" => {
              let mut zs: HashMap<Edn, Edn> = HashMap::new();
              for x in xs.iter().skip(1) {
                if is_comment(x) {
                  continue;
                }
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
              Ok(Edn::Map(EdnMapView(zs)))
            }
            "%{}" => {
              if xs.len() >= 3 {
                let name = match &xs[1] {
                  Cirru::Leaf(s) => EdnTag::new(s.strip_prefix(':').unwrap_or(s)),
                  Cirru::List(e) => return Err(format!("expected record name in string: {:?}", e)),
                };
                let mut entries: Vec<(EdnTag, Edn)> = Vec::with_capacity(xs.len() - 1);

                for x in xs.iter().skip(2) {
                  if is_comment(x) {
                    continue;
                  }
                  match x {
                    Cirru::Leaf(s) => return Err(format!("expected record, invalid record entry: {}", s)),
                    Cirru::List(ys) => {
                      if ys.len() == 2 {
                        match (&ys[0], extract_cirru_edn(&ys[1])) {
                          (Cirru::Leaf(s), Ok(v)) => {
                            entries.push((EdnTag::new(s.strip_prefix(':').unwrap_or(s)), v));
                          }
                          (Cirru::Leaf(s), Err(e)) => {
                            return Err(format!("invalid record value for `{}`, got: {}", s, e))
                          }
                          (Cirru::List(zs), _) => return Err(format!("invalid list as record key: {:?}", zs)),
                        }
                      } else {
                        return Err(format!("expected pair of 2: {:?}", ys));
                      }
                    }
                  }
                }
                if entries.is_empty() {
                  return Err(String::from("empty record is invalid"));
                }
                Ok(Edn::Record(EdnRecordView {
                  tag: name,
                  pairs: entries,
                }))
              } else {
                Err(String::from("insufficient items for edn record"))
              }
            }
            "buf" => {
              let mut ys: Vec<u8> = Vec::with_capacity(xs.len() - 1);
              for x in xs.iter().skip(1) {
                if is_comment(x) {
                  continue;
                }
                match x {
                  Cirru::Leaf(y) => {
                    if y.len() == 2 {
                      match hex::decode(&(**y)) {
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

fn is_comment(node: &Cirru) -> bool {
  match node {
    Cirru::Leaf(_) => false,
    Cirru::List(xs) => xs.first() == Some(&Cirru::Leaf(";".into())),
  }
}

fn assemble_cirru_node(data: &Edn) -> Cirru {
  match data {
    Edn::Nil => "nil".into(),
    Edn::Bool(v) => v.to_string().into(),
    Edn::Number(n) => n.to_string().into(),
    Edn::Symbol(s) => format!("'{}", s).into(),
    Edn::Tag(s) => format!(":{}", s).into(),
    Edn::Str(s) => format!("|{}", s).into(),
    Edn::Quote(v) => Cirru::List(vec!["quote".into(), (*v).to_owned()]),
    Edn::List(xs) => {
      let mut ys: Vec<Cirru> = Vec::with_capacity(xs.len() + 1);
      ys.push("[]".into());
      for x in &xs.0 {
        ys.push(assemble_cirru_node(x));
      }
      Cirru::List(ys)
    }
    Edn::Set(xs) => {
      let mut ys: Vec<Cirru> = Vec::with_capacity(xs.len() + 1);
      ys.push("#{}".into());
      let mut items = xs.0.iter().collect::<Vec<_>>();
      items.sort();
      for x in items {
        ys.push(assemble_cirru_node(x));
      }
      Cirru::List(ys)
    }
    Edn::Map(xs) => {
      let mut ys: Vec<Cirru> = Vec::with_capacity(xs.len() + 1);
      ys.push("{}".into());
      let mut items = Vec::from_iter(xs.0.iter());
      items.sort_by(|(a1, a2): &(&Edn, &Edn), (b1, b2): &(&Edn, &Edn)| {
        match (a1.is_literal(), b1.is_literal(), a2.is_literal(), b2.is_literal()) {
          (true, true, true, false) => Less,
          (true, true, false, true) => Greater,
          (true, false, ..) => Less,
          (false, true, ..) => Greater,
          _ => a1.cmp(b1),
        }
      });
      for (k, v) in items {
        ys.push(Cirru::List(vec![assemble_cirru_node(k), assemble_cirru_node(v)]))
      }
      Cirru::List(ys)
    }
    Edn::Record(EdnRecordView {
      tag: name,
      pairs: entries,
    }) => {
      let mut ys: Vec<Cirru> = Vec::with_capacity(entries.len() + 2);
      ys.push("%{}".into());
      ys.push(format!(":{}", name).into());
      let mut ordered_entries = entries.to_owned();
      ordered_entries.sort_by(|(_a1, a2), (_b1, b2)| match (a2.is_literal(), b2.is_literal()) {
        (true, false) => Less,
        (false, true) => Greater,
        _ => Equal,
      });
      for entry in ordered_entries {
        let v = &entry.1;
        ys.push(Cirru::List(vec![
          format!(":{}", entry.0).into(),
          assemble_cirru_node(v),
        ]));
      }

      Cirru::List(ys)
    }
    Edn::Tuple(EdnTupleView { tag, extra }) => {
      let mut ys: Vec<Cirru> = vec!["::".into(), assemble_cirru_node(tag)];
      for item in extra {
        ys.push(assemble_cirru_node(item))
      }
      Cirru::List(ys)
    }
    Edn::Buffer(buf) => {
      let mut ys: Vec<Cirru> = Vec::with_capacity(buf.len() + 1);
      ys.push("buf".into());
      for b in buf {
        ys.push(hex::encode(vec![b.to_owned()]).into());
      }
      Cirru::List(ys)
    }
  }
}

/// generate string from Edn
pub fn format(data: &Edn, use_inline: bool) -> Result<String, String> {
  let options = CirruWriterOptions { use_inline };
  match assemble_cirru_node(data) {
    Cirru::Leaf(s) => cirru_parser::format(&[vec!["do", &*s].into()], options),
    Cirru::List(xs) => cirru_parser::format(&[(Cirru::List(xs))], options),
  }
}
