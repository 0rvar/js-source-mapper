extern crate quickcheck;

use self::quickcheck::quickcheck;

use base64_vlq;

#[test]
fn quickcheck_base64_vlq_converts_to_vql_and_back() {
  fn prop(x: i32) -> bool {
    base64_vlq::from_vql(base64_vlq::to_vql(x)) == x
  }
  quickcheck(prop as fn(i32) -> bool);
}

#[test]
fn quickcheck_base64_vlq_encodes_and_decodes_some_numbers() {
  fn prop(x: i32) -> bool {
    base64_vlq::encode(x).and_then(|x| base64_vlq::decode(&x)).unwrap().0 == x
  }
  quickcheck(prop as fn(i32) -> bool);
}
