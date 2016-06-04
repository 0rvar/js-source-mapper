static CHARACTER_MAP : &'static [u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/**
 * Encode an integer in the range of 0 to 63 to a single base 64 digit.
 */
pub fn encode(number: i32) -> Option<u8> {
  if 0 <= number && number < (CHARACTER_MAP.len() as i32) {
    return Some(CHARACTER_MAP[number as usize])
  }
  None
}

#[test]
fn it_encodes_some_numbers() {
  assert!(encode(0) == Some('A' as u8));
  assert!(encode(22) == Some('W' as u8));
  assert!(encode(42) == Some('q' as u8));
  assert!(encode(55) == Some('3' as u8));
  assert!(encode(63) == Some('/' as u8));
  assert!(encode(-1) == None);
  assert!(encode(65) == None);
}

/**
 * Decode a single base 64 character code digit to an integer.
 */
pub fn decode(char_code: u8) -> Option<i32> {
  // 0 - 25: ABCDEFGHIJKLMNOPQRSTUVWXYZ
  if ('A' as u8) <= char_code && char_code <= ('Z' as u8) {
    return Some((char_code - ('A' as u8)) as i32);
  }

  // 26 - 51: abcdefghijklmnopqrstuvwxyz
  if ('a' as u8) <= char_code && char_code <= ('z' as u8) {
    let lowercase_map_offset = 26;
    return Some((char_code - ('a' as u8) + lowercase_map_offset) as i32);
  }

  // 52 - 61: 0123456789
  if ('0' as u8) <= char_code && char_code <= ('9' as u8) {
    let number_map_offset = 52;
    return Some((char_code - ('0' as u8) + number_map_offset) as i32);
  }

  // 62: +
  if char_code == ('+' as u8) {
    return Some(62);
  }

  // 63: /
  if char_code == ('/' as u8) {
    return Some(63);
  }

  // Invalid base64 digit.
  None
}

#[test]
fn it_decodes_some_codepoints() {
  assert!(decode('A' as u8) == Some(0));
  assert!(decode('W' as u8) == Some(22));
  assert!(decode('q' as u8) == Some(42));
  assert!(decode('3' as u8) == Some(55));
  assert!(decode('/' as u8) == Some(63));
  assert!(decode('+' as u8) == Some(62));
  assert!(decode('.' as u8) == None);
  assert!(decode('ö' as u8) == None);
}

#[test]
fn it_encodes_and_decodes_some_numbers() {
  for x in 0..64 {
    assert!(encode(x).and_then(decode) == Some(x));
  }
  for &x in CHARACTER_MAP {
    assert!(decode(x).and_then(encode) == Some(x));
  }
}
