use base64;

// A single base 64 digit can contain 6 bits of data. For the base 64 variable
// length quantities we use in the source map spec, the first bit is the sign,
// the next four bits are the actual value, and the 6th bit is the
// continuation bit. The continuation bit tells us whether there are more
// digits in this value following this digit.
//
//   Continuation
//   |    Sign
//   |    |
//   V    V
//   101011

const VLQ_BASE_SHIFT: i32 = 5;

// binary: 100000
const VLQ_BASE: i32 = 1 << VLQ_BASE_SHIFT;

// binary: 011111
const VLQ_BASE_MASK: i32 = VLQ_BASE - 1;

// binary: 100000
const VLQ_CONTINUATION_BIT: i32 = VLQ_BASE;

#[test]
fn it_defines_sane_constants() {
  assert!(VLQ_BASE == 0b100000);
  assert!(VLQ_BASE_MASK == 0b011111);
  assert!(VLQ_CONTINUATION_BIT == 0b100000);
}

/**
 * Converts from a two-complement value to a value where the sign bit is
 * placed in the least significant bit.  For example, as decimals:
 *   1 becomes 2 (10 binary), -1 becomes 3 (11 binary)
 *   2 becomes 4 (100 binary), -2 becomes 5 (101 binary)
 */
pub fn to_vql(value: i32) -> i32 {
  if value < 0 {
    ((-value) << 1) + 1
  } else {
    (value << 1) + 0
  }
}

#[test]
fn it_converts_to_vql() {
  assert!(to_vql(1) == 2);
  assert!(to_vql(-1) == 3);
  assert!(to_vql(2) == 4);
  assert!(to_vql(-2) == 5);
}

/**
 * Converts to a two-complement value from a value where the sign bit is
 * placed in the least significant bit.  For example, as decimals:
 *   2 (10 binary) becomes 1, 3 (11 binary) becomes -1
 *   4 (100 binary) becomes 2, 5 (101 binary) becomes -2
 */
pub fn from_vql(value: i32) -> i32 {
  let is_neative = (value & 1) == 1;
  let shifted = value >> 1;
  if is_neative {
    -shifted
  } else {
    shifted
  }
}

#[test]
fn it_converts_from_vql() {
  assert!(from_vql(2) == 1);
  assert!(from_vql(3) == -1);
  assert!(from_vql(4) == 2);
  assert!(from_vql(5) == -2);
}

/**
 * Returns the base 64 VLQ encoded value.
 */
pub fn encode(value: i32) -> Option<String> {
  let mut encoded: Vec<u8> = Vec::new();
  let mut vlq = to_vql(value);

  loop {
    let mut digit = vlq & VLQ_BASE_MASK;
    vlq = vlq >> VLQ_BASE_SHIFT;
    if vlq > 0 {
      // There are still more digits in this value, so we must make sure the
      // continuation bit is marked.
      digit |= VLQ_CONTINUATION_BIT;
    }

    encoded.push(match base64::encode(digit) {
      Some(x) => x,
      None => return None
    });

    if vlq <= 0 {
      break;
    }
  };

  match String::from_utf8(encoded) {
    Ok(s) => Some(s),
    Err(_) => None
  }
}

macro_rules! assert_encodes_to(
  ($number:expr, $string:expr) => (
    assert!(encode($number) == Some($string.into()))
  );
);
#[test]
fn it_encodes_some_numbers() {
  assert_encodes_to!(-1000000, "hkh9B");
  assert_encodes_to!(-124133, "ruyH");
  assert_encodes_to!(-12332, "5iY");
  assert_encodes_to!(-2222, "9qE");
  assert_encodes_to!(-1579, "3iD");
  assert_encodes_to!(-65, "jE");
  assert_encodes_to!(-25, "zB");
  assert_encodes_to!(-20, "pB");
  assert_encodes_to!(-11, "X");
  assert_encodes_to!(-9, "T");
  assert_encodes_to!(-2, "F");
  assert_encodes_to!(-1, "D");
  assert_encodes_to!(0, "A");
  assert_encodes_to!(1, "C");
  assert_encodes_to!(7, "O");
  assert_encodes_to!(15, "e");
  assert_encodes_to!(23, "uB");
  assert_encodes_to!(88, "wF");
  assert_encodes_to!(1254, "suC");
  assert_encodes_to!(2493, "67E");
  assert_encodes_to!(23903, "+1uB");
  assert_encodes_to!(129383, "u28H");
  assert_encodes_to!(298322, "k1mS");
  assert_encodes_to!(1000000, "gkh9B");
}


/*
 * Decodes the next base 64 VLQ value from the given string and returns the
 * value and the rest of the string via the out parameter.
 */
pub fn decode(encoded: &str) -> Option<i32> {
  let mut result: i32  = 0;
  let mut shift: i32  = 0;

  for character in encoded.chars() {
    let mut digit = match base64::decode(character as u8) {
      Some(x) => x,
      None => return None
    };
    let continuation = (digit & VLQ_CONTINUATION_BIT) != 0;
    digit &= VLQ_BASE_MASK;
    result = result + (digit << shift);
    shift += VLQ_BASE_SHIFT;
    if !continuation {
      break;
    }
  }

  Some(from_vql(result))
}

macro_rules! assert_decodes_to(
  ($str_slice:expr, $number:expr) => (
    assert!(decode($str_slice) == Some($number))
  );
);
#[test]
fn it_decodes_some_numbers() {
  assert_decodes_to!("hkh9B", -1000000);
  assert_decodes_to!("ruyH", -124133);
  assert_decodes_to!("5iY", -12332);
  assert_decodes_to!("9qE", -2222);
  assert_decodes_to!("3iD", -1579);
  assert_decodes_to!("jE", -65);
  assert_decodes_to!("zB", -25);
  assert_decodes_to!("pB", -20);
  assert_decodes_to!("X", -11);
  assert_decodes_to!("T", -9);
  assert_decodes_to!("F", -2);
  assert_decodes_to!("D", -1);
  assert_decodes_to!("A", 0);
  assert_decodes_to!("C", 1);
  assert_decodes_to!("O", 7);
  assert_decodes_to!("e", 15);
  assert_decodes_to!("uB", 23);
  assert_decodes_to!("wF", 88);
  assert_decodes_to!("suC", 1254);
  assert_decodes_to!("67E", 2493);
  assert_decodes_to!("+1uB", 23903);
  assert_decodes_to!("u28H", 129383);
  assert_decodes_to!("k1mS", 298322);
  assert_decodes_to!("gkh9B", 1000000);
}
