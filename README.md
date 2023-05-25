## Cirru Edn in Rust

> Extensible data notations based on Cirru syntax

### Usages

![](https://img.shields.io/crates/v/cirru_edn?style=flat-square)

[Rust Docs](https://docs.rs/crate/cirru_edn/).

```bash
cargo add cirru_edn
```

```rust
use cirru_edn::Edn;

cirru_edn::parse("[] 1 2 true"); // Result<Edn, String>

cirru_edn::format(data, /* use_inline */ true); // Result<String, String>.
```

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

a record, notice that now it's all using keywords:

```cirru
%{} :Demo (:a 1)
  :b 2
  :c $ [] 1 2 3
```

extra format for holding buffer, which is internally `Vec<u8>`:

```cirru
buf 00 01 f1 11
```

### License

MIT
