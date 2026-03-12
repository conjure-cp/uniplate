# Contributing

Thank you for your interest in contributing to Uniplate!

Here are some important resources:

- [Issue tracker](https://github.com/conjure-cp/uniplate/issues)
- [Maintainer Information](MAINTAIN.md)

Uniplate is part of the wider [conjure-oxide project](https://github.com/conjure-cp/conjure-oxide/).

### Submitting Changes

Changes to Uniplate can be submitted as Github pull requests. Changes will be
reviewed by a committer before merging, using the guidelines below.

Please specify if you wish your PR to be squash-merged. If a PR is to be squash
merged, only the PR name and description need follow to follow the conventions
below; otherwise, all commits must be appropriately named with good
descriptions.

Draft PRs are welcome. When marked ready to review, a committer will review
your PR. If your PR contains a major change (e.g. a new feature), it will be
left open for a period of time before merging to allow comments.

### Coding Conventions

* Obey `cargo clippy` and `cargo fmt`. This is checked in CI.

* Prefix commit titles with `feat:`, `test:`, `doc:`, `perf:`, `refactor:`,
  etc.  If a change affects the derive macro only, this should be specified by
  placing brackets after the change type, e.g. `feat(derive):`.
    
    + These prefixes are used during the release process to help with semantic
      versioning.

*  We follow no particular commit template; however, you should clearly
   describe your changes in each commit message (or in the PR description if
   squash-merging).

     + For a good general guide, see [writing commit messages](https://www.chiark.greenend.org.uk/~sgtatham/quasiblog/commit-messages/).

* Peformance commits should provide benchmark data (either from Uniplate or
  Conjure Oxide) justifying the change.
