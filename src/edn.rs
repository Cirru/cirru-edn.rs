mod any_ref;
mod list;
mod map;
mod record;
mod set;
mod tuple;

use std::{
  cmp::{
    Eq,
    Ordering::{self, *},
  },
  collections::{HashMap, HashSet},
  convert::{TryFrom, TryInto},
  fmt::{self, Write},
  hash::{Hash, Hasher},
  iter::FromIterator,
  ptr,
  sync::Arc,
};

use cirru_parser::Cirru;

pub use self::tuple::EdnTupleView;
pub use any_ref::{DynEq, EdnAnyRef};
pub use list::EdnListView;
pub use map::EdnMapView;
pub use record::EdnRecordView;
pub use set::EdnSetView;

use crate::tag::EdnTag;

/// Data format based on subset of EDN, but in Cirru syntax.
/// different parts are quote and Record.
#[derive(fmt::Debug, Clone)]
pub enum Edn {
  Nil,
  Bool(bool),
  Number(f64),
  Symbol(Arc<str>),
  Tag(EdnTag),
  Str(Arc<str>), // name collision
  Quote(Cirru),
  Tuple(EdnTupleView),
  List(EdnListView),
  Set(EdnSetView),
  Map(EdnMapView),
  Record(EdnRecordView),
  Buffer(Vec<u8>),
  /// reference to Rust data, not interpretable in Calcit
  AnyRef(EdnAnyRef),
  Atom(Box<Edn>),
}

impl fmt::Display for Edn {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Nil => f.write_str("nil"),
      Self::Bool(v) => f.write_fmt(format_args!("{v}")),
      Self::Number(n) => f.write_fmt(format_args!("{n}")),
      Self::Symbol(s) => f.write_fmt(format_args!("'{s}")),
      Self::Tag(s) => f.write_fmt(format_args!(":{s}")),
      Self::Str(s) => {
        if is_simple_token(s) {
          f.write_fmt(format_args!("|{s}"))
        } else {
          f.write_str("\"|")?;
          for c in s.chars() {
            if is_simple_char(c) {
              f.write_char(c)?;
            } else {
              f.write_str(&c.escape_default().to_string())?;
            }
          }
          f.write_char('"')
        }
      }
      Self::Quote(v) => f.write_fmt(format_args!("(quote {v})")),
      Self::Tuple(EdnTupleView { tag, extra }) => {
        let mut extra_str = String::new();
        for item in extra {
          extra_str.push(' ');
          extra_str.push_str(&item.to_string());
        }

        f.write_fmt(format_args!("(:: {tag}{extra_str})"))
      }
      Self::List(EdnListView(xs)) => {
        f.write_str("([]")?;
        for x in xs {
          f.write_fmt(format_args!(" {x}"))?;
        }
        f.write_str(")")
      }
      Self::Set(xs) => {
        f.write_str("(#{}")?;
        for x in &xs.0 {
          f.write_fmt(format_args!(" {x}"))?;
        }
        f.write_str(")")
      }
      Self::Map(xs) => {
        f.write_str("({}")?;
        for (k, v) in &xs.0 {
          f.write_fmt(format_args!(" ({k} {v})"))?;
        }
        f.write_str(")")
      }
      Self::Record(EdnRecordView {
        tag: name,
        pairs: entries,
      }) => {
        f.write_fmt(format_args!("(%{{}} :{name}"))?;

        for entry in entries {
          f.write_fmt(format_args!(" ({} {})", Edn::Tag(entry.0.to_owned()), entry.1))?;
        }

        f.write_str(")")
      }
      Self::Buffer(buf) => {
        f.write_str("(buf")?;
        for b in buf {
          f.write_str(" ")?;
          f.write_str(&hex::encode(vec![*b]))?;
        }
        f.write_str(")")
      }
      Self::AnyRef(_r) => f.write_str("(any-ref ...)"),
      Self::Atom(a) => f.write_fmt(format_args!("(atom {a})")),
    }
  }
}

