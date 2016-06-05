extern crate rustc_serialize;


mod base64;
mod base64_vlq;
mod consume;

pub use consume::{Cache, consume};

#[cfg(test)]
mod test;
