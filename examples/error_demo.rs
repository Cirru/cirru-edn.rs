//! Example demonstrating error handling with path tracking and node previews
//!
//! This example shows two types of errors:
//! 1. Parse errors: Full Cirru parser errors with detailed position info
//! 2. Structure/Value errors: EDN structure errors with path coordinates and node previews

use cirru_edn::parse;

fn main() {
  println!("=== Cirru EDN Error Handling Demo ===\n");

  // Example 1: Parse error - mismatched brackets
  println!("1. Parse Error - Mismatched Brackets:");
  let invalid_cirru = "( ] mismatched";
  match parse(invalid_cirru) {
    Ok(_) => println!("  Unexpected success"),
    Err(e) => println!("{e}\n"),
  }

  // Example 2: Parse error - unclosed string
  println!("2. Parse Error - Unclosed String:");
  let unclosed_string = r#"[] |unclosed"#;
  match parse(unclosed_string) {
    Ok(_) => println!("  Unexpected success"),
    Err(e) => println!("{e}\n"),
  }

  // Example 3: Parse error - nested unclosed brackets
  println!("3. Parse Error - Nested Unclosed Brackets:");
  let nested_unclosed = "[] (a (b (c)";
  match parse(nested_unclosed) {
    Ok(_) => println!("  Unexpected success"),
    Err(e) => println!("{e}\n"),
  }

  // Example 4: Parse error - unexpected closing bracket
  println!("4. Parse Error - Unexpected Closing Bracket:");
  let unexpected_close = "[] a b c )";
  match parse(unexpected_close) {
    Ok(_) => println!("  Unexpected success"),
    Err(e) => println!("{e}\n"),
  }

  // Example 5: Parse error - invalid escape sequence
  println!("5. Parse Error - Invalid Escape in String:");
  let invalid_escape = r#"[] "\x invalid""#;
  match parse(invalid_escape) {
    Ok(_) => println!("  Unexpected success"),
    Err(e) => println!("{e}\n"),
  }

  // Example 6: Empty string value - shows path and node preview
  // Example 6: Empty string value - shows path and node preview
  println!("6. Empty String Value:");
  let empty_string = r#""""#;
  match parse(empty_string) {
    Ok(_) => println!("  Unexpected success"),
    Err(e) => println!("{e}\n"),
  }

  // Example 7: Invalid quote (missing value) - shows path and node preview
  println!("7. Invalid Quote (Missing Value):");
  let invalid_quote = "quote";
  match parse(invalid_quote) {
    Ok(_) => println!("  Unexpected success"),
    Err(e) => println!("{e}\n"),
  }

  // Example 8: Invalid map entry (not a pair) - shows path and node preview
  println!("8. Invalid Map Entry (Not a Pair):");
  let invalid_map = "{} :key";
  match parse(invalid_map) {
    Ok(_) => println!("  Unexpected success"),
    Err(e) => println!("{e}\n"),
  }

  // Example 9: Invalid record (empty) - shows path and node preview
  println!("9. Invalid Record (Empty):");
  let empty_record = "%{} :User";
  match parse(empty_record) {
    Ok(_) => println!("  Unexpected success"),
    Err(e) => println!("{e}\n"),
  }

  // Example 10: Invalid buffer value - shows path and node preview
  println!("10. Invalid Buffer Value:");
  let invalid_buffer = "buf xyz";
  match parse(invalid_buffer) {
    Ok(_) => println!("  Unexpected success"),
    Err(e) => println!("{e}\n"),
  }

  // Example 11: Invalid hex in buffer - shows path and node preview
  println!("11. Invalid Hex in Buffer:");
  let invalid_hex_buffer = "buf ff gg";
  match parse(invalid_hex_buffer) {
    Ok(_) => println!("  Unexpected success"),
    Err(e) => println!("{e}\n"),
  }

  // Example 12: Empty expression - shows path and node preview
  println!("12. Empty Expression:");
  let empty_expr = "()";
  match parse(empty_expr) {
    Ok(_) => println!("  Unexpected success"),
    Err(e) => println!("{e}\n"),
  }

  // Example 13: Invalid operator - shows path and node preview
  println!("13. Invalid Operator:");
  let invalid_op = "unknown-op 1 2";
  match parse(invalid_op) {
    Ok(_) => println!("  Unexpected success"),
    Err(e) => println!("{e}\n"),
  }

  // Example 14: Invalid record name (not a string) - shows path and node preview
  println!("14. Invalid Record Name (Not a String):");
  let invalid_record_name = "%{} (nested expr) (:field value)";
  match parse(invalid_record_name) {
    Ok(_) => println!("  Unexpected success"),
    Err(e) => println!("{e}\n"),
  }

  // Example 15: Nested error in structure - shows path tracking
  println!("15. Nested Error in List:");
  let nested_error = "[] 1 2 (invalid) 4";
  match parse(nested_error) {
    Ok(_) => println!("  Unexpected success"),
    Err(e) => println!("{e}\n"),
  }

  // Example 16: Deep nested error - demonstrates path tracking through multiple levels
  println!("16. Deep Nested Error (Shows Path Tracking):");
  let deep_nested = "[] 1 ({} (:key1 val1) (:key2 (invalid-op x))) 3";
  match parse(deep_nested) {
    Ok(_) => println!("  Unexpected success"),
    Err(e) => println!("{e}\n"),
  }

  // Example 17: Record with invalid value - shows nested path
  println!("17. Record with Invalid Value:");
  let invalid_record_value = "%{} :User (:name |John|) (:age (bad-value))";
  match parse(invalid_record_value) {
    Ok(_) => println!("  Unexpected success"),
    Err(e) => println!("{e}\n"),
  }

  println!("=== Demo Complete ===");
}
