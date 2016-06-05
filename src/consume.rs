use std::cmp::Ordering;
use std::error::Error;

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
  mappings: Vec<u8>,
  file: Option<String>

  // We skip this. Keeping megabytes of data that we do not care about
  // in memory seems reckless to caches.
  //sourcesContent: Option<vec<String>>,
}

#[derive(Eq, PartialEq)]
struct CodePosition {
  line: u32,
  column: u32
}

#[derive(Eq)]
struct Mapping {
  generated: CodePosition,
  original: CodePosition,
  source: String,
  name: String
}

impl PartialEq for Mapping {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.generated == other.generated
  }
}

impl Ord for Mapping {
  fn cmp(&self, other: &Self) -> Ordering {
    (self.generated.line, &self.generated.column)
      .cmp(&(other.generated.line, &other.generated.column))
  }
}

impl PartialOrd for Mapping {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some((self.generated.line, &self.generated.column)
      .cmp(&(other.generated.line, &other.generated.column)))
  }
}

pub struct Cache {
  generated_mappings: Vec<Mapping>
}

/*
 * consume parses a SourceMap into a cache that can be queries for mappings
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
    Err(err) => return Err(err.description().into())
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

  for line in source_map.mappings.split(|&x| x == (';' as u8)) {
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
        mapping.source = source_map.names[previous_source as usize].to_owned();

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

  generated_mappings.sort();
  Ok(Cache {
    generated_mappings: generated_mappings
  })
}

/*
 * Returns the original source, line, and column information for the generated
 * source's line and column positions provided. The only argument is an object
 * with the following properties:
 *
 *   - line: The line number in the generated source.
 *   - column: The column number in the generated source.
 *   - bias: Either 'SourceMapConsumer.GREATEST_LOWER_BOUND' or
 *     'SourceMapConsumer.LEAST_UPPER_BOUND'. Specifies whether to return the
 *     closest element that is smaller than or greater than the one we are
 *     searching for, respectively, if the exact element cannot be found.
 *     Defaults to 'SourceMapConsumer.GREATEST_LOWER_BOUND'.
 *
 * and an object is returned with the following properties:
 *
 *   - source: The original source file, or null.
 *   - line: The line number in the original source, or null.
 *   - column: The column number in the original source, or null.
 *   - name: The original identifier, or null.
 */

//fn originalPositionFor(line: u32, column: u32) -> CodePosition {
  // binary search
//}
