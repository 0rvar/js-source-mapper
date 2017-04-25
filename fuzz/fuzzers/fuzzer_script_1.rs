#![no_main]
#[macro_use] extern crate libfuzzer_sys;

extern crate js_source_mapper;

use js_source_mapper::{consume};

pub fn utf8_to_string(bytes: &[u8]) -> String {
    let vector: Vec<u8> = Vec::from(bytes);
    match String::from_utf8(vector) {
        Ok(s) => s,
        _ => "".into()
    }
}

fuzz_target!(|data: &[u8]| {
    let json = format!(r#"{{
        "version": 3,
        "file": "foo.js",
        "sources": ["source.js"],
        "names": ["name1", "name1", "name3"],
        "mappings": "{}",
        "sourceRoot": "http://example.com"
    }}"#, utf8_to_string(data));
    match consume(&json) {
        Ok(cache) => {
            cache.mapping_for_generated_position(2, 2)
        },
        _ => return
    };
});
