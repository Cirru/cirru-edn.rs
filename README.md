## Cirru Edn in Rust

### Usages

```bash
cargo add cirru_edn
```

```rust
use cirru_edn::Edn;

cirru_edn::parse("[] 1 2 true"); // Result<Edn, String>

cirru_edn::format(data, /* use_inline */ true); // Result<String, String>.
```

### License

MIT
