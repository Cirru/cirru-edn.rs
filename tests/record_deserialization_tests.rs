//! Tests for Record deserialization functionality

#![allow(clippy::mutable_key_type)]
#![allow(clippy::bool_assert_comparison)]

extern crate cirru_edn;

use cirru_edn::{Edn, EdnRecordView, EdnTag, from_edn, to_edn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestPerson {
  name: String,
  age: u32,
  email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestPersonWithRename {
  name: String,
  age: u32,
  #[serde(rename = "email-address")]
  email_address: Option<String>,
  #[serde(rename = "is-active")]
  is_active: bool,
}

#[test]
fn test_record_to_struct_basic() {
  // Create a Record with person data
  let person_record = Edn::Record(EdnRecordView {
    tag: EdnTag::new("PersonRecord"), // This tag will be ignored
    pairs: vec![
      (EdnTag::new("name"), "Alice".into()),
      (EdnTag::new("age"), Edn::Number(30.0)),
      (EdnTag::new("email"), "alice@example.com".into()),
    ],
  });

  // Deserialize Record to Person struct
  let person: TestPerson = from_edn(person_record).unwrap();

  assert_eq!(person.name, "Alice");
  assert_eq!(person.age, 30);
  assert_eq!(person.email, Some("alice@example.com".to_string()));
}

#[test]
fn test_record_to_struct_with_nil() {
  // Create a Record with nil email
  let person_record = Edn::Record(EdnRecordView {
    tag: EdnTag::new("PersonRecord"),
    pairs: vec![
      (EdnTag::new("name"), "Bob".into()),
      (EdnTag::new("age"), Edn::Number(25.0)),
      (EdnTag::new("email"), Edn::Nil),
    ],
  });

  let person: TestPerson = from_edn(person_record).unwrap();

  assert_eq!(person.name, "Bob");
  assert_eq!(person.age, 25);
  assert_eq!(person.email, None);
}

#[test]
fn test_record_to_struct_with_renamed_fields() {
  // Create a Record with hyphenated field names
  let person_record = Edn::Record(EdnRecordView {
    tag: EdnTag::new("SpecialPersonRecord"),
    pairs: vec![
      (EdnTag::new("name"), "Charlie".into()),
      (EdnTag::new("age"), Edn::Number(35.0)),
      (EdnTag::new("email-address"), "charlie@example.com".into()),
      (EdnTag::new("is-active"), true.into()),
    ],
  });

  let person: TestPersonWithRename = from_edn(person_record).unwrap();

  assert_eq!(person.name, "Charlie");
  assert_eq!(person.age, 35);
  assert_eq!(person.email_address, Some("charlie@example.com".to_string()));
  assert_eq!(person.is_active, true);
}

#[test]
fn test_struct_serializes_to_map_not_record() {
  let person = TestPerson {
    name: "David".to_string(),
    age: 40,
    email: Some("david@example.com".to_string()),
  };

  let edn_value = to_edn(&person).unwrap();

  // Struct should serialize to Map, not Record
  match edn_value {
    Edn::Map(_) => {} // This is expected
    Edn::Record(_) => panic!("Struct should not serialize to Record"),
    _ => panic!("Struct should serialize to Map"),
  }
}

#[test]
fn test_roundtrip_record_to_struct_to_map() {
  // Start with a Record
  let original_record = Edn::Record(EdnRecordView {
    tag: EdnTag::new("PersonRecord"),
    pairs: vec![
      (EdnTag::new("name"), "Eve".into()),
      (EdnTag::new("age"), Edn::Number(28.0)),
      (EdnTag::new("email"), "eve@example.com".into()),
    ],
  });

  // Deserialize to struct
  let person: TestPerson = from_edn(original_record).unwrap();

  // Serialize back to EDN (should become Map)
  let serialized_edn = to_edn(&person).unwrap();

  // Should be able to deserialize again
  let person2: TestPerson = from_edn(serialized_edn).unwrap();

  assert_eq!(person, person2);
}

#[test]
fn test_record_ignores_tag_name() {
  // Test that different record tag names don't affect deserialization
  let record1 = Edn::Record(EdnRecordView {
    tag: EdnTag::new("PersonRecord"),
    pairs: vec![
      (EdnTag::new("name"), "Frank".into()),
      (EdnTag::new("age"), Edn::Number(32.0)),
      (EdnTag::new("email"), Edn::Nil),
    ],
  });

  let record2 = Edn::Record(EdnRecordView {
    tag: EdnTag::new("CompleteDifferentName"),
    pairs: vec![
      (EdnTag::new("name"), "Frank".into()),
      (EdnTag::new("age"), Edn::Number(32.0)),
      (EdnTag::new("email"), Edn::Nil),
    ],
  });

  let person1: TestPerson = from_edn(record1).unwrap();
  let person2: TestPerson = from_edn(record2).unwrap();

  assert_eq!(person1, person2);
}

#[test]
fn test_record_with_complex_nested_data() {
  #[derive(Debug, Serialize, Deserialize, PartialEq)]
  struct Department {
    name: String,
    employees: Vec<TestPerson>,
    metadata: HashMap<String, String>,
  }

  // Create a Record with nested data
  let dept_record = Edn::Record(EdnRecordView {
    tag: EdnTag::new("DepartmentRecord"),
    pairs: vec![
      (EdnTag::new("name"), "Engineering".into()),
      (EdnTag::new("employees"), {
        vec![
          Edn::Record(EdnRecordView {
            tag: EdnTag::new("PersonRecord"),
            pairs: vec![
              (EdnTag::new("name"), "Alice".into()),
              (EdnTag::new("age"), Edn::Number(30.0)),
              (EdnTag::new("email"), "alice@example.com".into()),
            ],
          }),
          Edn::Record(EdnRecordView {
            tag: EdnTag::new("PersonRecord"),
            pairs: vec![
              (EdnTag::new("name"), "Bob".into()),
              (EdnTag::new("age"), Edn::Number(25.0)),
              (EdnTag::new("email"), Edn::Nil),
            ],
          }),
        ]
        .into()
      }),
      (EdnTag::new("metadata"), {
        let mut map = HashMap::new();
        map.insert(Edn::Str("location".into()), "San Francisco".into());
        map.insert(Edn::Str("budget".into()), "1000000".into());
        Edn::Map(cirru_edn::EdnMapView(map))
      }),
    ],
  });

  let dept: Department = from_edn(dept_record).unwrap();

  assert_eq!(dept.name, "Engineering");
  assert_eq!(dept.employees.len(), 2);
  assert_eq!(dept.employees[0].name, "Alice");
  assert_eq!(dept.employees[1].name, "Bob");
  assert_eq!(dept.employees[1].email, None);
  assert_eq!(dept.metadata.get("location"), Some(&"San Francisco".to_string()));
}
