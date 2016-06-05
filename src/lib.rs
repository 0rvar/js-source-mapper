extern crate rustc_serialize;


mod base64;
mod base64_vlq;
mod consume;

pub use consume::{Cache, Mapping, CodePosition, consume};

#[cfg(test)]
mod test;
