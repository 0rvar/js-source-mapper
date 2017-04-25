# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [0.2.0] - 2017-04-25
### Changed
* Switched to Serde for json parsing. Should not affect end users.
### Security
* Fuzzed project with cargo-fuzz, fixed several sources of panics.

[0.2.0]: https://github.com/awestroke/js-source-mapper/compare/v0.1.1...v0.2.0