/// check if a char is simple enough to be printed without quotes
pub fn is_simple_char(c: char) -> bool {
  matches!(c, '0'..='9' | 'A'..='Z' | 'a'..='z' | '-' | '?' | '.' | '$' | ',') || cjk::is_cjk_codepoint(c)
}

fn is_simple_token(tok: &str) -> bool {
  for s in tok.chars() {
    if !is_simple_char(s) {
      return false;
    }
  }
  true
}

impl Hash for Edn {
  fn hash<H>(&self, _state: &mut H)
  where
    H: Hasher,
  {
    match self {
      Self::Nil => "nil:".hash(_state),
      Self::Bool(v) => {
        "bool:".hash(_state);
        v.hash(_state);
      }
      Self::Number(n) => {
        "number:".hash(_state);
        (*n as usize).hash(_state) // TODO inaccurate solution
      }
      Self::Symbol(s) => {
        "symbol:".hash(_state);
        s.hash(_state);
      }
      Self::Tag(s) => {
        "tag:".hash(_state);
        s.hash(_state);
      }
      Self::Str(s) => {
        "string:".hash(_state);
        s.hash(_state);
      }
      Self::Quote(v) => {
        "quote:".hash(_state);
        v.hash(_state);
      }
      Self::Tuple(EdnTupleView { tag: pair, extra }) => {
        "tuple".hash(_state);
        pair.hash(_state);
        extra.hash(_state);
      }
      Self::List(v) => {
        "list:".hash(_state);
        v.hash(_state);
      }
      Self::Set(v) => {
        "set:".hash(_state);
        // TODO order for set is stable
        for x in &v.0 {
          x.hash(_state)
        }
      }
      Self::Map(v) => {
        "map:".hash(_state);
        // TODO order for map is not stable
        for x in &v.0 {
          x.hash(_state)
        }
      }
      Self::Record(EdnRecordView {
        tag: name,
        pairs: entries,
      }) => {
        "record:".hash(_state);
        name.hash(_state);
        entries.hash(_state);
      }
      Self::Buffer(buf) => {
        "buffer:".hash(_state);
        for b in buf {
          b.hash(_state);
        }
      }
      Self::AnyRef(h) => {
        "any-ref:".hash(_state);
        ptr::hash(h, _state);
      }
      Self::Atom(a) => {
        "atom:".hash(_state);
        a.hash(_state);
      }
    }
  }
}

impl Ord for Edn {
  fn cmp(&self, other: &Self) -> Ordering {
    match (self, other) {
      (Self::Nil, Self::Nil) => Equal,
      (Self::Nil, _) => Less,
      (_, Self::Nil) => Greater,

      (Self::Bool(a), Self::Bool(b)) => a.cmp(b),
      (Self::Bool(_), _) => Less,
      (_, Self::Bool(_)) => Greater,

      (Self::Number(a), Self::Number(b)) => {
        if a < b {
          Less
        } else if a > b {
          Greater
        } else {
          Equal
        }
      }
      (Self::Number(_), _) => Less,
      (_, Self::Number(_)) => Greater,

      (Self::Symbol(a), Self::Symbol(b)) => a.cmp(b),
      (Self::Symbol(_), _) => Less,
      (_, Self::Symbol(_)) => Greater,

      (Self::Tag(a), Self::Tag(b)) => a.cmp(b),
      (Self::Tag(_), _) => Less,
      (_, Self::Tag(_)) => Greater,

      (Self::Str(a), Self::Str(b)) => a.cmp(b),
      (Self::Str(_), _) => Less,
      (_, Self::Str(_)) => Greater,

      (Self::Quote(a), Self::Quote(b)) => a.cmp(b),
      (Self::Quote(_), _) => Less,
      (_, Self::Quote(_)) => Greater,

      (Self::Tuple(a), Self::Tuple(b)) => a.cmp(b),
      (Self::Tuple(..), _) => Less,
      (_, Self::Tuple(..)) => Greater,

      (Self::List(a), Self::List(b)) => a.cmp(b),
      (Self::List(_), _) => Less,
      (_, Self::List(_)) => Greater,

      (Self::Buffer(a), Self::Buffer(b)) => a.cmp(b),
      (Self::Buffer(_), _) => Less,
      (_, Self::Buffer(_)) => Greater,

      (Self::Set(a), Self::Set(b)) => match a.len().cmp(&b.len()) {
        Equal => unreachable!("TODO sets are not cmp ed"), // TODO
        a => a,
      },
      (Self::Set(_), _) => Less,
      (_, Self::Set(_)) => Greater,

      (Self::Map(a), Self::Map(b)) => {
        match a.len().cmp(&b.len()) {
          Equal => unreachable!("TODO maps are not cmp ed {:?} {:?}", a, b), // TODO
          a => a,
        }
      }
      (Self::Map(_), _) => Less,
      (_, Self::Map(_)) => Greater,

      (
        Self::Record(EdnRecordView {
          tag: name1,
          pairs: entries1,
        }),
        Self::Record(EdnRecordView {
          tag: name2,
          pairs: entries2,
        }),
      ) => name1.cmp(name2).then_with(|| entries1.cmp(entries2)),

      (Self::Record(..), _) => Less,
      (_, Self::Record(..)) => Greater,

      (Self::Atom(a), Self::Atom(b)) => a.cmp(b),
      (Self::Atom(_), _) => Less,
      (_, Self::Atom(_)) => Greater,

      (Self::AnyRef(a), Self::AnyRef(b)) => {
        if ptr::eq(a, b) {
          Equal
        } else {
          unreachable!("anyref are not cmp ed {:?} {:?}", a, b)
        }
      }
    }
  }
}

