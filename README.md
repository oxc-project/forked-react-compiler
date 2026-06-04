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

After each snapshot, two small Rust tools (re)transform the tree so the crates can be published to
crates.io under the oxc namespace — re-applied automatically on every `just sync`:

- [`codemod/`](./codemod) — prefixes every crate with **`oxc_`** (`react_compiler_oxc` →
  `oxc_react_compiler_oxc`, etc.): directories, package names, path deps, and source.
- [`prepare-publish/`](./prepare-publish) — uses `toml_edit` to set `license` / `version` /
  `description` on each crate and add a `version` to internal path deps, and downloads React's MIT
  `LICENSE`.

The publish version is a constant in [`prepare-publish/src/main.rs`](./prepare-publish/src/main.rs)
(`VERSION`); bump it before publishing, since crates.io rejects re-publishing an existing version.

## Updating

```sh
just import   # one-time: create react-compiler/ (already done)
just sync     # pull the latest state of PR #36173, re-transform, and commit
just prefix   # (re)run codemod + prepare-publish on react-compiler/ in place
just status   # show which upstream commit is currently vendored
```

The source ref and target directory are configurable at the top of the [`justfile`](./justfile).
