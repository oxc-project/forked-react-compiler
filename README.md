# Forked React Compiler

* this vendors the [React Compiler (Rust port)](https://github.com/facebook/react/pull/36173) into [`react-compiler/`](./react-compiler)
* patches `Cargo.toml` to make it releasable (published as `forked_react_compiler_*`)
* license is "Copyright (c) Meta Platforms, Inc. and affiliates."

## Why this exists

[Rolldown](https://github.com/rolldown/rolldown) needs the React Compiler's Rust port available on
crates.io. That port currently lives only in an unmerged pull request against `facebook/react`
([#36173](https://github.com/facebook/react/pull/36173)), and its crates have never been published
to any registry.

A crate uploaded to crates.io cannot depend on `git` or `path` dependencies — every dependency of a
published crate must itself resolve to a released version on crates.io. So Rolldown, which is itself
published to crates.io, cannot point a `git = "…"` dependency at the React PR: doing so would make
Rolldown unpublishable.

This repository bridges that gap. It vendors the React Compiler crates, patches their `Cargo.toml`
files so they are valid to publish (renaming the published package to `forked_react_compiler_*` and
adding the version / license / description / `[workspace.dependencies]` that crates.io requires), and
publishes them to crates.io. Rolldown can then depend on the `forked_react_compiler_*` crates as
ordinary registry dependencies.
