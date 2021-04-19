use cirru_parser::CirruNode;
use core::cmp::Ord;
use regex::Regex;
use std::cmp::Eq;
use std::cmp::Ordering;
use std::cmp::Ordering::*;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};

/// Data format based on subset of EDN, but in Cirru syntax.
/// different parts are quote and Record.
#[derive(fmt::Debug, Clone)]
pub enum CirruEdn {
  CirruEdnNil,
  CirruEdnBool(bool),
  CirruEdnNumber(f32),
  CirruEdnSymbol(String),
  CirruEdnKeyword(String),
  CirruEdnString(String),
  CirruEdnQuote(CirruNode),
  CirruEdnList(Vec<CirruEdn>),
  CirruEdnSet(HashSet<CirruEdn>),
  CirruEdnMap(HashMap<CirruEdn, CirruEdn>),
  CirruEdnRecord(String, Vec<String>, Vec<CirruEdn>),
}

use CirruEdn::*;

lazy_static! {
  static ref RE_SIMPLE_TOKEN: Regex = Regex::new("^[\\d\\w\\-\\?\\.\\$,]+$").unwrap();
}

impl fmt::Display for CirruEdn {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      CirruEdnNil => f.write_str("nil"),
      CirruEdnBool(v) => f.write_str(&format!("{}", v)),
      CirruEdnNumber(n) => f.write_str(&format!("{}", n)),
      CirruEdnSymbol(s) => f.write_str(&format!("'{}", s)),
      CirruEdnKeyword(s) => f.write_str(&format!(":{}", s)),
      CirruEdnString(s) => {
        if RE_SIMPLE_TOKEN.is_match(s) {
          f.write_str(&format!("|{}", s))
        } else {
          f.write_str(&format!("\"|{}\"", s))
        }
      }
      CirruEdnQuote(v) => f.write_str(&format!("(quote {})", v)),
      CirruEdnList(xs) => {
        f.write_str("([]")?;
        for x in xs {
          f.write_str(&format!(" {}", x))?;
        }
        f.write_str(")")
      }
      CirruEdnSet(xs) => {
        f.write_str("(#{}")?;
        for x in xs {
          f.write_str(&format!(" {}", x))?;
        }
        f.write_str(")")
      }
      CirruEdnMap(xs) => {
        f.write_str("({}")?;
        for (k, v) in xs {
          f.write_str(&format!(" ({} {})", k, v))?;
        }
        f.write_str(")")
      }
      CirruEdnRecord(name, fields, values) => {
        f.write_str(&format!("(%{{}} {}", name))?;

        for idx in 0..fields.len() {
          f.write_str(&format!("({} {})", fields[idx], values[idx]))?;
        }

        f.write_str(")")
      }
    }
  }
}

impl Hash for CirruEdn {
  fn hash<H>(&self, _state: &mut H)
  where
    H: Hasher,
  {
    match self {
      CirruEdnNil => "nil:".hash(_state),
      CirruEdnBool(v) => {
        "bool:".hash(_state);
        v.hash(_state);
      }
      CirruEdnNumber(n) => {
        "number:".hash(_state);
        (*n as usize).hash(_state) // TODO inaccurate solution
      }
      CirruEdnSymbol(s) => {
        "symbol:".hash(_state);
        s.hash(_state);
      }
      CirruEdnKeyword(s) => {
        "keyword:".hash(_state);
        s.hash(_state);
      }
      CirruEdnString(s) => {
        "string:".hash(_state);
        s.hash(_state);
      }
      CirruEdnQuote(v) => {
        "quote:".hash(_state);
        v.hash(_state);
      }
      CirruEdnList(v) => {
        "list:".hash(_state);
        v.hash(_state);
      }
      CirruEdnSet(v) => {
        "set:".hash(_state);
        // TODO order for set is stable
        for x in v {
          x.hash(_state)
        }
      }
      CirruEdnMap(v) => {
        "map:".hash(_state);
        // TODO order for map is not stable
        for x in v {
          x.hash(_state)
        }
      }
      CirruEdnRecord(name, fields, values) => {
        "record:".hash(_state);
        name.hash(_state);
        fields.hash(_state);
        values.hash(_state);
      }
    }
  }
}

