# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0](https://github.com/conjure-cp/uniplate/compare/v0.3.2...v0.4.0) - 2025-07-25

### Added

- *(derive)* [**breaking**] automatically determine types to walk into
- add specialization helpers for derive
- *(derive)* support simple generic type parameters
- [**breaking**] replace `Arc/Box` closures with `&impl Fn`
- *(derive)* support enum struct variants

### Other

- move traits into seperate files
- Update uniplate-derive/src/ast/derive_input.rs

## [0.3.2](https://github.com/conjure-cp/uniplate/compare/v0.3.1...v0.3.2) - 2025-07-21

### Added

- add Zipper sibling and ancestor iterators

## [0.3.1](https://github.com/conjure-cp/uniplate/compare/v0.3.0...v0.3.1) - 2025-07-21

### Other

- remove unused dependencies
- *(release)* make readme links always point to latest version
- *(release)* update documentation links to 0.3.0

## [0.3.0](https://github.com/conjure-cp/uniplate/compare/v0.2.3...v0.3.0) - 2025-07-20

### Added

- [**breaking**] move derive macro from `uniplate::derive::Uniplate` to `uniplate::Uniplate`
- unhide `Tree` type
- add `uniplate::tagged_zipper::TaggedZipper`

### Other

- *(deps)* bump the all group across 1 directory with 9 updates
