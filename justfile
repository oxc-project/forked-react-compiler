# This repo vendors the React Compiler (Rust port) from facebook/react PR #36173
# into ./react-compiler.
#
# The oxc-project org ruleset forbids merge commits on default branches, so the
# merge-based `git subtree` model can't be used. Instead each `sync` re-snapshots
# react's `compiler/` directory into ./react-compiler as a single linear commit
# (additions, edits, and deletions are all handled), which keeps `main` linear.
#
#   just import   # one-time: create ./react-compiler
#   just sync     # update ./react-compiler to the latest PR state
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

# Snapshot react's `compiler/` into ./{{prefix}} as a single linear commit
sync:
    #!/usr/bin/env bash
    set -euo pipefail
    git fetch --depth=1 --no-tags {{react_repo}} {{pr_ref}}
    upstream="$(git rev-parse FETCH_HEAD)"
    tree="$(git rev-parse "FETCH_HEAD:{{src_dir}}")"
    # Replace ./{{prefix}} with the upstream subtree wholesale (clears deletions too)
    git rm -r --cached --quiet --ignore-unmatch {{prefix}}
    rm -rf {{prefix}}
    git read-tree --prefix={{prefix}}/ -u "$tree"
    git add -A {{prefix}}
    if git diff --cached --quiet -- {{prefix}}; then
        echo "{{prefix}} already at react {{pr_ref}} @ ${upstream} — nothing to commit."
    else
        git commit -q -m "vendor: react-compiler from {{pr_ref}} @ ${upstream}"
        echo "Committed {{prefix}} @ ${upstream}."
    fi

# Show the upstream commit currently vendored
status:
    #!/usr/bin/env bash
    set -euo pipefail
    line="$(git log -1 --grep='^vendor: react-compiler' --format='%h  %ci%n%s' 2>/dev/null || true)"
    if [ -n "$line" ]; then echo "$line"; else echo "Nothing vendored yet — run 'just import'."; fi