impl Ord for CirruEdn {
  fn cmp(&self, other: &Self) -> Ordering {
    match (self, other) {
      (CirruEdnNil, CirruEdnNil) => Equal,
      (CirruEdnNil, _) => Less,
      (_, CirruEdnNil) => Greater,

      (CirruEdnBool(a), CirruEdnBool(b)) => a.cmp(b),
      (CirruEdnBool(_), _) => Less,
      (_, CirruEdnBool(_)) => Greater,

      (CirruEdnNumber(a), CirruEdnNumber(b)) => {
        if a < b {
          Less
        } else if a > b {
          Greater
        } else {
          Equal
        }
      }
      (CirruEdnNumber(_), _) => Less,
      (_, CirruEdnNumber(_)) => Greater,

      (CirruEdnSymbol(a), CirruEdnSymbol(b)) => a.cmp(b),
      (CirruEdnSymbol(_), _) => Less,
      (_, CirruEdnSymbol(_)) => Greater,

      (CirruEdnKeyword(a), CirruEdnKeyword(b)) => a.cmp(b),
      (CirruEdnKeyword(_), _) => Less,
      (_, CirruEdnKeyword(_)) => Greater,

      (CirruEdnString(a), CirruEdnString(b)) => a.cmp(b),
      (CirruEdnString(_), _) => Less,
      (_, CirruEdnString(_)) => Greater,

      (CirruEdnQuote(a), CirruEdnQuote(b)) => a.cmp(b),
      (CirruEdnQuote(_), _) => Less,
      (_, CirruEdnQuote(_)) => Greater,

      (CirruEdnList(a), CirruEdnList(b)) => a.cmp(b),
      (CirruEdnList(_), _) => Less,
      (_, CirruEdnList(_)) => Greater,

      (CirruEdnSet(a), CirruEdnSet(b)) => match a.len().cmp(&b.len()) {
        Equal => {
          unreachable!("TODO sets are not cmp ed") // TODO
        }
        a => a,
      },
      (CirruEdnSet(_), _) => Less,
      (_, CirruEdnSet(_)) => Greater,

      (CirruEdnMap(a), CirruEdnMap(b)) => {
        unreachable!(format!("TODO maps are not cmp ed {:?} {:?}", a, b)) // TODO
      }
      (CirruEdnMap(_), _) => Less,
      (_, CirruEdnMap(_)) => Greater,

      (CirruEdnRecord(_name1, _fields1, _values1), CirruEdnRecord(_name2, _fields2, _values2)) => {
        unreachable!("TODO records are not cmp ed") // TODO
      }
    }
  }
}

impl PartialOrd for CirruEdn {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Eq for CirruEdn {}

impl PartialEq for CirruEdn {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (CirruEdnNil, CirruEdnNil) => true,
      (CirruEdnBool(a), CirruEdnBool(b)) => a == b,
      (CirruEdnNumber(a), CirruEdnNumber(b)) => a == b,
      (CirruEdnSymbol(a), CirruEdnSymbol(b)) => a == b,
      (CirruEdnKeyword(a), CirruEdnKeyword(b)) => a == b,
      (CirruEdnString(a), CirruEdnString(b)) => a == b,
      (CirruEdnQuote(a), CirruEdnQuote(b)) => a == b,
      (CirruEdnList(a), CirruEdnList(b)) => a == b,
      (CirruEdnSet(a), CirruEdnSet(b)) => a == b,
      (CirruEdnMap(a), CirruEdnMap(b)) => a == b,
      (CirruEdnRecord(name1, fields1, values1), CirruEdnRecord(name2, fields2, values2)) => {
        name1 == name2 && fields1 == fields2 && values1 == values2
      }
      (_, _) => false,
    }
  }
}
