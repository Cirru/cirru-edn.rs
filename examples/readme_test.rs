#![allow(clippy::mutable_key_type)]
#![allow(clippy::uninlined_format_args)]

#[cfg(feature = "serde")]
use cirru_edn::{from_edn, to_edn, Edn};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "serde")]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Person {
  name: String,
  age: u32,
  email: Option<String>,
  tags: Vec<String>,
}

#[cfg(feature = "serde")]
fn main() -> Result<(), String> {
  // Test the README examples

  // Basic usage example
  let person = Person {
    name: "Alice".to_string(),
    age: 30,
    email: Some("alice@example.com".to_string()),
    tags: vec!["developer".to_string(), "rust".to_string()],
  };

  // Convert struct to Edn
  let edn_value = to_edn(&person)?;
  println!("EDN: {}", edn_value);

  // Convert Edn back to struct
  let reconstructed: Person = from_edn(edn_value)?;
  assert_eq!(person, reconstructed);

  // Manual Edn construction example
  let edn_data = Edn::map_from_iter([
    ("name".into(), "Bob".into()),
    ("age".into(), Edn::Number(25.0)),
    ("email".into(), Edn::Nil),
    (
      "tags".into(),
      vec!["junior".to_string(), "javascript".to_string()].into(),
    ),
  ]);

  let person2: Person = from_edn(edn_data)?;
  println!("Person from manual EDN: {:?}", person2);

  // Error handling example
  let incomplete_edn = Edn::map_from_iter([
    ("name".into(), "Invalid".into()),
    // Missing required age field
  ]);

  match from_edn::<Person>(incomplete_edn) {
    Ok(person) => println!("Success: {:?}", person),
    Err(e) => println!("Expected error: {}", e),
  }

  println!("All README examples work correctly!");
  Ok(())
}

#[cfg(not(feature = "serde"))]
fn main() {
  println!("This example requires the 'serde' feature to be enabled.");
  println!("Run with: cargo run --example readme_test --features serde");
}
