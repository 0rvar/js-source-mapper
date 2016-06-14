# js-source-mapper

A rust library for consuming JavaScript source maps with a focus on performance.
Supports [Source Map revision 3](https://docs.google.com/document/d/1U1RGAehQwRypUTovF1KRlpiOFze0b-_2gc6fAH0KY0k/edit).

[![Build Status](https://travis-ci.org/awestroke/js-source-mapper.svg?branch=master)](https://travis-ci.org/awestroke/js-source-mapper)
[![Build status](https://ci.appveyor.com/api/projects/status/0biffgxl3p49ici3?svg=true)](https://ci.appveyor.com/project/awestroke/js-source-mapper)
[![Coverage Status](https://coveralls.io/repos/github/awestroke/js-source-mapper/badge.svg?branch=master)](https://coveralls.io/github/awestroke/js-source-mapper?branch=master)
[![Crates.io](https://img.shields.io/crates/v/js-source-mapper.svg)](https://crates.io/crates/js-source-mapper)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

[Documentation](http://awestroke.github.io/js-source-mapper)

## Usage

```toml
[dependencies]
js-source-mapper = "0.1.1"
```

```rust
extern crate js_source_mapper;

use js_source_mapper::{Cache, consume};

fn main() {
  let cache = consume(r#"{
    "version": 3,
    "file": "foo.js",
    "sources": ["source.js"],
    "names": ["name1", "name1", "name3"],
    "mappings": ";EAACA;;IAEEA;;MAEEE",
    "sourceRoot": "http://example.com"
  }"#).unwrap();

  let mapping = cache.mapping_for_generated_position(2, 2);
  assert!(mapping.original.line == 1);
  assert!(mapping.original.column == 1);
  assert!(mapping.source == "source.js".into());
  assert!(mapping.name == "name1".into());
}
```
