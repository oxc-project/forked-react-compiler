# Forked React Compiler

* this vendors the [React Compiler (Rust port)](https://github.com/facebook/react/pull/36173) into [`react-compiler/`](./react-compiler)
* patches `Cargo.toml` to make it releasable (published as `forked_react_compiler_*`)
* license is "Copyright (c) Meta Platforms, Inc. and affiliates."

## Updating

```sh
just import   # one-time: create react-compiler/ (already done)
just sync     # pull the latest state of PR #36173, re-transform, and commit
just codemod  # (re)run codemod on react-compiler/ in place
just status   # show which upstream commit is currently vendored
```

The source ref and target directory are configurable at the top of the [`justfile`](./justfile).
