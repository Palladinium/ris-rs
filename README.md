# RIS

[Build Status]: https://travis-ci.com/Palladinium/ris-rs.svg?branch=master
[travis]: https://travis-ci.com/Palladinium/ris-rs

[![Build Status]][travis]
[![Crate](https://img.shields.io/crates/v/ris.svg)](https://crates.io/crates/ris)
[![Docs](https://docs.rs/ris/badge.svg)](https://docs.rs/ris)

A simple [RIS bibliography file](https://en.wikipedia.org/wiki/RIS_%28file_format%29) (de)serializer for Rust.

# Features

- [ ] Deserialization
  - [x] From `&str`
  - [ ] From `Read`
- [x] Serialization
  - [x] To `String`
  - [x] To `Write`
- [ ] Extensive test coverage
- [ ] Tested on bibliography managers
  - [ ] Mendeley
  - [ ] Zotero
  - [ ] Citavi
  - [ ] EndNote

# Contributing

PRs and issues are welcome!

Please ensure all of the following for PRs, or mark them as WIP:
- New features have some test coverage
- All `pub` functions and structs are documented
- All tests pass
- No compiler warnings
- Formatted according to `rustfmt`

# License

ris-rs is licensed under the [MIT License](https://opensource.org/licenses/MIT).
