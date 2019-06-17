# RIS

[![https://travis-ci.com/Palladinium/ris-rs.svg?branch=master]](https://travis-ci.com/Palladinium/ris-rs)
[![Crate](https://img.shields.io/crates/v/ris.svg)](https://crates.io/crates/ris)
[![Docs](https://docs.rs/ris/badge.svg)](https://docs.rs/ris)

A simple [https://en.wikipedia.org/wiki/RIS_(file_format)](RIS bibliography file) (de)serializer for Rust.

# Features

- [ ] Deserialization
  - [x] From `&str`
  - [ ] From `Read`
- [x] Serialization
  - [x] To `String`
  - [x] To `Write`
- [ ] Extensive test coverage
- [ ] Tested on bibliography managers
  - [x] Mendeley (1.19.5)
  - [ ] Zotero
  - [ ] Citavi
  - [ ] EndNote

# Contributing

PRs and issues are welcome!

Please ensure the following before submitting a PR:
- New features have some test coverage
- All tests pass
- No compiler warnings
- Formatted according to `rustfmt`

# License

Waddle is licensed under the [MIT License](https://opensource.org/licenses/MIT).
