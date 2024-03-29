use std::fs;
use std::io::Error;

const DEMO_INVALID: &str = r#"
{}
  :a $ {}
    :b $ {}
      :1 E
"#;

fn main() -> Result<(), Error> {
  // let large_file_path = "/Users/chen/repo/calcit-lang/runner.rs/src/cirru/calcit-core.cirru";
  let large_file_path = "/Users/chenyong/repo/cirru/hovenia-editor/calcit.cirru";
  let content = fs::read_to_string(large_file_path)?;
  let d = cirru_edn::parse(&content).unwrap();

  let t = cirru_edn::format(&d, true).unwrap();
  println!("{}", t.len());

  println!("{:?}", cirru_edn::parse(DEMO_INVALID));

  Ok(())
}
