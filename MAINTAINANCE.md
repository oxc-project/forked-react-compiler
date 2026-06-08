# Maintenance

How to keep `react-compiler/` in sync with upstream and publish the
`forked_react_compiler_*` crates to crates.io.

All recipes live in the [`justfile`](./justfile); run `just --list` for the full set.

## Syncing from upstream

```sh
just sync      # re-extract react's compiler/ (PR #36173), re-run codemod, re-apply patches, commit
just status    # show which upstream commit is currently vendored
just check     # cargo check the vendored workspace
```

`just sync` snapshots upstream's `compiler/` as one linear commit (no merge commits — the
oxc-project org ruleset forbids them), then runs `just codemod` and `just patch`. Landing it on
`main` is a PR workflow (squash-merge only; direct push to `main` is blocked):

```sh
git switch -c sync-react-compiler
just sync
git push -u origin sync-react-compiler
gh pr create --fill && gh pr merge --squash --auto
```

The codemod edits **only** `Cargo.toml` files: it sets each crate's published `[package] name` to
`forked_react_compiler_*` while keeping the `[lib] name` (and source/dirs) as upstream's
`react_compiler_*`, wires up workspace inheritance, and marks each crate `publish = true`/`false`.

## Releasing to crates.io

Publishing uses [`cargo-release-oxc`](https://github.com/oxc-project/cargo-release-oxc),
configured by the codemod-written `react-compiler/oxc_release.toml`.

### Prerequisites

```sh
cargo install --git https://github.com/oxc-project/cargo-release-oxc
```

- A clean git tree (cargo-release-oxc refuses to run otherwise).
- crates.io credentials — either `CARGO_REGISTRY_TOKEN` in the environment or a token stored in
  `~/.cargo/credentials.toml`.

### 1. Bump the version

The version is a single source of truth in `react-compiler/Cargo.toml`. Re-run the codemod with the
new version to re-stamp every crate, then land it via a PR:

```sh
git switch -c release-0.1.3
just codemod 0.1.3
git commit -aqm "release: bump to 0.1.3"
git push -u origin release-0.1.3
gh pr create --title "release: bump to 0.1.3" --fill
gh pr merge --squash --auto
```

`just sync` preserves the current version across re-extraction, so a plain sync never resets it —
only an explicit `just codemod <version>` bumps it.

### 2. Publish

From an up-to-date, clean `main`:

```sh
just publish        # cd react-compiler && cargo release-oxc publish --release crates
```

This publishes `forked_react_compiler` plus its 11 transitive deps **bottom-up** (deps before
dependents, 12 crates total), skipping the `publish = false` crates
(`*_oxc`, `*_swc`, `*_napi`, `*_e2e_cli`). cargo-release-oxc waits for each crate to appear on the
index before publishing its dependents.

### Dry run

```sh
just publish-dry    # cargo publish --workspace --dry-run
```

> [!NOTE]
> A dry run **fails partway through** with `failed to select a version for the requirement
> forked_react_compiler_X = "^<version>"` — this is expected. Nothing is actually uploaded in a dry
> run, so each dependent crate can't find its just-"published" dep on the index. The real
> `just publish` succeeds because it uploads bottom-up and each dep exists before its dependents are
> packaged. The dry run still confirms the leaf crates package cleanly.
