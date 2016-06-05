use std::cmp::Ordering;

use rustc_serialize::json;

use base64_vlq;

static SOURCE_MAP_VERSION: u32 = 3;

#[allow(dead_code)]
#[allow(non_snake_case)]
#[derive(RustcDecodable)]
struct SourceMap {
  version: u32,
  sources: Vec<String>,
  names: Vec<String>,
  sourceRoot: Option<String>,
  mappings: String,
  file: Option<String>

  // We skip this. Keeping megabytes of data that we do not care about
  // in memory seems reckless to caches.
  //sourcesContent: Option<vec<String>>,
}

#[derive(Eq, PartialEq, Debug)]
struct CodePosition {
  line: u32,
  column: u32
}

#[derive(Eq, PartialEq, Debug)]
pub struct Mapping {
  generated: CodePosition,
  original: CodePosition,
  source: String,
  name: String
}

pub struct Cache {
  generated_mappings: Vec<Mapping>,
  pub source_root: String
}

/*
 * consume parses a SourceMap into a cache that can be queried for mappings
 *
 * The only parameter is the raw source map as a JSON string.
 * According to the spec, source maps have the following attributes:
 *
 *   - version: Which version of the source map spec this map is following.
 *   - sources: An array of URLs to the original source files.
 *   - names: An array of identifiers which can be referrenced by individual mappings.
 *   - sourceRoot: Optional. The URL root from which all sources are relative.
 *   - sourcesContent: Optional. An array of contents of the original source files.
 *   - mappings: A string of base64 VLQs which contain the actual mappings.
 *   - file: Optional. The generated file this source map is associated with.
 *
 * Here is an example source map, taken from the source map spec[0]:
 *
 *     {
 *       version : 3,
 *       file: "out.js",
 *       sourceRoot : "",
 *       sources: ["foo.js", "bar.js"],
 *       names: ["src", "maps", "are", "fun"],
 *       mappings: "AA,AB;;ABCDE;"
 *     }
 *
 * [0]: https://docs.google.com/document/d/1U1RGAehQwRypUTovF1KRlpiOFze0b-_2gc6fAH0KY0k/edit?pli=1#
 */
pub fn consume(map: &str) -> Result<Cache, String> {
  let source_map: SourceMap = match json::decode(map) {
    Ok(x) => x,
    Err(err) => return Err(format!("{}", err))
  };

  parse_mappings(&source_map)
}

fn parse_mappings(source_map: &SourceMap) -> Result<Cache, String>{
  if source_map.version != SOURCE_MAP_VERSION {
    return Err("Only Source Map version 3 is implemented".into())
  }

  let mut generated_mappings: Vec<Mapping> = Vec::new();

  let mut generated_line: u32 = 0;
  let mut previous_original_line: u32 = 0;
  let mut previous_original_column: u32 = 0;
  let mut previous_source: u32 = 0;
  let mut previous_name: u32 = 0;

  for line in source_map.mappings.as_bytes().split(|&x| x == (';' as u8)) {
    generated_line += 1;
    let mut previous_generated_column: u32 = 0;

    for segment in line.split(|&x| x == (',' as u8)) {
      let segment_length = segment.len();
      let mut fields: Vec<i32> = Vec::new();
      let mut character_index = 0;
      while character_index < segment_length {
        match base64_vlq::decode(&segment[character_index..segment_length]) {
          Some((value, field_length)) => {
            fields.push(value);
            character_index += field_length;
          },
          None => return Err("Invalid VLQ mapping field".into())
        };
      }

      if fields.len() < 1 {
        continue;
      }

      if fields.len() == 2 {
        return Err("Found a source, but no line and column".into());
      }

      if fields.len() == 3 {
        return Err("Found a source and line, but no column".into());
      }

      let mut mapping = Mapping {
        generated: CodePosition {
          line: generated_line,
          column: ((previous_generated_column as i32) + fields[0]) as u32
        },
        original: CodePosition {
          line: 0,
          column: 0
        },
        source: "".into(),
        name: "".into()
      };

      previous_generated_column = mapping.generated.column;

      if fields.len() > 1 {
        // Original source.
        previous_source = ((previous_source as i32) + fields[1]) as u32;
        mapping.source = source_map.sources[previous_source as usize].to_owned();

        // Original line.
        previous_original_line = ((previous_original_line as i32) + fields[2]) as u32;
        // Lines are stored 0-based
        mapping.original.line = previous_original_line + 1;

        // Original column.
        previous_original_column = ((previous_original_column as i32) + fields[3]) as u32;
        mapping.original.column = previous_original_column;

        if fields.len() > 4 {
          // Original name.
          previous_name = ((previous_name as i32) + fields[4]) as u32;
          mapping.name = source_map.names[previous_name as usize].to_owned();
        }
      }

      generated_mappings.push(mapping);
    }
  }

  fn sort_key(mapping: &Mapping) -> (u32, u32) {
    (mapping.generated.line, mapping.generated.column)
  }
  generated_mappings.sort_by(|a, b| sort_key(a).cmp(&sort_key(b)));

  Ok(Cache {
    generated_mappings: generated_mappings,
    source_root: match &source_map.sourceRoot {
      &Some(ref x) => x.to_owned(),
      &None => "".into()
    }
  })
}


