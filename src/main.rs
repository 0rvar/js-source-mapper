extern crate rustc_serialize;

use std::fs::File;
use std::io::prelude::*;

mod base64;
mod base64_vlq;
mod consume;

use consume::{consume};

fn main() {
  let mut f = File::open("../bundle.min.js.map").unwrap();
  let mut s = String::new();
  f.read_to_string(&mut s).unwrap();
  consume(&s).unwrap();
}
