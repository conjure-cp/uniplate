# Maintainers Guide 

*This document covers the process for maintainers of Uniplate. For general
information about contributing to Uniplate, see [CONTRIBUTING.md].*

This project is part of the [conjure](https://github.com/conjure-cp) project;
however, it mostly has its own organisational structure, which is described in
this document.

Uniplate maintainers can be split into the following categories:

* *Committers* (@uniplate-committers) have full write access, the ability to
  make releases, and perform code review. 

* The *Project Owners* (@ozgurakgun @niklasdewally) have repository admin
  access.

## List of Maintainers

@niklasdewally
@ozgurakgun
@lixitrixi
@gskorokhod

## PR submission and review process

All PRs must be approved by a committer who is not the author before merging.

Our goal is for every change to have a consensus between committers. For minor
changes, such as bug fixes or performance improvements, it is obvious that they
are the right thing to do, so PRs can be merged immediately. However, major
changes should be left open for a period before merging to ensure a consensus
exist, as described in the following section.

If you think another reviewer should see this code, say something like "LGTM,
but @foo should review before merging".

See [CONTRIBUTING.md] for code-review requirements.

## Major Change Approval Process

Examples of major changes include user-facing changes, new features, large
refactors, and anything likely to break Conjure Oxide.

A consensus between committers is required on a major change before PRs
relating to it can be merged. We achieve consensus lazily; that is, a change
can be merged as long as no objections are raised.

Major changes can be proposed either as an issue, or in a PR.

1. Ping @uniplate-committers in the PR/issue in question, asking for any
   objections on the change. 

   For PRs, this can be done once code review is completed, e.g. "LGTM, leaving
   open for a week for feedback @uniplate-committers".

2. After a period of at least a week, consensus is assumed as long as no
   objections are raised.

3. During this period, committers can object to the change or add themselves as
   reviewers.

4. Once a consensus is reached, the change is approved.

5. If a major PR is urgent, ask @uniplate-committers for an explicit response
   or ping @ozgurakgun if Conjure Oxide related.

The project owners have the final say: in particular, a change may be vetoed if
it breaks Conjure Oxide in any way.

Questions can also be raised about already merged PRs. In this case, reverting
them might be appropriate.

## Releases

Releases can be made by merging the open `release-plz` PR. Please add a
synopsis to the changelog summarising the release and tidy up the changelog
before merging.