impl PartialOrd for Edn {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Eq for Edn {}

impl PartialEq for Edn {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Nil, Self::Nil) => true,
      (Self::Bool(a), Self::Bool(b)) => a == b,
      (Self::Number(a), Self::Number(b)) => (a - b).abs() < f64::EPSILON,
      (Self::Symbol(a), Self::Symbol(b)) => a == b,
      (Self::Tag(a), Self::Tag(b)) => a == b,
      (Self::Str(a), Self::Str(b)) => a == b,
      (Self::Quote(a), Self::Quote(b)) => a == b,
      (Self::Tuple(a), Self::Tuple(b)) => a == b,
      (Self::List(a), Self::List(b)) => a == b,
      (Self::Buffer(a), Self::Buffer(b)) => a == b,
      (Self::Set(a), Self::Set(b)) => a == b,
      (Self::Map(a), Self::Map(b)) => a == b,
      (Self::Record(a), Self::Record(b)) => a == b,
      (Self::AnyRef(a), Self::AnyRef(b)) => a == b,
      (Self::Atom(a), Self::Atom(b)) => a == b,
      (_, _) => false,
    }
  }
}

/// Support reading from EDN
impl Edn {
  /// create new string
  pub fn str<T: Into<Arc<str>>>(s: T) -> Self {
    Edn::Str(s.into())
  }
  /// create new tag
  pub fn tag<T: Into<Arc<str>>>(s: T) -> Self {
    Edn::Tag(EdnTag::new(s.into()))
  }
  /// create new symbol
  pub fn sym<T: Into<Arc<str>>>(s: T) -> Self {
    Edn::Symbol(s.into())
  }
  /// create new tuple
  pub fn tuple(tag: Self, extra: Vec<Self>) -> Self {
    Edn::Tuple(EdnTupleView {
      tag: Arc::new(tag),
      extra,
    })
  }
  /// create any-ref
  pub fn any_ref<T: ToOwned + DynEq + 'static>(d: T) -> Self {
    Edn::AnyRef(EdnAnyRef::new(d))
  }
  pub fn is_literal(&self) -> bool {
    matches!(
      self,
      Self::Nil | Self::Bool(_) | Self::Number(_) | Self::Symbol(_) | Self::Tag(_) | Self::Str(_)
    )
  }
  pub fn map_from_iter<T: IntoIterator<Item = (Edn, Edn)>>(pairs: T) -> Self {
    Self::Map(EdnMapView(HashMap::from_iter(pairs)))
  }
  pub fn record_from_pairs(tag: EdnTag, pairs: &[(EdnTag, Edn)]) -> Self {
    Self::Record(EdnRecordView {
      tag,
      pairs: pairs.to_vec(),
    })
  }
  pub fn read_string(&self) -> Result<String, String> {
    match self {
      Edn::Str(s) => Ok((**s).to_owned()),
      a => Err(format!("failed to convert to string: {a}")),
    }
  }
  pub fn read_symbol_string(&self) -> Result<String, String> {
    match self {
      Edn::Symbol(s) => Ok((**s).to_owned()),
      a => Err(format!("failed to convert to symbol: {a}")),
    }
  }
  pub fn read_str(&self) -> Result<Arc<str>, String> {
    match self {
      Edn::Str(s) => Ok(s.to_owned()),
      a => Err(format!("failed to convert to string: {a}")),
    }
  }
  pub fn read_symbol_str(&self) -> Result<Arc<str>, String> {
    match self {
      Edn::Symbol(s) => Ok(s.to_owned()),
      a => Err(format!("failed to convert to symbol: {a}")),
    }
  }
  pub fn read_tag_str(&self) -> Result<Arc<str>, String> {
    match self {
      Edn::Tag(s) => Ok(s.arc_str()),
      a => Err(format!("failed to convert to tag: {a}")),
    }
  }

  pub fn read_bool(&self) -> Result<bool, String> {
    match self {
      Edn::Bool(b) => Ok(*b),
      a => Err(format!("failed to convert to bool: {a}")),
    }
  }

  pub fn read_number(&self) -> Result<f64, String> {
    match self {
      Edn::Number(n) => Ok(*n),
      a => Err(format!("failed to convert to number: {a}")),
    }
  }

  pub fn read_quoted_cirru(&self) -> Result<Cirru, String> {
    match self {
      Edn::Quote(c) => Ok(c.to_owned()),
      a => Err(format!("failed to convert to cirru code: {a}")),
    }
  }

  // viewers

  /// get List variant in struct
  pub fn view_list(&self) -> Result<EdnListView, String> {
    match self {
      Edn::List(xs) => Ok((*xs).to_owned()),
      Edn::Nil => Ok(EdnListView::default()),
      a => Err(format!("failed to convert to list: {a}")),
    }
  }

  /// get Map variant in struct
  pub fn view_map(&self) -> Result<EdnMapView, String> {
    match self {
      Edn::Map(xs) => Ok(xs.to_owned()),
      Edn::Nil => Ok(EdnMapView::default()),
      a => Err(format!("failed to convert to map: {a}")),
    }
  }

  /// get Set variant in struct
  pub fn view_set(&self) -> Result<EdnSetView, String> {
    match self {
      Edn::Set(xs) => Ok(xs.to_owned()),
      Edn::Nil => Ok(EdnSetView::default()),
      a => Err(format!("failed to convert to set: {a}")),
    }
  }

  /// get Record variant in struct
  pub fn view_record(&self) -> Result<EdnRecordView, String> {
    match self {
      Edn::Record(EdnRecordView { tag, pairs }) => Ok(EdnRecordView {
        tag: tag.to_owned(),
        pairs: pairs.to_owned(),
      }),
      a => Err(format!("failed to convert to record: {a}")),
    }
  }

  /// get Tuple variant in struct
  pub fn view_tuple(&self) -> Result<EdnTupleView, String> {
    match self {
      Edn::Tuple(EdnTupleView { tag, extra }) => Ok(EdnTupleView {
        tag: tag.to_owned(),
        extra: extra.to_owned(),
      }),
      a => Err(format!("failed to convert to tuple: {a}")),
    }
  }
}

