# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.2](https://github.com/conjure-cp/uniplate/compare/v0.4.1...v0.4.2) - 2025-08-05

### Added

- *(derive)* support tuples in derive macro

### Other

- *(deps)* bump criterion in the all group across 1 directory
- run cargo fmt
- split spez into different files
- update edition to 2024
- *(derive)* simplify types

## [0.4.1](https://github.com/conjure-cp/uniplate/compare/v0.4.0...v0.4.1) - 2025-08-04

### Added

- add impls for tuples

## [0.4.0](https://github.com/conjure-cp/uniplate/compare/v0.3.2...v0.4.0) - 2025-07-25

### Major Changes

* The `walk into` property has been removed from the derive macro. The types to
  walk into when deriving a biplate instance are now determined by whether the
  type implements biplate or not. That is, the derived instance `Biplate<T> for
  U` will walk into fields of type `V` looking for children of type `T` if `V`
  implements `Biplate<T>`.

  Remove any `walk_into` fields from any `#[biplate]` and `#[uniplate]`
  attributes.

* `Uniplate` and `Biplate` functions that used to take functions as `Arc<dyn
  Fn..>` objects now take a `&impl Fn()`.

  For example, `x.transform_bi(Arc::new(|x| x+1))` is now `x.transform_bi(&|x| x+1)`.

* The derive macro now supports enum structs and type parameters (lifetimes are
  not yet supported).


### Added

- *(derive)* [**breaking**] automatically determine types to walk into
- add specialization helpers for derive
- *(derive)* support simple generic type parameters
- [**breaking**] replace `Arc/Box` closures with `&impl Fn`
- *(derive)* support enum struct variants

### Other

- move traits into seperate files

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
