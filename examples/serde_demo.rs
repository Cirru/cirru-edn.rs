//! # Serde Demo
//!
//! This example demonstrates basic serde integration with Cirru EDN,
//! including serialization, deserialization, error handling, and manual EDN construction.

#![allow(clippy::mutable_key_type)]
#![allow(clippy::uninlined_format_args)]

use cirru_edn::{from_edn, to_edn, Edn};
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

  // Demonstrate manual Edn construction
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

  // Demonstrate error handling
  println!("5. Error handling example...");
  let incomplete_edn = Edn::map_from_iter([
    (Edn::tag("name"), "Invalid".into()),
    // Missing required fields
  ]);

  match from_edn::<Person>(incomplete_edn) {
    Ok(_) => println!("Unexpected success"),
    Err(e) => println!("Expected error: {}\n", e),
  }

  // Demonstrate special field names with serde rename
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

  // Manual construction with special field names
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

  println!("🎉 All demonstrations completed successfully!");
  Ok(())
}
