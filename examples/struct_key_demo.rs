//! # Tag vs String Key Example
//!
//! This example demonstrates the important distinction between Tags and Strings
//! in Cirru EDN's serde implementation.

use cirru_edn::{Edn, EdnMapView, EdnTag, from_edn, to_edn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct PersonalProfile {
  // These struct fields will become Tags in EDN (:first_name, :last_name, etc.)
  first_name: String,
  last_name: String,
  age: u32,
  is_active: bool,

  // This HashMap's keys will become Strings in EDN ("hobby", "skill", etc.)
  attributes: HashMap<String, String>,

  // Nested struct fields also become Tags
  contact: ContactInfo,

  // Vec elements don't have field names, so they remain as-is
  scores: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ContactInfo {
  // These will also be Tags (:email, :phone)
  email: Option<String>,
  phone: Option<String>,
}

#[allow(clippy::mutable_key_type)]
fn main() -> Result<(), String> {
  println!("=== Cirru EDN: Tag vs String Key Demonstration ===\n");

  // Create test data
  let profile = PersonalProfile {
    first_name: "Alice".to_string(),
    last_name: "Johnson".to_string(),
    age: 28,
    is_active: true,
    attributes: [
      ("hobby".to_string(), "photography".to_string()),
      ("skill".to_string(), "rust programming".to_string()),
      ("city".to_string(), "San Francisco".to_string()),
    ]
    .into_iter()
    .collect(),
    contact: ContactInfo {
      email: Some("alice.johnson@example.com".to_string()),
      phone: Some("+1-555-0123".to_string()),
    },
    scores: vec![95.5, 87.2, 92.0],
  };

  println!("Original Rust struct:");
  println!("{profile:#?}\n");

  // Serialize to EDN
  let edn_value = to_edn(&profile)?;
  println!("EDN representation:");
  println!("{edn_value}\n");

  // Analyze the key types
  if let Edn::Map(ref map) = edn_value {
    println!("Key type analysis:");
    for key in map.0.keys() {
      match key {
        Edn::Tag(tag) => {
          println!("  Tag (struct field): :{tag}");
        }
        Edn::Str(s) => {
          println!("  String (map key): \"{s}\"");
        }
        _ => {
          println!("  Other key type: {key}");
        }
      }
    }

    // Analyze nested attributes map
    if let Some(attributes_edn) = map.0.get(&Edn::Tag(EdnTag::new("attributes"))) {
      println!("\nNested 'attributes' map keys:");
      if let Edn::Map(attr_map) = attributes_edn {
        for key in attr_map.0.keys() {
          match key {
            Edn::Tag(tag) => {
              println!("  Tag: :{tag}");
            }
            Edn::Str(s) => {
              println!("  String: \"{s}\"");
            }
            _ => {
              println!("  Other: {key}");
            }
          }
        }
      }
    }

    // Analyze nested contact struct
    if let Some(contact_edn) = map.0.get(&Edn::Tag(EdnTag::new("contact"))) {
      println!("\nNested 'contact' struct fields:");
      if let Edn::Map(contact_map) = contact_edn {
        for key in contact_map.0.keys() {
          match key {
            Edn::Tag(tag) => {
              println!("  Tag (struct field): :{tag}");
            }
            Edn::Str(s) => {
              println!("  String: \"{s}\"");
            }
            _ => {
              println!("  Other: {key}");
            }
          }
        }
      }
    }
  }

  // Test round-trip conversion
  println!("\n=== Round-trip Test ===");
  let reconstructed: PersonalProfile = from_edn(edn_value)?;

  if profile == reconstructed {
    println!("✅ Round-trip conversion successful!");
    println!("   Original and reconstructed data are identical.");
  } else {
    println!("❌ Round-trip conversion failed!");
    return Err("Data integrity check failed".to_string());
  }

  // Demonstrate manual EDN construction with correct key types
  println!("\n=== Manual EDN Construction ===");
  let manual_edn = Edn::Map(EdnMapView({
    let mut map = HashMap::new();
    // Use Tags for struct fields
    map.insert(Edn::tag("first_name"), "Bob".into());
    map.insert(Edn::tag("last_name"), "Smith".into());
    map.insert(Edn::tag("age"), 35.into());
    map.insert(Edn::tag("is_active"), true.into());
    map.insert(Edn::tag("scores"), vec![88.0, 92.5, 85.0].into());

    // Use Strings for HashMap keys
    map.insert(Edn::tag("attributes"), {
      let mut attr_map = HashMap::new();
      attr_map.insert(Edn::Str("hobby".into()), Edn::Str("hiking".into()));
      attr_map.insert(Edn::Str("language".into()), Edn::Str("english".into()));
      Edn::Map(EdnMapView(attr_map))
    });

    // Nested struct with Tags for fields
    map.insert(Edn::Tag(EdnTag::new("contact")), {
      let mut contact_map = HashMap::new();
      contact_map.insert(Edn::tag("email"), "bob.smith@example.com".into());
      contact_map.insert(Edn::tag("phone"), Edn::Nil);
      Edn::Map(EdnMapView(contact_map))
    });

    map
  }));

  let manual_profile: PersonalProfile = from_edn(manual_edn)?;
  println!("Manually constructed profile:");
  println!("{manual_profile:#?}");

  println!("\n🎉 All demonstrations completed successfully!");
  println!("\nKey Takeaways:");
  println!("  • Struct fields → Tags (e.g., :first_name, :contact)");
  println!("  • HashMap keys → Strings (e.g., \"hobby\", \"skill\")");
  println!("  • This distinction preserves semantic meaning in EDN");

  Ok(())
}
