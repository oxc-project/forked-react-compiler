# This repo vendors the React Compiler (Rust port) from facebook/react PR #36173
# into ./react-compiler, then prefixes every crate with `oxc_` and adds the
# metadata needed to publish them to crates.io under the oxc namespace.
#
# The oxc-project org ruleset forbids merge commits, so the vendor is a linear
# snapshot (not `git subtree`): each `sync` re-extracts upstream's `compiler/`,
# re-runs the transform tools (./codemod + ./prepare-publish), and commits once.
# The transform is re-applied every sync because the snapshot is taken fresh.
#
#   just import   # one-time: create ./react-compiler (transformed)
#   just sync     # update ./react-compiler to the latest PR state (re-transformed)
#   just prefix   # (re)run codemod + prepare-publish on ./react-compiler in place
#   just status   # show which upstream commit is currently vendored

react_repo := "https://github.com/facebook/react.git"
pr_ref     := "pull/36173/head"
src_dir    := "compiler"          # path of the compiler inside the react monorepo
prefix     := "react-compiler"    # where it lives in THIS repo

# Show available recipes
default:
    @just --list

# One-time import (same operation as sync; kept for discoverability)
import: sync

# Snapshot react's `compiler/` into ./{{prefix}}, oxc_-prefix the crates, commit once
sync:
    #!/usr/bin/env bash
    set -euo pipefail
    git fetch --depth=1 --no-tags {{react_repo}} {{pr_ref}}
    upstream="$(git rev-parse FETCH_HEAD)"
    tree="$(git rev-parse "FETCH_HEAD:{{src_dir}}")"
    git rm -r --cached --quiet --ignore-unmatch {{prefix}}
    rm -rf {{prefix}}
    git read-tree --prefix={{prefix}}/ -u "$tree"
    just prefix
    git add -A {{prefix}}
    if git diff --cached --quiet -- {{prefix}}; then
        echo "{{prefix}} already at react {{pr_ref}} @ ${upstream} — nothing to commit."
    else
        git commit -q -m "vendor: react-compiler from {{pr_ref}} @ ${upstream} (oxc_-prefixed)"
        echo "Committed {{prefix}} @ ${upstream}."
    fi

# Transform the vendored tree for publishing (idempotent; run automatically by `sync`):
# `codemod` oxc_-prefixes every crate; `prepare-publish` adds license/version/description.
prefix:
    cargo run --quiet -p codemod -- {{prefix}}
    cargo run --quiet -p prepare-publish -- {{prefix}}

# Show the upstream commit currently vendored
status:
    #!/usr/bin/env bash
    set -euo pipefail
    line="$(git log -1 --grep='^vendor: react-compiler' --format='%h  %ci%n%s' 2>/dev/null || true)"
    if [ -n "$line" ]; then echo "$line"; else echo "Nothing vendored yet — run 'just import'."; fi