impl TryFrom<Edn> for EdnTag {
  type Error = String;
  fn try_from(x: Edn) -> Result<EdnTag, String> {
    match x {
      Edn::Tag(k) => Ok(k),
      _ => Err(format!("failed to convert to tag: {x}")),
    }
  }
}

impl From<EdnTag> for Edn {
  fn from(k: EdnTag) -> Edn {
    Edn::Tag(k)
  }
}

impl From<&EdnTag> for Edn {
  fn from(k: &EdnTag) -> Edn {
    Edn::Tag(k.to_owned())
  }
}

impl TryFrom<Edn> for String {
  type Error = String;
  fn try_from(x: Edn) -> Result<String, Self::Error> {
    match x {
      Edn::Str(s) => Ok((*s).to_owned()),
      Edn::Symbol(s) => Err(format!("cannot convert symbol {s} into string")),
      Edn::Tag(s) => Ok(s.to_string()),
      a => Err(format!("failed to convert to string: {a}")),
    }
  }
}

impl TryFrom<&Edn> for String {
  type Error = String;
  fn try_from(x: &Edn) -> Result<String, Self::Error> {
    match x {
      Edn::Str(s) => Ok((**s).to_owned()),
      Edn::Symbol(s) => Err(format!("cannot convert symbol {s} into string")),
      Edn::Tag(s) => Ok(s.to_string()),
      a => Err(format!("failed to convert to string: {a}")),
    }
  }
}

