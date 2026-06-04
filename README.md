# oxc-react-compiler

Vendors the **React Compiler (Rust port)** from
[facebook/react#36173](https://github.com/facebook/react/pull/36173)
(*"[compiler] Port React Compiler to Rust"*) into [`react-compiler/`](./react-compiler).

`react-compiler/` is react's `compiler/` directory: a Cargo workspace
(`react-compiler/Cargo.toml`) whose crates are published as `forked_react_compiler_*` (imported in
code as `react_compiler_*`), alongside the reference TypeScript `babel-plugin-react-compiler` and its
test fixtures.

It is vendored as a **linear snapshot** rather than a merge-based `git subtree`, so `main` stays
free of merge commits: each sync re-snapshots upstream's `compiler/` into `react-compiler/` as a
single ordinary commit (handling additions, edits, and deletions).

After each snapshot, a single Rust tool — [`codemod/`](./codemod) — (re)transforms the tree so the
crates can be published to crates.io, re-applied automatically on every `just sync`. **It edits only
`Cargo.toml` files** — source, directories, and crate import names stay exactly as upstream
(`react_compiler_*`). It:

- sets each crate's published **`[package] name`** to `forked_react_compiler_*` (`react_compiler_oxc`
  → `forked_react_compiler_oxc`, etc.) while keeping `[lib] name = react_compiler_*`, so
  `use react_compiler_*` still compiles unchanged;
- sets up workspace inheritance like the main [oxc](https://github.com/oxc-project/oxc) repo —
  `[workspace.package]` (`version` / `edition` / `license` / `description` / `repository`) and
  `[workspace.dependencies]` (internal crates, with versions) — so each crate uses
  `field.workspace = true` and `dep = { workspace = true }`;
- writes React's MIT `LICENSE` (kept as a local copy at [`./LICENSE`](./LICENSE) and embedded into
  the tool via `include_str!`, so syncing needs no network for it).

The publish version is the `VERSION` constant in [`codemod/src/main.rs`](./codemod/src/main.rs);
bump it before publishing, since crates.io rejects re-publishing an existing version.

## Updating

```sh
just import   # one-time: create react-compiler/ (already done)
just sync     # pull the latest state of PR #36173, re-transform, and commit
just prefix   # (re)run codemod on react-compiler/ in place
just status   # show which upstream commit is currently vendored
```

The source ref and target directory are configurable at the top of the [`justfile`](./justfile).
