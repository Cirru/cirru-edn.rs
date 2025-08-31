#![allow(clippy::mutable_key_type)]

use cirru_edn::{Edn, from_edn, to_edn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ApiDocWithSetAndTag {
  name: String,
  tags: HashSet<String>,
  description: String,
}

#[test]
fn test_set_and_tag_deserialization() {
  use cirru_edn::parse;

  // Create EDN data with Set containing Tags
  let edn_str = r#"
{}
  :name |test-function
  :tags $ #{} :functional :collection :utility
  :description "|A test function"
"#;
  let parsed_edn = parse(edn_str).expect("Failed to parse EDN");

  // Convert to our struct
  let api_doc: ApiDocWithSetAndTag = from_edn(parsed_edn).expect("Failed to deserialize");

  // Verify the deserialization worked correctly
  assert_eq!(api_doc.name, "test-function");
  assert_eq!(api_doc.description, "A test function");

  // Verify the Set contains the expected tags as strings
  let expected_tags: HashSet<String> = [
    "functional".to_string(),
    "collection".to_string(),
    "utility".to_string(),
  ]
  .into_iter()
  .collect();

  assert_eq!(api_doc.tags, expected_tags);

  // Test round-trip serialization
  let serialized_edn = to_edn(&api_doc).expect("Failed to serialize");
  let deserialized: ApiDocWithSetAndTag = from_edn(serialized_edn).expect("Failed to deserialize round-trip");

  assert_eq!(api_doc, deserialized);
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
  // Create an Edn map manually using Tags for struct fields
  let edn_map = Edn::map_from_iter([
    (Edn::tag("name"), "Charlie".into()),
    (Edn::tag("age"), 35i64.into()),
    (Edn::tag("is_active"), true.into()),
    (Edn::tag("scores"), vec![95.0, 87.5, 91.0].into()),
    (Edn::tag("metadata"), {
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
    (Edn::tag("name"), "John".into()),
    // Missing age, is_active, scores, metadata
  ]);

  // This should fail
  let result: Result<Person, _> = from_edn(incomplete_edn);
  assert!(result.is_err());
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct PersonWithHyphenatedFields {
  name: String,
  #[serde(rename = "first-name")]
  first_name: String,
  #[serde(rename = "last-name")]
  last_name: String,
  #[serde(rename = "is-active")]
  is_active: bool,
  #[serde(rename = "skill-level")]
  skill_level: u32,
}

#[test]
fn test_hyphenated_field_names() {
  let person = PersonWithHyphenatedFields {
    name: "Alice".to_string(),
    first_name: "Alice".to_string(),
    last_name: "Johnson".to_string(),
    is_active: true,
    skill_level: 8,
  };

  // Convert to Edn - should use hyphenated field names
  let edn_value = to_edn(&person).expect("Failed to convert to Edn");

  // Verify that the EDN contains hyphenated field names
  if let Edn::Map(map) = &edn_value {
    assert!(map.0.contains_key(&Edn::tag("first-name")));
    assert!(map.0.contains_key(&Edn::tag("last-name")));
    assert!(map.0.contains_key(&Edn::tag("is-active")));
    assert!(map.0.contains_key(&Edn::tag("skill-level")));
    // Should NOT contain underscored versions
    assert!(!map.0.contains_key(&Edn::tag("first_name")));
    assert!(!map.0.contains_key(&Edn::tag("last_name")));
    assert!(!map.0.contains_key(&Edn::tag("is_active")));
    assert!(!map.0.contains_key(&Edn::tag("skill_level")));
  } else {
    panic!("Expected Edn::Map");
  }

  // Convert back to struct
  let reconstructed: PersonWithHyphenatedFields = from_edn(edn_value).expect("Failed to convert from Edn");
  assert_eq!(person, reconstructed);
}

#[test]
fn test_manual_hyphenated_field_construction() {
  // Create an Edn map manually with hyphenated field names
  let edn_map = Edn::map_from_iter([
    (Edn::tag("name"), "Bob".into()),
    (Edn::tag("first-name"), "Bob".into()),
    (Edn::tag("last-name"), "Smith".into()),
    (Edn::tag("is-active"), false.into()),
    (Edn::tag("skill-level"), 5i64.into()),
  ]);

  // Convert to struct
  let person: PersonWithHyphenatedFields = from_edn(edn_map).expect("Failed to convert from manual Edn");

  assert_eq!(person.name, "Bob");
  assert_eq!(person.first_name, "Bob");
  assert_eq!(person.last_name, "Smith");
  assert!(!person.is_active);
  assert_eq!(person.skill_level, 5);
}
