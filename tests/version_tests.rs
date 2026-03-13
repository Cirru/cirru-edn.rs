use cirru_edn::version;

#[test]
fn test_version() {
  assert_eq!(version(), env!("CARGO_PKG_VERSION"));
}
