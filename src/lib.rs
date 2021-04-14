mod primes;

use cirru_parser::{parse, write_cirru};
use primes::CirruEdn;
use primes::CirruEdn::*;

/// TODO
pub fn parse_cirru_edn(s: String) -> CirruEdn {
  CirruEdnNil
}

/// TODO
pub fn write_cirru_edn(data: CirruEdn) -> String {
  String::from("TODO")
}
