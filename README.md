# oxc-react-compiler

Vendors the **React Compiler (Rust port)** from
[facebook/react#36173](https://github.com/facebook/react/pull/36173)
(*"[compiler] Port React Compiler to Rust"*) into [`react-compiler/`](./react-compiler).

`react-compiler/` is react's `compiler/` directory: a Cargo workspace
(`react-compiler/Cargo.toml`) whose crates include `oxc_react_compiler_oxc`, alongside the reference
TypeScript `babel-plugin-react-compiler` and its test fixtures.

It is vendored as a **linear snapshot** rather than a merge-based `git subtree`, so `main` stays
free of merge commits: each sync re-snapshots upstream's `compiler/` into `react-compiler/` as a
single ordinary commit (handling additions, edits, and deletions).

Every crate is then prefixed with **`oxc_`** (`react_compiler_oxc` → `oxc_react_compiler_oxc`, etc.)
so the crates can be published under the oxc namespace. This rename is re-applied automatically on
every `just sync`.

## Updating

```sh
just import   # one-time: create react-compiler/ (already done)
just sync     # pull the latest state of PR #36173 into react-compiler/ (re-prefixed)
just prefix   # (re)apply the oxc_ rename to react-compiler/ in place
just status   # show which upstream commit is currently vendored
```

The source ref and target directory are configurable at the top of the [`justfile`](./justfile).
