#![cfg(feature = "serde")]
#![allow(clippy::mutable_key_type)]

use cirru_edn::{from_edn, to_edn, Edn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Person {
  name: String,
  age: u32,
  is_active: bool,
  scores: Vec<f64>,
  metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Company {
  name: String,
  employees: Vec<Person>,
  founded_year: u32,
  headquarters: Option<String>,
}

#[test]
fn test_basic_serde_conversion() {
  let person = Person {
    name: "Alice".to_string(),
    age: 30,
    is_active: true,
    scores: vec![85.5, 92.0, 78.5],
    metadata: [("role".to_string(), "admin".to_string())].into_iter().collect(),
  };

  // Convert to Edn
  let edn_value = to_edn(&person).expect("Failed to convert to Edn");

  // Verify it's a map
  assert!(matches!(edn_value, Edn::Map(_)));

  // Convert back to struct
  let reconstructed: Person = from_edn(edn_value).expect("Failed to convert from Edn");

  assert_eq!(person, reconstructed);
}

#[test]
fn test_nested_struct_conversion() {
  let company = Company {
    name: "Tech Corp".to_string(),
    employees: vec![
      Person {
        name: "Alice".to_string(),
        age: 30,
        is_active: true,
        scores: vec![85.5, 92.0],
        metadata: HashMap::new(),
      },
      Person {
        name: "Bob".to_string(),
        age: 25,
        is_active: false,
        scores: vec![90.0, 88.5],
        metadata: [("department".to_string(), "engineering".to_string())]
          .into_iter()
          .collect(),
      },
    ],
    founded_year: 2020,
    headquarters: Some("San Francisco".to_string()),
  };

  // Convert to Edn
  let edn_value = to_edn(&company).expect("Failed to convert to Edn");

  // Convert back to struct
  let reconstructed: Company = from_edn(edn_value).expect("Failed to convert from Edn");

  assert_eq!(company, reconstructed);
}

#[test]
fn test_option_handling() {
  let company_with_hq = Company {
    name: "Tech Corp".to_string(),
    employees: vec![],
    founded_year: 2020,
    headquarters: Some("New York".to_string()),
  };

  let company_without_hq = Company {
    name: "Startup Inc".to_string(),
    employees: vec![],
    founded_year: 2023,
    headquarters: None,
  };

  // Test with Some value
  let edn1 = to_edn(&company_with_hq).unwrap();
  let reconstructed1: Company = from_edn(edn1).unwrap();
  assert_eq!(company_with_hq, reconstructed1);

  // Test with None value
  let edn2 = to_edn(&company_without_hq).unwrap();
  let reconstructed2: Company = from_edn(edn2).unwrap();
  assert_eq!(company_without_hq, reconstructed2);
}

#[test]
fn test_manual_edn_to_struct() {
  // Create an Edn map manually
  let edn_map = Edn::map_from_iter([
    ("name".into(), "Charlie".into()),
    ("age".into(), 35i64.into()),
    ("is_active".into(), true.into()),
    ("scores".into(), vec![95.0, 87.5, 91.0].into()),
    ("metadata".into(), {
      let mut meta = HashMap::new();
      meta.insert(Edn::Str("title".into()), Edn::Str("Senior Engineer".into()));
      Edn::Map(cirru_edn::EdnMapView(meta))
    }),
  ]);

  // Convert to struct
  let person: Person = from_edn(edn_map).expect("Failed to convert from manual Edn");

  assert_eq!(person.name, "Charlie");
  assert_eq!(person.age, 35);
  assert!(person.is_active);
  assert_eq!(person.scores, vec![95.0, 87.5, 91.0]);
  assert_eq!(person.metadata.get("title"), Some(&"Senior Engineer".to_string()));
}
#[test]
fn test_error_handling() {
  // Create an invalid Edn structure (missing required fields)
  let incomplete_edn = Edn::map_from_iter([
    ("name".into(), "John".into()),
    // Missing age, is_active, scores, metadata
  ]);

  // This should fail
  let result: Result<Person, _> = from_edn(incomplete_edn);
  assert!(result.is_err());
}
