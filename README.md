# Forked React Compiler

* this vendors the [React Compiler (Rust port)](https://github.com/facebook/react/pull/36173) into [`react-compiler/`](./react-compiler)
* patches `Cargo.toml` to make it releasable (published as `forked_react_compiler_*`)
* license is "Copyright (c) Meta Platforms, Inc. and affiliates."

## Why this exists

[oxc](https://github.com/oxc-project/oxc) and [Rolldown](https://github.com/rolldown/rolldown) need
the React Compiler Rust port on crates.io, but published crates can't use `git` dependencies — every
dependency must itself be on crates.io. The port lives only in an unmerged
[React PR](https://github.com/facebook/react/pull/36173) and was never published, so this repo
vendors it, patches the crates to be releasable, and publishes them as `forked_react_compiler_*`.

The source is synced over unchanged — the only edits are to `Cargo.toml` files (no code changes).
