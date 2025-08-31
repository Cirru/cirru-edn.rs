//! # API Documentation Demo
//!
//! This example demonstrates how to parse Cirru EDN data into a structured ApiDoc format.
//! It shows how to deserialize complex nested data including sets, vectors, and quoted Cirru code.

use cirru_edn::{Edn, parse};
use cirru_parser::Cirru;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashSet;

// Custom deserializer for tag set
fn deserialize_tag_set<'de, D>(deserializer: D) -> Result<HashSet<String>, D::Error>
where
  D: Deserializer<'de>,
{
  // Since Set is now converted to a sequence in the library, we expect a Vec<String>
  let vec: Vec<String> = Vec::deserialize(deserializer)?;
  Ok(vec.into_iter().collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiDoc {
  name: String,
  desc: String,
  #[serde(deserialize_with = "deserialize_tag_set")]
  tags: HashSet<String>,
  snippets: Vec<Cirru>,
}

fn main() -> Result<(), String> {
  println!("API Documentation Demo");
  println!("======================\n");

  // The original Cirru EDN data
  let cirru_edn_text = r#"
[]
  {}
    :name |defn
    :tags $ #{} :syntax
    :desc "|create functions on namespaces"
    :snippets $ []
      quote $ defn (a b) $ + a b
      quote $ defn (a $ xs)
        echo a xs
"#;

  println!("Original Cirru EDN:");
  println!("{cirru_edn_text}");
  println!();

  // Parse the Cirru EDN
  let parsed_edn: Edn = parse(cirru_edn_text).map_err(|e| format!("Parse error: {e}"))?;
  println!("Parsed EDN structure:");
  println!("{parsed_edn:#?}");
  println!();

  // Extract the first element from the vector (which contains our API doc)
  let api_doc_edn = match &parsed_edn {
    Edn::List(list) if !list.is_empty() => list.get(0).ok_or("List is empty")?,
    _ => return Err("Expected a non-empty list".to_string()),
  };

  // Convert to ApiDoc struct using from_edn
  let api_doc: ApiDoc = cirru_edn::from_edn(api_doc_edn.clone()).map_err(|e| format!("Deserialization error: {e}"))?;

  println!("Deserialized ApiDoc:");
  println!("{api_doc:#?}");
  println!();

  // Demonstrate the parsed data
  println!("API Documentation Details:");
  println!("Name: {}", api_doc.name);
  println!("Description: {}", api_doc.desc);
  println!("Tags: {:?}", api_doc.tags);
  println!("Number of snippets: {}", api_doc.snippets.len());
  println!();

  // Show each snippet
  for (i, snippet) in api_doc.snippets.iter().enumerate() {
    println!("Snippet {}:", i + 1);
    println!("{snippet:#?}");
    println!();
  }

  // Demonstrate round-trip serialization
  println!("Round-trip test:");
  let serialized = cirru_edn::to_edn(&api_doc).map_err(|e| format!("Serialization error: {e}"))?;
  println!("Serialized back to EDN:");
  println!("{serialized}");
  println!();

  let deserialized: ApiDoc =
    cirru_edn::from_edn(serialized).map_err(|e| format!("Round-trip deserialization error: {e}"))?;

  if api_doc.name == deserialized.name
    && api_doc.desc == deserialized.desc
    && api_doc.tags == deserialized.tags
    && api_doc.snippets == deserialized.snippets
  {
    println!("✅ Round-trip serialization successful!");
  } else {
    println!("❌ Round-trip serialization failed!");
    return Err("Round-trip test failed".to_string());
  }

  // Demonstrate creating ApiDoc programmatically
  println!("\nCreating ApiDoc programmatically:");
  let programmatic_doc = ApiDoc {
    name: "map".to_string(),
    desc: "apply function to each element in a collection".to_string(),
    tags: ["functional".to_string(), "collection".to_string()]
      .into_iter()
      .collect(),
    snippets: vec![
      Cirru::List(vec![
        Cirru::Leaf("map".into()),
        Cirru::Leaf("inc".into()),
        Cirru::List(vec![Cirru::Leaf("range".into()), Cirru::Leaf("10".into())]),
      ]),
      Cirru::List(vec![
        Cirru::Leaf("map".into()),
        Cirru::List(vec![
          Cirru::Leaf("fn".into()),
          Cirru::List(vec![Cirru::Leaf("x".into())]),
          Cirru::List(vec![
            Cirru::Leaf("*".into()),
            Cirru::Leaf("x".into()),
            Cirru::Leaf("x".into()),
          ]),
        ]),
        Cirru::List(vec![Cirru::Leaf("range".into()), Cirru::Leaf("5".into())]),
      ]),
    ],
  };

  println!("{programmatic_doc:#?}");
  println!();

  let prog_serialized =
    cirru_edn::to_edn(&programmatic_doc).map_err(|e| format!("Programmatic serialization error: {e}"))?;
  println!("Programmatic ApiDoc serialized to EDN:");
  println!("{prog_serialized}");

  println!("\n✅ API Documentation demo completed successfully!");
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_api_doc_deserialization() {
    let cirru_edn_text = r#"
[]
  {}
    :name |defn
    :tags $ #{} :syntax
    :desc "|create functions on namespaces"
    :snippets $ []
      quote $ defn (a b) $ + a b
      quote $ defn (a $ xs)
        echo a xs
"#;

    let parsed_edn: Edn = parse(cirru_edn_text).unwrap();
    let api_doc_edn = match &parsed_edn {
      Edn::List(list) if !list.is_empty() => list.get(0).unwrap(),
      _ => panic!("Expected a non-empty list"),
    };

    let api_doc: ApiDoc = cirru_edn::from_edn(api_doc_edn.clone()).unwrap();

    assert_eq!(api_doc.name, "defn");
    assert_eq!(api_doc.desc, "create functions on namespaces");
    assert_eq!(api_doc.tags, ["syntax".to_string()].into_iter().collect());
    assert_eq!(api_doc.snippets.len(), 2);
  }

  #[test]
  fn test_api_doc_round_trip() {
    let original = ApiDoc {
      name: "test".to_string(),
      desc: "test function".to_string(),
      tags: ["testing".to_string()].into_iter().collect(),
      snippets: vec![Cirru::List(vec![
        Cirru::Leaf("assert".into()),
        Cirru::Leaf("true".into()),
      ])],
    };

    let serialized = cirru_edn::to_edn(&original).unwrap();
    let deserialized: ApiDoc = cirru_edn::from_edn(serialized).unwrap();

    assert_eq!(original.name, deserialized.name);
    assert_eq!(original.desc, deserialized.desc);
    assert_eq!(original.tags, deserialized.tags);
    assert_eq!(original.snippets, deserialized.snippets);
  }
}