impl From<String> for Edn {
  fn from(x: String) -> Self {
    Edn::Str(x.into())
  }
}

impl From<&str> for Edn {
  fn from(x: &str) -> Self {
    Edn::Str(x.into())
  }
}

impl From<Box<str>> for Edn {
  fn from(x: Box<str>) -> Self {
    Edn::Str(x.into())
  }
}

impl From<&Box<str>> for Edn {
  fn from(x: &Box<str>) -> Self {
    Edn::Str((**x).into())
  }
}

impl TryFrom<Edn> for Arc<str> {
  type Error = String;
  fn try_from(x: Edn) -> Result<Self, Self::Error> {
    match x {
      Edn::Str(s) => Ok((*s).into()),
      Edn::Tag(s) => Ok(s.arc_str()),
      a => Err(format!("failed to convert to arc str: {a}")),
    }
  }
}

impl From<Arc<str>> for Edn {
  fn from(x: Arc<str>) -> Self {
    Edn::Str((*x).into())
  }
}

impl From<&Arc<str>> for Edn {
  fn from(x: &Arc<str>) -> Self {
    Edn::Str((**x).into())
  }
}

impl TryFrom<Edn> for bool {
  type Error = String;
  fn try_from(x: Edn) -> Result<Self, Self::Error> {
    match x {
      Edn::Bool(s) => Ok(s),
      a => Err(format!("failed to convert to bool: {a}")),
    }
  }
}

impl From<bool> for Edn {
  fn from(x: bool) -> Self {
    Edn::Bool(x)
  }
}

impl From<&bool> for Edn {
  fn from(x: &bool) -> Self {
    Edn::Bool(*x)
  }
}

impl TryFrom<Edn> for f64 {
  type Error = String;
  fn try_from(x: Edn) -> Result<Self, Self::Error> {
    match x {
      Edn::Number(s) => Ok(s),
      a => Err(format!("failed to convert to number: {a}")),
    }
  }
}

impl From<f64> for Edn {
  fn from(x: f64) -> Self {
    Edn::Number(x)
  }
}

impl From<&f64> for Edn {
  fn from(x: &f64) -> Self {
    Edn::Number(*x)
  }
}

impl TryFrom<Edn> for f32 {
  type Error = String;
  fn try_from(x: Edn) -> Result<Self, Self::Error> {
    match x {
      Edn::Number(s) => Ok(s as f32),
      a => Err(format!("failed to convert to number: {a}")),
    }
  }
}

impl From<f32> for Edn {
  fn from(x: f32) -> Self {
    Edn::Number(x as f64)
  }
}

impl From<&f32> for Edn {
  fn from(x: &f32) -> Self {
    Edn::Number(*x as f64)
  }
}

impl TryFrom<Edn> for i64 {
  type Error = String;
  fn try_from(x: Edn) -> Result<Self, Self::Error> {
    match x {
      Edn::Number(s) => Ok(s as i64),
      a => Err(format!("failed to convert to number: {a}")),
    }
  }
}

