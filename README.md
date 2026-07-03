# Forked React Compiler

* this vendors the [React Compiler (Rust port)](https://github.com/react/react/tree/main/compiler) into [`react-compiler/`](./react-compiler)
* patches `Cargo.toml` to make it releasable — published to crates.io as [`forked_react_compiler_*`](https://crates.io/crates/forked_react_compiler)
* license is "Copyright (c) Meta Platforms, Inc. and affiliates."

## Why this exists

[oxc](https://github.com/oxc-project/oxc) and [Rolldown](https://github.com/rolldown/rolldown) need
the React Compiler Rust port on crates.io, but published crates can't use `git` dependencies — every
dependency must itself be on crates.io. The port lives in the
[React monorepo](https://github.com/react/react/tree/main/compiler) but isn't published to
crates.io, so this repo vendors it, patches the crates to be releasable, and publishes them as
`forked_react_compiler_*`.

The source is synced over unchanged — the only edits are to `Cargo.toml` files (no code changes).

## ❤ Who's [Sponsoring Oxc](https://github.com/sponsors/Boshen)?

<p align="center">
  <a href="https://github.com/sponsors/Boshen">
    <img src="https://raw.githubusercontent.com/Boshen/sponsors/main/sponsors.svg" alt="Our sponsors" />
  </a>
</p>
