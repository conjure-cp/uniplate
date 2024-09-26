1. Run `cargo update`
2. Update `Cargo.toml` version fields to the new version.
3. Update the version of `uniplate-derive` that `uniplate` depends on to the new version.
4. Increment version numbers in the docs.rs and crates.io links at the top of README.md
4. Check documentation build.
5. Create a git tag with the version number.
7. Run `cargo publish --dry-run` and `cargo publish` for `uniplate-derive`.
8. Run `cargo publish --dry-run` and `cargo publish` for`uniplate`.
9. Create a Github release, automatically generating the changelog.