impl From<i64> for Edn {
  fn from(x: i64) -> Self {
    Edn::Number(x as f64)
  }
}

impl From<&i64> for Edn {
  fn from(x: &i64) -> Self {
    Edn::Number(*x as f64)
  }
}

impl From<u8> for Edn {
  fn from(x: u8) -> Self {
    Edn::Number(x as f64)
  }
}

impl From<&u8> for Edn {
  fn from(x: &u8) -> Self {
    Edn::Number(*x as f64)
  }
}

impl From<usize> for Edn {
  fn from(x: usize) -> Self {
    Edn::Number(x as f64)
  }
}

impl From<i32> for Edn {
  fn from(x: i32) -> Self {
    Edn::Number(x as f64)
  }
}

impl From<&i32> for Edn {
  fn from(x: &i32) -> Self {
    Edn::Number(*x as f64)
  }
}

impl TryFrom<Edn> for u8 {
  type Error = String;
  fn try_from(x: Edn) -> Result<Self, Self::Error> {
    match x {
      Edn::Number(s) => {
        if s >= u8::MIN as f64 && s <= u8::MAX as f64 && s.fract().abs() <= f64::EPSILON {
          Ok(s as u8)
        } else {
          Err(format!("invalid u8 value: {s}"))
        }
      }
      a => Err(format!("failed to convert to u8: {a}")),
    }
  }
}

impl From<i8> for Edn {
  fn from(x: i8) -> Self {
    Edn::Number(x as f64)
  }
}

impl From<&i8> for Edn {
  fn from(x: &i8) -> Self {
    Edn::Number(*x as f64)
  }
}

impl From<&[Edn]> for Edn {
  fn from(xs: &[Edn]) -> Self {
    Edn::List(EdnListView(xs.to_vec()))
  }
}

impl TryFrom<Edn> for i8 {
  type Error = String;
  fn try_from(x: Edn) -> Result<Self, Self::Error> {
    match x {
      Edn::Number(s) => {
        if s >= i8::MIN as f64 && s <= i8::MAX as f64 && s.fract().abs() <= f64::EPSILON {
          Ok(s as i8)
        } else {
          Err(format!("invalid i8 value: {s}"))
        }
      }
      a => Err(format!("failed to convert to i8: {a}")),
    }
  }
}

impl From<Cirru> for Edn {
  fn from(x: Cirru) -> Self {
    Edn::Quote(x)
  }
}

impl From<&Cirru> for Edn {
  fn from(x: &Cirru) -> Self {
    Edn::Quote(x.to_owned())
  }
}

impl TryFrom<Edn> for Cirru {
  type Error = String;
  fn try_from(x: Edn) -> Result<Self, Self::Error> {
    match x {
      Edn::Quote(s) => Ok(s),
      a => Err(format!("failed to convert to cirru code: {a}")),
    }
  }
}

impl<T> TryFrom<Edn> for Vec<T>
where
  T: TryFrom<Edn, Error = String>,
{
  type Error = String;
  fn try_from(x: Edn) -> Result<Self, Self::Error> {
    match x {
      Edn::List(xs) => {
        let mut ys = Vec::new();
        for x in xs.0 {
          let y = x.try_into()?;
          ys.push(y);
        }
        Ok(ys)
      }
      Edn::Nil => Ok(vec![]),
      a => Err(format!("failed to convert to vec: {a}")),
    }
  }
}

/// `Option<T>` is a special case to convert since it has it's own implementation in core.
/// To handle `Edn::Nil` which is dynamically typed, some code like this is required:
/// ```ignore
/// {
///   let v = value.map_get("<FIELD_NAME>")?;
///   if v == Edn::Nil {
///     None
///   } else {
///     Some(v.try_into()?)
///   }
/// }
/// ```
impl<T> From<Option<T>> for Edn
where
  T: Into<Edn>,
{
  fn from(xs: Option<T>) -> Self {
    match xs {
      Some(x) => x.into(),
      None => Edn::Nil,
    }
  }
}

