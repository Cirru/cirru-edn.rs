use cirru_edn::{format, parse, Edn};
use std::fs;

fn main() -> Result<(), String> {
  let large_demo = "/Users/chenyong/repo/calcit-lang/editor/compact.cirru";
  let content = fs::read_to_string(large_demo).unwrap();

  let v = parse(&content)?;

  let buf = bincode::encode_to_vec(&v, bincode::config::standard()).map_err(|e| e.to_string())?;

  let bin_out = "target/bincode/calcit-info.bin";

  fs::write(bin_out, &buf).map_err(|e| e.to_string())?;

  let (decoded, _length): (Edn, usize) = bincode::decode_from_slice(&buf[..], bincode::config::standard()).unwrap();

  println!("wrote to {}", bin_out);

  println!("{}", format(&decoded, true).unwrap());

  Ok(())
}
