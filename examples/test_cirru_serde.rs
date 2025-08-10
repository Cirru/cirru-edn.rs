use cirru_edn::{from_edn, to_edn};
use cirru_parser::Cirru;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeEntry {
  pub doc: String,
  pub code: Cirru,
}

fn main() {
  let entry = CodeEntry {
    doc: "Hello world function".to_string(),
    code: Cirru::List(vec![
      Cirru::Leaf("println!".into()),
      Cirru::Leaf("\"Hello, world!\"".into()),
    ]),
  };

  println!("Original: {entry:#?}");

  // Test if Cirru can be serialized with our EDN functions
  match to_edn(&entry) {
    Ok(edn) => {
      println!("EDN serialization successful:");
      println!("{edn}");

      // Test deserialization
      match from_edn::<CodeEntry>(edn) {
        Ok(deserialized) => {
          println!("Deserialization successful!");
          println!("Equal: {}", entry == deserialized);
        }
        Err(e) => println!("Deserialization failed: {e}"),
      }
    }
    Err(e) => println!("Serialization failed: {e}"),
  }
}
