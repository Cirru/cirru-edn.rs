use cirru_edn::{Edn, parse};

pub fn main() {
  println!("{}", Edn::Buffer(vec![1, 2, 3]));

  println!("{:?}", parse("buf 03 55 77 ff 00"));
}
