# oxc-react-compiler

Vendors the **React Compiler (Rust port)** from
[facebook/react#36173](https://github.com/facebook/react/pull/36173)
(*"[compiler] Port React Compiler to Rust"*) into [`react-compiler/`](./react-compiler).

`react-compiler/` is react's `compiler/` directory: a Cargo workspace
(`react-compiler/Cargo.toml`) whose crates include `react_compiler_oxc`, alongside the reference
TypeScript `babel-plugin-react-compiler` and its test fixtures.

It is vendored as a **linear snapshot** rather than a merge-based `git subtree`, so `main` stays
free of merge commits: each sync re-snapshots upstream's `compiler/` into `react-compiler/` as a
single ordinary commit (handling additions, edits, and deletions).

## Updating

```sh
just import   # one-time: create react-compiler/ (already done)
just sync     # pull the latest state of PR #36173 into react-compiler/
just status   # show which upstream commit is currently vendored
```

The source ref and target directory are configurable at the top of the [`justfile`](./justfile).
