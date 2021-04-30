use std::fs;
use std::io::Error;

fn main() -> Result<(), Error> {
  // let large_file_path = "/Users/chen/repo/calcit-lang/runner.rs/src/cirru/calcit-core.cirru";
  let large_file_path = "/Users/chen/repo/cirru/calcit-editor/calcit.cirru";
  let content = fs::read_to_string(large_file_path)?;
  let d = cirru_edn::parse(&content).unwrap();

  println!("{}", cirru_edn::format(&d, true).unwrap());

  Ok(())
}
