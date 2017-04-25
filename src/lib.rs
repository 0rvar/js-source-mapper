mod base64;
mod base64_vlq;
mod consume;

#[macro_use] extern crate serde_derive;

pub use consume::{Cache, Mapping, CodePosition, consume};

#[cfg(test)]
mod test;