impl Cache {
  /*
   * Returns the original source, line, and column information for the generated
   * source's line and column positions provided. Arguments:
   *
   *   - line: The line number in the generated source.
   *   - column: The column number in the generated source.
   */

  pub fn mapping_for_generated_position(&self, line: u32, column: u32) -> &Mapping {
    let matcher = |mapping: &Mapping| -> Ordering {
      (mapping.generated.line, mapping.generated.column).cmp(&(line, column))
    };
    match self.generated_mappings.binary_search_by(matcher) {
      Ok(index) => &self.generated_mappings[index],
      Err(index) => &self.generated_mappings[index]
    }
  }
}

macro_rules! assert_equal_mappings(
  ($a:expr, $b:expr) => (
    if $a != &$b {
      panic!(format!("\n\n{:?}\n\n!=\n\n{:?}\n\n", $a, $b));
    }
  );
);

#[test]
fn test_source_map_issue_64() {
  let cache = consume(r#"{
    "version": 3,
    "file": "foo.js",
    "sourceRoot": "http://example.com/",
    "sources": ["/a"],
    "names": [],
    "mappings": "AACA",
    "sourcesContent": ["foo"]
  }"#).unwrap();

  let expected = Mapping {
    generated: CodePosition { line: 1, column: 0 },
    original: CodePosition { line: 2, column: 0 },
    source: "/a".into(),
    name: "".into()
  };
  let actual = cache.mapping_for_generated_position(1, 0);
  assert_equal_mappings!(actual, expected);
}

#[test]
fn test_source_map_issue_72_duplicate_sources() {
  let cache = consume(r#"{
    "version": 3,
    "file": "foo.js",
    "sources": ["source1.js", "source1.js", "source3.js"],
    "names": [],
    "mappings": ";EAAC;;IAEE;;MEEE",
    "sourceRoot": "http://example.com"
  }"#).unwrap();


  {
    let expected = Mapping {
      generated: CodePosition { line: 2, column: 2 },
      original: CodePosition { line: 1, column: 1 },
      source: "source1.js".into(),
      name: "".into()
    };
    let actual = cache.mapping_for_generated_position(2, 2);
    assert_equal_mappings!(actual, expected);
  }

  {
    let expected = Mapping {
      generated: CodePosition { line: 4, column: 4 },
      original: CodePosition { line: 3, column: 3 },
      source: "source1.js".into(),
      name: "".into()
    };
    let actual = cache.mapping_for_generated_position(4, 4);
    assert_equal_mappings!(actual, expected);
  }

  {
    let expected = Mapping {
      generated: CodePosition { line: 6, column: 6 },
      original: CodePosition { line: 5, column: 5 },
      source: "source3.js".into(),
      name: "".into()
    };
    let actual = cache.mapping_for_generated_position(6, 6);
    assert_equal_mappings!(actual, expected);
  }
}

#[test]
fn test_source_map_issue_72_duplicate_names() {
  let cache = consume(r#"{
    "version": 3,
    "file": "foo.js",
    "sources": ["source.js"],
    "names": ["name1", "name1", "name3"],
    "mappings": ";EAACA;;IAEEA;;MAEEE",
    "sourceRoot": "http://example.com"
  }"#).unwrap();

  {
    let expected = Mapping {
      generated: CodePosition { line: 2, column: 2 },
      original: CodePosition { line: 1, column: 1 },
      source: "source.js".into(),
      name: "name1".into()
    };
    let actual = cache.mapping_for_generated_position(2, 2);
    assert_equal_mappings!(actual, expected);
  }

  {
    let expected = Mapping {
      generated: CodePosition { line: 4, column: 4 },
      original: CodePosition { line: 3, column: 3 },
      source: "source.js".into(),
      name: "name1".into()
    };
    let actual = cache.mapping_for_generated_position(4, 4);
    assert_equal_mappings!(actual, expected);
  }

  {
    let expected = Mapping {
      generated: CodePosition { line: 6, column: 6 },
      original: CodePosition { line: 5, column: 5 },
      source: "source.js".into(),
      name: "name3".into()
    };
    let actual = cache.mapping_for_generated_position(6, 6);
    assert_equal_mappings!(actual, expected);
  }
}
