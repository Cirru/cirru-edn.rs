#![allow(clippy::mutable_key_type)]

use cirru_edn::{from_edn, to_edn, Edn};
use cirru_parser::Cirru;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeEntry {
  pub doc: String,
  pub code: Cirru,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CodeLibrary {
  name: String,
  entries: Vec<CodeEntry>,
}

#[test]
fn test_quote_to_code_entry_conversion() {
  // Create EDN Quote and deserialize to CodeEntry
  let quote_code = Edn::Quote(Cirru::List(vec![
    Cirru::Leaf("map".into()),
    Cirru::Leaf("inc".into()),
    Cirru::List(vec![Cirru::Leaf("range".into()), Cirru::Leaf("10".into())]),
  ]));

  // Create a manual EDN with Quote that should deserialize to CodeEntry
  let manual_entry_edn = Edn::map_from_iter([
    (Edn::tag("doc"), "Map increment over range".into()),
    (Edn::tag("code"), quote_code), // Cirru EDN 当中在 code 字段中直接使用 Quote
  ]);

  // Deserialize to CodeEntry
  let manual_entry: CodeEntry = from_edn(manual_entry_edn).expect("Failed to deserialize CodeEntry from Quote");

  // Verify the structure
  assert_eq!(manual_entry.doc, "Map increment over range");

  // Verify the code structure
  if let Cirru::List(ref items) = manual_entry.code {
    assert_eq!(items.len(), 3);
    assert_eq!(items[0], Cirru::Leaf("map".into()));
    assert_eq!(items[1], Cirru::Leaf("inc".into()));
    if let Cirru::List(ref range_items) = items[2] {
      assert_eq!(range_items.len(), 2);
      assert_eq!(range_items[0], Cirru::Leaf("range".into()));
      assert_eq!(range_items[1], Cirru::Leaf("10".into()));
    } else {
      panic!("Expected List for range expression");
    }
  } else {
    panic!("Expected List for code");
  }
}

#[test]
fn test_code_entry_roundtrip() {
  // Create CodeEntry with simple Cirru code
  let simple_entry = CodeEntry {
    doc: "Simple greeting function".to_string(),
    code: Cirru::List(vec![
      Cirru::Leaf("println!".into()),
      Cirru::Leaf("\"Hello, world!\"".into()),
    ]),
  };

  // Serialize and deserialize
  let serialized = to_edn(&simple_entry).expect("Failed to serialize CodeEntry");
  let deserialized: CodeEntry = from_edn(serialized).expect("Failed to deserialize CodeEntry");

  assert_eq!(simple_entry, deserialized);
}

#[test]
fn test_complex_code_entry_roundtrip() {
  // Create CodeEntry with complex nested Cirru code
  let complex_entry = CodeEntry {
    doc: "Factorial function with recursion".to_string(),
    code: Cirru::List(vec![
      Cirru::Leaf("defn".into()),
      Cirru::Leaf("factorial".into()),
      Cirru::List(vec![Cirru::Leaf("n".into())]),
      Cirru::List(vec![
        Cirru::Leaf("if".into()),
        Cirru::List(vec![
          Cirru::Leaf("<=".into()),
          Cirru::Leaf("n".into()),
          Cirru::Leaf("1".into()),
        ]),
        Cirru::Leaf("1".into()),
        Cirru::List(vec![
          Cirru::Leaf("*".into()),
          Cirru::Leaf("n".into()),
          Cirru::List(vec![
            Cirru::Leaf("factorial".into()),
            Cirru::List(vec![
              Cirru::Leaf("-".into()),
              Cirru::Leaf("n".into()),
              Cirru::Leaf("1".into()),
            ]),
          ]),
        ]),
      ]),
    ]),
  };

  let serialized = to_edn(&complex_entry).expect("Failed to serialize complex CodeEntry");
  let deserialized: CodeEntry = from_edn(serialized).expect("Failed to deserialize complex CodeEntry");

  assert_eq!(complex_entry, deserialized);
}

#[test]
fn test_code_library_with_multiple_entries() {
  let simple_entry = CodeEntry {
    doc: "Simple greeting function".to_string(),
    code: Cirru::List(vec![
      Cirru::Leaf("println!".into()),
      Cirru::Leaf("\"Hello, world!\"".into()),
    ]),
  };

  let add_entry = CodeEntry {
    doc: "Add two numbers".to_string(),
    code: Cirru::List(vec![
      Cirru::Leaf("fn".into()),
      Cirru::Leaf("add".into()),
      Cirru::List(vec![Cirru::Leaf("a".into()), Cirru::Leaf("b".into())]),
      Cirru::List(vec![
        Cirru::Leaf("+".into()),
        Cirru::Leaf("a".into()),
        Cirru::Leaf("b".into()),
      ]),
    ]),
  };

  let library = CodeLibrary {
    name: "Basic Functions".to_string(),
    entries: vec![simple_entry, add_entry],
  };

  let serialized = to_edn(&library).expect("Failed to serialize CodeLibrary");
  let deserialized: CodeLibrary = from_edn(serialized).expect("Failed to deserialize CodeLibrary");

  assert_eq!(library, deserialized);
}

#[test]
fn test_quote_serialization_becomes_regular_cirru() {
  // When we serialize a CodeEntry, the Cirru should become a regular Cirru value, not a Quote
  let entry = CodeEntry {
    doc: "Test function".to_string(),
    code: Cirru::List(vec![Cirru::Leaf("test".into()), Cirru::Leaf("123".into())]),
  };

  let serialized = to_edn(&entry).expect("Failed to serialize");

  // Verify the structure
  if let Edn::Map(ref map) = serialized {
    let code_value = map.0.get(&Edn::tag("code")).expect("Should have code field");

    // The code should be serialized as a regular Cirru value, not a Quote
    match code_value {
      Edn::List(_) => {
        // This is expected - Cirru serializes as List
      }
      Edn::Quote(_) => {
        panic!("Code should not be serialized as Quote when coming from struct field");
      }
      other => {
        panic!("Unexpected code value type: {:?}", other);
      }
    }
  } else {
    panic!("Expected Map for serialized CodeEntry");
  }
}
