use cirru_edn::Edn;

extern crate cirru_edn;

#[test]
fn atom_parse() {
  let atom = Edn::Atom(Box::new("test".into()));
  let formatted = cirru_edn::parse("atom |test");
  assert_eq!(Ok(atom), formatted);

  let atom = Edn::Atom(Box::new(Edn::List(vec![Edn::Number(1.), Edn::Number(2.)].into())));
  let formatted = cirru_edn::parse("atom $ [] 1 2");
  assert_eq!(Ok(atom), formatted);
}

#[test]
fn atom_format() -> Result<(), String> {
  let data = Edn::Atom(Box::new("test".into()));
  let formatted = cirru_edn::format(&data, true)?;
  assert_eq!(formatted, "\natom |test\n");

  let data = Edn::Atom(Box::new(Edn::List(vec![Edn::Number(1.), Edn::Number(2.)].into())));
  let formatted = cirru_edn::format(&data, true)?;
  assert_eq!(formatted, "\natom $ [] 1 2\n");

  Ok(())
}
