//! # Serde Demo
//!
//! This example demonstrates basic serde integration with Cirru EDN,
//! including serialization, deserialization, error handling, manual EDN construction,
//! and special handling of Cirru EDN Records.

#![allow(clippy::mutable_key_type)]
#![allow(clippy::uninlined_format_args)]

use cirru_edn::{from_edn, to_edn, Edn, EdnRecordView, EdnTag};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Person {
  name: String,
  age: u32,
  email: Option<String>,
  tags: Vec<String>,
  scores: HashMap<String, f64>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Department {
  name: String,
  employees: Vec<Person>,
  budget: f64,
  active: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct PersonWithSpecialFields {
  name: String,
  age: u32,
  // Rust 字段名使用下划线，但在 EDN 中映射为连字符
  #[serde(rename = "first-name")]
  first_name: String,
  #[serde(rename = "last-name")]
  last_name: String,
  #[serde(rename = "is-active")]
  is_active: bool,
  #[serde(rename = "email-address")]
  email_address: Option<String>,
  #[serde(rename = "skill-level")]
  skill_level: u32,
}

fn main() -> Result<(), String> {
  println!("=== Cirru EDN Serde Support Demo ===\n");

  // 1. Basic round-trip conversion demo
  demo_basic_roundtrip()?;

  // 2. Manual Edn construction demo
  demo_manual_construction()?;

  // 3. Error handling demo
  demo_error_handling()?;

  // 4. Special field names with serde rename demo
  demo_special_field_names()?;

  // 5. Manual construction with hyphenated field names demo
  demo_manual_hyphenated_fields()?;

  // 6. Record deserialization demo (new feature)
  demo_record_deserialization()?;

  println!("🎉 All demonstrations completed successfully!");
  Ok(())
}

/// Demonstrates basic struct to Edn conversion and back
fn demo_basic_roundtrip() -> Result<(), String> {
  // Create a complex nested structure
  let dept = Department {
    name: "Engineering".to_string(),
    employees: vec![
      Person {
        name: "Alice".to_string(),
        age: 30,
        email: Some("alice@company.com".to_string()),
        tags: vec!["senior".to_string(), "rust".to_string()],
        scores: [("performance".to_string(), 4.5), ("leadership".to_string(), 4.0)]
          .into_iter()
          .collect(),
      },
      Person {
        name: "Bob".to_string(),
        age: 25,
        email: None,
        tags: vec!["junior".to_string(), "javascript".to_string()],
        scores: [("performance".to_string(), 3.8), ("creativity".to_string(), 4.2)]
          .into_iter()
          .collect(),
      },
    ],
    budget: 250000.75,
    active: true,
  };

  println!("Original Department structure:");
  println!("{:#?}\n", dept);

  // Convert to Edn
  println!("1. Converting Rust struct to Edn...");
  let edn_value = to_edn(&dept)?;
  println!("Edn representation:");
  println!("{}\n", edn_value);

  // Convert back to Rust struct
  println!("2. Converting Edn back to Rust struct...");
  let reconstructed: Department = from_edn(edn_value)?;
  println!("Reconstructed Department:");
  println!("{:#?}\n", reconstructed);

  // Verify they are equal
  println!("3. Verifying round-trip conversion...");
  if dept == reconstructed {
    println!("✅ Success! Round-trip conversion works perfectly.\n");
  } else {
    println!("❌ Error! Round-trip conversion failed.\n");
    return Err("Round-trip conversion failed".to_string());
  }

  Ok(())
}

/// Demonstrates manual Edn construction and conversion
fn demo_manual_construction() -> Result<(), String> {
  println!("4. Manual Edn construction and conversion...");
  let manual_person = Edn::map_from_iter([
    (Edn::tag("name"), "Charlie".into()),
    (Edn::tag("age"), Edn::Number(35.0)),
    (Edn::tag("email"), "charlie@company.com".into()),
    (Edn::tag("tags"), vec!["staff".to_string(), "python".to_string()].into()),
    (Edn::tag("scores"), {
      let mut scores = HashMap::new();
      scores.insert(Edn::Str("performance".into()), Edn::Number(4.1));
      scores.insert(Edn::Str("teamwork".into()), Edn::Number(4.7));
      Edn::Map(cirru_edn::EdnMapView(scores))
    }),
  ]);

  let charlie: Person = from_edn(manual_person)?;
  println!("Charlie from manual Edn:");
  println!("{:#?}\n", charlie);

  Ok(())
}

/// Demonstrates error handling when deserializing incomplete data
fn demo_error_handling() -> Result<(), String> {
  println!("5. Error handling example...");
  let incomplete_edn = Edn::map_from_iter([
    (Edn::tag("name"), "Invalid".into()),
    // Missing required fields
  ]);

  match from_edn::<Person>(incomplete_edn) {
    Ok(_) => println!("Unexpected success"),
    Err(e) => println!("Expected error: {}\n", e),
  }

  Ok(())
}

/// Demonstrates special field names with serde rename
fn demo_special_field_names() -> Result<(), String> {
  println!("6. Special field names with serde rename...");
  let special_person = PersonWithSpecialFields {
    name: "David".to_string(),
    age: 28,
    first_name: "David".to_string(),
    last_name: "Wilson".to_string(),
    is_active: true,
    email_address: Some("david.wilson@company.com".to_string()),
    skill_level: 5,
  };

  println!("Original PersonWithSpecialFields:");
  println!("{:#?}\n", special_person);

  // Convert to Edn (should use hyphenated field names)
  let special_edn = to_edn(&special_person)?;
  println!("EDN with special field names:");
  println!("{}\n", special_edn);

  // Convert back to struct
  let reconstructed_special: PersonWithSpecialFields = from_edn(special_edn)?;
  println!("Reconstructed PersonWithSpecialFields:");
  println!("{:#?}\n", reconstructed_special);

  // Verify round-trip
  if special_person == reconstructed_special {
    println!("✅ Special field names round-trip successful!\n");
  } else {
    println!("❌ Special field names round-trip failed!\n");
    return Err("Special field names round-trip failed".to_string());
  }

  Ok(())
}

/// Demonstrates manual construction with hyphenated field names
fn demo_manual_hyphenated_fields() -> Result<(), String> {
  println!("7. Manual construction with hyphenated field names...");
  let manual_special = Edn::map_from_iter([
    (Edn::tag("name"), "Emma".into()),
    (Edn::tag("age"), Edn::Number(32.0)),
    (Edn::tag("first-name"), "Emma".into()),
    (Edn::tag("last-name"), "Chen".into()),
    (Edn::tag("is-active"), false.into()),
    (Edn::tag("email-address"), "emma.chen@company.com".into()),
    (Edn::tag("skill-level"), Edn::Number(8.0)),
  ]);

  let emma: PersonWithSpecialFields = from_edn(manual_special)?;
  println!("Emma from manual EDN with hyphenated fields:");
  println!("{:#?}\n", emma);

  Ok(())
}

/// Demonstrates Cirru EDN Record deserialization
/// Records in Cirru EDN have named types, but Rust structs don't expose their name at runtime.
/// This demo shows how Records can be deserialized to structs by ignoring the record name.
fn demo_record_deserialization() -> Result<(), String> {
  println!("8. Cirru EDN Record deserialization demo...");

  // Create a record manually - this represents what might come from EDN text parsing
  let person_record = Edn::Record(EdnRecordView {
    tag: EdnTag::new("PersonRecord"), // This name will be ignored during deserialization
    pairs: vec![
      (EdnTag::new("name"), "Frank".into()),
      (EdnTag::new("age"), Edn::Number(42.0)),
      (EdnTag::new("email"), "frank@company.com".into()),
      (
        EdnTag::new("tags"),
        vec!["manager".to_string(), "leadership".to_string()].into(),
      ),
      (EdnTag::new("scores"), {
        let mut scores = HashMap::new();
        scores.insert(Edn::Str("strategic".into()), Edn::Number(4.8));
        scores.insert(Edn::Str("communication".into()), Edn::Number(4.5));
        Edn::Map(cirru_edn::EdnMapView(scores))
      }),
    ],
  });

  println!("Original EDN Record:");
  println!("{}\n", person_record);

  // Deserialize Record to Person struct (ignoring the record name)
  let frank: Person = from_edn(person_record)?;
  println!("Person deserialized from Record:");
  println!("{:#?}\n", frank);

  // Demonstrate that serialization goes to Map, not Record
  // (since Rust structs don't expose their type name at runtime)
  let frank_edn = to_edn(&frank)?;
  println!("Person serialized back to EDN (becomes Map, not Record):");
  println!("{}\n", frank_edn);

  // Also demonstrate special fields with records
  let special_record = Edn::Record(EdnRecordView {
    tag: EdnTag::new("SpecialPersonRecord"),
    pairs: vec![
      (EdnTag::new("name"), "Grace".into()),
      (EdnTag::new("age"), Edn::Number(29.0)),
      (EdnTag::new("first-name"), "Grace".into()),
      (EdnTag::new("last-name"), "Kim".into()),
      (EdnTag::new("is-active"), true.into()),
      (EdnTag::new("email-address"), "grace.kim@company.com".into()),
      (EdnTag::new("skill-level"), Edn::Number(7.0)),
    ],
  });

  println!("Special fields Record:");
  println!("{}\n", special_record);

  let grace: PersonWithSpecialFields = from_edn(special_record)?;
  println!("PersonWithSpecialFields from Record:");
  println!("{:#?}\n", grace);

  println!("✅ Record deserialization successful!\n");

  Ok(())
}
