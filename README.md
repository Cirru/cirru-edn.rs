## Cirru Edn in Rust

### Usages

```bash
cargo add cirru_edn
```

```rs
use cirru_edn::{parse_cirru_edn, write_cirru_edn, CirruEdn};

parse_cirru_edn(String::from("[] 1 2 true")); // Result<CirruEdn, String>

write_cirru_edn(data); // String
```

### License

MIT
