# This repo vendors the React Compiler (Rust port) from facebook/react PR #36173
# into ./react-compiler, then prefixes every crate with `oxc_` so they can be
# published to crates.io under the oxc namespace.
#
# The oxc-project org ruleset forbids merge commits, so the vendor is a linear
# snapshot (not `git subtree`): each `sync` re-extracts upstream's `compiler/`,
# re-applies the `oxc_` rename, and commits once. The rename is re-applied every
# sync because the snapshot is taken fresh from upstream each time.
#
#   just import   # one-time: create ./react-compiler (oxc_-prefixed)
#   just sync     # update ./react-compiler to the latest PR state (re-prefixed)
#   just prefix   # (re)apply the oxc_ rename to ./react-compiler in place
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

# (Re)apply the oxc_ crate rename to ./{{prefix}} (idempotent; run automatically by `sync`)
prefix:
    #!/usr/bin/env bash
    set -euo pipefail
    cd {{prefix}}
    # 1. rename crate directories: crates/react_compiler* -> crates/oxc_react_compiler*
    for d in crates/react_compiler*; do
        [ -e "$d" ] || continue
        mv "$d" "crates/oxc_$(basename "$d")"
    done
    # 2. rewrite crate identifiers in manifests, source, scripts, and docs.
    #    Only the underscore `react_compiler` form is rewritten (npm's hyphenated
    #    `react-compiler` is untouched); the (?<!oxc_) guard keeps it idempotent.
    find . -type d \( -name node_modules -o -name target -o -name .git \) -prune -o \
        -type f \( -name '*.rs' -o -name '*.toml' -o -name '*.md' -o -name '*.ts' \
                   -o -name '*.tsx' -o -name '*.js' -o -name '*.mjs' -o -name '*.cjs' -o -name '*.sh' \) \
        -exec perl -i -pe 's/(?<!oxc_)react_compiler/oxc_react_compiler/g' {} +
    # 3. Cargo.lock is regenerated locally by cargo and is git-ignored (keeps `sync` deterministic)
    rm -f Cargo.lock

# Show the upstream commit currently vendored
status:
    #!/usr/bin/env bash
    set -euo pipefail
    line="$(git log -1 --grep='^vendor: react-compiler' --format='%h  %ci%n%s' 2>/dev/null || true)"
    if [ -n "$line" ]; then echo "$line"; else echo "Nothing vendored yet — run 'just import'."; fi