impl<'a, T> From<&'a Option<&'a T>> for Edn
where
  T: Into<Edn> + Clone,
{
  fn from(xs: &'a Option<&'a T>) -> Self {
    match xs {
      Some(x) => (*x).to_owned().into(),
      None => Edn::Nil,
    }
  }
}

impl<T> From<Vec<T>> for Edn
where
  T: Into<Edn>,
{
  fn from(xs: Vec<T>) -> Self {
    Edn::List(EdnListView(xs.into_iter().map(|x| x.into()).collect()))
  }
}

impl<'a, T> From<&'a Vec<&'a T>> for Edn
where
  T: Into<Edn> + Clone,
{
  fn from(xs: &'a Vec<&'a T>) -> Self {
    Edn::List(EdnListView(xs.iter().map(|x| (*x).to_owned().into()).collect()))
  }
}

impl<'a, T> From<&'a [&'a T]> for Edn
where
  T: Into<Edn> + Clone,
{
  fn from(xs: &'a [&'a T]) -> Self {
    Edn::List(EdnListView(xs.iter().map(|x| (*x).to_owned().into()).collect()))
  }
}

impl<T> TryFrom<Edn> for HashSet<T>
where
  T: TryFrom<Edn, Error = String> + Eq + Hash,
{
  type Error = String;
  fn try_from(x: Edn) -> Result<Self, Self::Error> {
    match x {
      Edn::Set(xs) => {
        let mut ys = HashSet::new();
        for x in xs.0 {
          let y = x.try_into()?;
          ys.insert(y);
        }
        Ok(ys)
      }
      Edn::Nil => Ok(HashSet::new()),
      a => Err(format!("failed to convert to vec: {a}")),
    }
  }
}

impl<T> From<HashSet<T>> for Edn
where
  T: Into<Edn>,
{
  fn from(xs: HashSet<T>) -> Self {
    Edn::Set(EdnSetView(xs.into_iter().map(|x| x.into()).collect()))
  }
}

impl<'a, T> From<&'a HashSet<&'a T>> for Edn
where
  T: Into<Edn> + Clone,
{
  fn from(xs: &'a HashSet<&'a T>) -> Self {
    Edn::Set(EdnSetView(xs.iter().map(|x| (*x).to_owned().into()).collect()))
  }
}

impl<T, K> TryFrom<Edn> for HashMap<K, T>
where
  T: TryFrom<Edn, Error = String>,
  K: TryFrom<Edn, Error = String> + Eq + Hash,
{
  type Error = String;
  fn try_from(x: Edn) -> Result<Self, Self::Error> {
    match x {
      Edn::Map(xs) => {
        let mut ys = HashMap::new();
        for (k, v) in &xs.0 {
          let k = k.to_owned().try_into()?;
          let v = v.to_owned().try_into()?;
          ys.insert(k, v);
        }
        Ok(ys)
      }
      Edn::Nil => Ok(HashMap::new()),
      a => Err(format!("failed to convert to vec: {a}")),
    }
  }
}

impl<T, K> From<HashMap<K, T>> for Edn
where
  T: Into<Edn>,
  K: Into<Edn>,
{
  fn from(xs: HashMap<K, T>) -> Self {
    Edn::Map(EdnMapView(xs.into_iter().map(|(k, v)| (k.into(), v.into())).collect()))
  }
}

impl<'a, T, K> From<&'a HashMap<&'a K, &'a T>> for Edn
where
  T: Into<Edn> + Clone,
  K: Into<Edn> + Clone,
{
  fn from(xs: &'a HashMap<&'a K, &'a T>) -> Self {
    Edn::Map(EdnMapView(
      xs.iter()
        .map(|(k, v)| ((*k).to_owned().into(), (*v).to_owned().into()))
        .collect(),
    ))
  }
}

impl From<(Arc<Edn>, Vec<Edn>)> for Edn {
  fn from((tag, extra): (Arc<Edn>, Vec<Edn>)) -> Edn {
    Edn::Tuple(EdnTupleView { tag, extra })
  }
}
