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

#### Basic Usage

```rust
use cirru_edn::{to_edn, from_edn};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Person {
    name: String,
    age: u32,
    email: Option<String>,
    tags: Vec<String>,
}

let person = Person {
    name: "Alice".to_string(),
    age: 30,
    email: Some("alice@example.com".to_string()),
    tags: vec!["developer".to_string(), "rust".to_string()],
};

// Convert struct to Edn
let edn_value = to_edn(&person).unwrap();
println!("EDN: {}", edn_value);
// Output: ({} (|name |Alice) (|age 30) (|email "|alice@example.com") (|tags ([] |developer |rust)))

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

You can also manually construct Edn data and then deserialize it to structs:

```rust
use cirru_edn::{Edn, from_edn};
use std::collections::HashMap;

let edn_data = Edn::map_from_iter([
    ("name".into(), "Bob".into()),
    ("age".into(), Edn::Number(25.0)),
    ("email".into(), Edn::Nil),
    ("tags".into(), vec!["junior".to_string(), "javascript".to_string()].into()),
]);

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

#### Limitations

- Some special Edn types (like `Quote`, `AnyRef`) cannot be serialized
- Maps with complex keys will use their string representation when serializing structs

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

### License

MIT
