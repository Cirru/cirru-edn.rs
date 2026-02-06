## Cirru Edn in Rust

> Extensible data notations based on Cirru syntax

### Usages

![](https://img.shields.io/crates/v/cirru_edn?style=flat-square)

[Rust Docs](https://docs.rs/crate/cirru_edn/).

```bash
cargo add cirru_edn
```

Basic parsing and formatting:

```rust
use cirru_edn::Edn;

cirru_edn::parse("[] 1 2 true"); // Result<Edn, String>

cirru_edn::format(data, /* use_inline */ true); // Result<String, String>.
```

### Serde Integration

Cirru EDN provides seamless integration with serde, allowing you to easily convert between Rust structs and EDN data with efficient direct serialization and deserialization.

#### Key Type Distinction

An important feature of this implementation is the semantic distinction between struct fields and map keys:

- **Struct fields** use `Tag` (`:field_name`) - representing named constants and structured identifiers
- **Map keys** use `String` (`"key"`) - representing arbitrary string data

This design preserves the intended meaning of different data elements in EDN format.

#### Basic Usage

```rust
use cirru_edn::{to_edn, from_edn};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Person {
    name: String,
    age: u32,
    email: Option<String>,
    tags: Vec<String>,
    metadata: HashMap<String, String>,  // Map keys will be Strings
}

let person = Person {
    name: "Alice".to_string(),
    age: 30,
    email: Some("alice@example.com".to_string()),
    tags: vec!["developer".to_string(), "rust".to_string()],
    metadata: [("role".to_string(), "senior".to_string())].into_iter().collect(),
};

// Convert struct to Edn
let edn_value = to_edn(&person).unwrap();
println!("EDN: {}", edn_value);
// Output: {:name "Alice", :age 30, :email "alice@example.com", :tags ["developer", "rust"], :metadata {"role" "senior"}}
//          ^^^^^ Tag                                                                                    ^^^^^^ String

// Convert Edn back to struct
let reconstructed: Person = from_edn(edn_value).unwrap();
assert_eq!(person, reconstructed);
```

#### Supported Data Types

- **Primitive types**: `bool`, `i32`, `i64`, `u32`, `u64`, `f32`, `f64`, `String`
- **Container types**: `Vec<T>`, `HashMap<K, V>`, `HashSet<T>`
- **Optional types**: `Option<T>` (maps to `Edn::Nil` or the actual value)
- **Nested structures**: Arbitrarily deep nested structs

#### Manual Edn Construction

You can also manually construct Edn data and then deserialize it to structs. Remember to use Tags for struct field keys:

```rust
use cirru_edn::{Edn, EdnTag, EdnMapView, from_edn};
use std::collections::HashMap;

// Construct EDN manually with proper key types
let mut map = HashMap::new();
map.insert(Edn::Tag(EdnTag::new("name")), "Bob".into());         // Tag for struct field
map.insert(Edn::Tag(EdnTag::new("age")), Edn::Number(25.0));     // Tag for struct field
map.insert(Edn::Tag(EdnTag::new("email")), Edn::Nil);            // Tag for struct field
map.insert(Edn::Tag(EdnTag::new("tags")), vec!["junior".to_string(), "javascript".to_string()].into());

// For metadata HashMap, use String keys
let mut metadata_map = HashMap::new();
metadata_map.insert(Edn::Str("department".into()), Edn::Str("engineering".into()));  // String for map key
map.insert(Edn::Tag(EdnTag::new("metadata")), Edn::Map(EdnMapView(metadata_map)));

let edn_data = Edn::Map(EdnMapView(map));
let person: Person = from_edn(edn_data).unwrap();
println!("{:?}", person);
```

#### Error Handling

When deserialization fails (e.g., missing required fields or type mismatches), descriptive error messages are returned:

```rust
let incomplete_edn = Edn::map_from_iter([
    ("name".into(), "Invalid".into()),
    // Missing required age field
]);

match from_edn::<Person>(incomplete_edn) {
    Ok(person) => println!("Success: {:?}", person),
    Err(e) => println!("Error: {}", e), // Error: missing field `age`
}
```

#### Complex Examples

See `examples/serde_demo.rs` for more complex nested structures and usage patterns.

#### Record Deserialization

Cirru EDN supports Record types with named tags, which can be deserialized to Rust structs. During deserialization, the record name is ignored since Rust structs don't expose their type names at runtime:

```rust
use cirru_edn::{Edn, EdnRecordView, EdnTag, from_edn};

// Create a Record with a named type
let person_record = Edn::Record(EdnRecordView {
    tag: EdnTag::new("PersonRecord"),  // This name will be ignored during deserialization
    pairs: vec![
        (EdnTag::new("name"), "Frank".into()),
        (EdnTag::new("age"), Edn::Number(42.0)),
        (EdnTag::new("email"), "frank@example.com".into()),
    ],
});

// Deserialize Record to struct (ignoring the record name)
let person: Person = from_edn(person_record).unwrap();
println!("{:?}", person);

// Note: When serializing structs back to EDN, they become Maps, not Records
// since Rust doesn't provide struct names at runtime
let edn_back = to_edn(&person).unwrap();
// This will be a Map, not a Record
```

This feature allows interoperability between EDN data containing Records and Rust structs, with the semantic understanding that record names are metadata that may be lost during round-trip conversion.

#### Limitations

- Some special Edn types (like `Quote`, `AnyRef`) cannot be serialized
- Maps with complex keys will use their string representation when serializing structs
- Record names are ignored during deserialization and structs serialize to Maps, not Records

### EDN Format

mixed data:

```cirru
{} (:a 1.0)
  :b $ [] 2.0 3.0 4.0
  :c $ {} (:d 4.0)
    :e true
    :f :g
    :h $ {} (|a 1.0)
      |b true
```

```cirru
{}
  :b $ [] 2 3 4
  :a 1
  :c $ {}
    :h $ {} (|b true) (|a 1)
    :f :g
    :e true
    :d 4
```

for top-level literals, need to use `do` expression:

```cirru
do nil
```

```cirru
do true
do false
```

```cirru
do 1
do -1.1
```

quoted code:

```cirru
do 'a
quote (a b)
```

tags(previously called "keyword")

```cirru
do :a
```

string syntax, note it's using prefixed syntax of `|`:

```cirru
do |a
```

string with special characters:

```cirru
do \"|a b\"
```

nested list:

```cirru
[] 1 2 $ [] 3
```

```cirru
#{} ([] 3) 1
```

tuple, or tagged union, actually very limitted due to Calcit semantics:

```cirru
:: :a

:: :b 1
```

extra values can be added to tuple since `0.3`:

```cirru
:: :a 1 |extra :l
```

newly added `%::` for representing enums with a type tag:

```cirru
%:: :e :a 1 |extra :l
```

a record, notice that now it's all using tags:

```cirru
%{} :Demo (:a 1)
  :b 2
  :c $ [] 1 2 3
```

extra format for holding buffer, which is internally `Vec<u8>`:

```cirru
buf 00 01 f1 11
```

atom, which translates to a reference to a value:

```cirru
atom 1
```

### Error Handling (v0.7.0+)

With cirru_parser 0.2.0, Cirru EDN provides enhanced error reporting with position information:

```rust
use cirru_edn::{parse, EdnError};

match parse("[] 1 2 invalid") {
    Ok(data) => println!("Parsed: {:?}", data),
    Err(EdnError::ValueError { message, position }) => {
        println!("Error: {}", message);
        if let Some(pos) = position {
            println!("  at line {}, column {}, byte {}",
                     pos.line, pos.column, pos.offset);
        }
    }
    Err(e) => println!("Other error: {}", e),
}
```

Error types include:

- `ParseError` - Syntax errors from the parser
- `StructureError` - Invalid EDN structure
- `ValueError` - Invalid values (e.g., bad hex, invalid tokens)
- `DeserializationError` - Serde deserialization errors

See [ERROR_HANDLING.md](ERROR_HANDLING.md) for detailed documentation and examples.

Run the error demo:

```bash
cargo run --example error_demo
```

### License

MIT
