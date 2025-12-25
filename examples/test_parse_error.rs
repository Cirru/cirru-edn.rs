fn main() {
  let test = "[] a b c )";
  match cirru_parser::parse(test) {
    Ok(_) => println!("Success"),
    Err(e) => {
      println!("Error Display: {e}");
      println!("\nError format_detailed:");
      println!("{}", e.format_detailed(Some(test)));
    }
  }
}
