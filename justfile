# This repo vendors the React Compiler (Rust port) from facebook/react PR #36173
# into ./react-compiler, then (editing only Cargo.toml for crate naming) sets each
# crate's published name to `forked_react_compiler_*` while keeping the lib/import
# name, source, and directories as upstream's `react_compiler_*`, and adds the
# metadata + `publish` flags to release `forked_react_compiler` and its deps to crates.io.
#
# The oxc-project org ruleset forbids merge commits, so the vendor is a linear
# snapshot (not `git subtree`): each `sync` re-extracts upstream's `compiler/`,
# re-runs the transform tool (./codemod), and commits once. The transform is
# re-applied every sync because the snapshot is taken fresh.
#
#   just import       # one-time: create ./react-compiler (transformed)
#   just sync         # update ./react-compiler to the latest PR state (re-transformed)
#   just codemod      # (re)run codemod on ./react-compiler in place
#   just check        # cargo check the vendored workspace
#   just publish-dry  # dry-run publishing the whole set (cargo publish --workspace --dry-run)
#   just publish      # publish forked_react_compiler + its deps via cargo-release-oxc
#   just status       # show which upstream commit is currently vendored

react_repo := "https://github.com/facebook/react.git"
pr_ref     := "pull/36173/head"
src_dir    := "compiler"          # path of the compiler inside the react monorepo
prefix     := "react-compiler"    # where it lives in THIS repo

# Show available recipes
default:
    @just --list

# One-time import (same operation as sync; kept for discoverability)
import: sync

# Snapshot react's `compiler/` into ./{{prefix}}, set published names to forked_react_compiler_*, commit once
sync:
    #!/usr/bin/env bash
    set -euo pipefail
    git fetch --depth=1 --no-tags {{react_repo}} {{pr_ref}}
    upstream="$(git rev-parse FETCH_HEAD)"
    tree="$(git rev-parse "FETCH_HEAD:{{src_dir}}")"
    git rm -r --cached --quiet --ignore-unmatch {{prefix}}
    rm -rf {{prefix}}
    git read-tree --prefix={{prefix}}/ -u "$tree"
    just codemod
    git add -A {{prefix}}
    if git diff --cached --quiet -- {{prefix}}; then
        echo "{{prefix}} already at react {{pr_ref}} @ ${upstream} — nothing to commit."
    else
        git commit -q -m "vendor: react-compiler from {{pr_ref}} @ ${upstream} (forked_react_compiler)"
        echo "Committed {{prefix}} @ ${upstream}."
    fi

# (Re)run codemod on ./{{prefix}}: published names, workspace deps, publish flags, LICENSE, oxc_release.toml
codemod:
    cargo run --quiet -p codemod -- {{prefix}}

# Type-check the vendored workspace
check:
    cd {{prefix}} && cargo check --workspace

# Dry-run publishing the whole publishable set (cargo's workspace publish handles inter-crate deps)
publish-dry:
    cd {{prefix}} && cargo publish --workspace --dry-run

# Needs cargo-release-oxc (`cargo install --git https://github.com/oxc-project/cargo-release-oxc`),
# a clean git tree, and CARGO_REGISTRY_TOKEN.
# Publish forked_react_compiler + its deps in order, skipping publish=false crates
publish:
    cd {{prefix}} && cargo release-oxc publish --release crates

# Show the upstream commit currently vendored
status:
    #!/usr/bin/env bash
    set -euo pipefail
    line="$(git log -1 --grep='^vendor: react-compiler' --format='%h  %ci%n%s' 2>/dev/null || true)"
    if [ -n "$line" ]; then echo "$line"; else echo "Nothing vendored yet — run 'just import'."; fi
