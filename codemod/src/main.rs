//! Rewrites the vendored React Compiler tree so every crate is prefixed with
//! `oxc_` (e.g. `react_compiler_oxc` -> `oxc_react_compiler_oxc`). Idempotent:
//! safe to run repeatedly, including on an already-prefixed tree.
//!
//! Usage: `codemod [TREE_DIR]`   (default: `react-compiler`)

use std::fs;
use std::path::{Path, PathBuf};

/// File extensions whose contents may reference crate identifiers.
const EXTS: &[&str] = &["rs", "toml", "md", "ts", "tsx", "js", "mjs", "cjs", "sh"];
/// Directories to skip while walking the tree.
const SKIP_DIRS: &[&str] = &["node_modules", "target", ".git"];

fn main() {
    let root = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "react-compiler".to_string()),
    );

    let renamed = rename_crate_dirs(&root.join("crates"));
    let files = rewrite_idents(&root);
    let dropped_lock = fs::remove_file(root.join("Cargo.lock")).is_ok();

    println!(
        "codemod: renamed {renamed} crate dir(s), rewrote {files} file(s){}",
        if dropped_lock { ", dropped Cargo.lock" } else { "" }
    );
}

/// Rename `crates/react_compiler*` -> `crates/oxc_react_compiler*`.
fn rename_crate_dirs(crates: &Path) -> usize {
    let mut n = 0;
    let Ok(entries) = fs::read_dir(crates) else {
        return 0;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.path().is_dir() && name.starts_with("react_compiler") {
            fs::rename(entry.path(), crates.join(format!("oxc_{name}"))).expect("rename crate dir");
            n += 1;
        }
    }
    n
}

/// Recursively rewrite crate identifiers in text files under `dir`.
fn rewrite_idents(dir: &Path) -> usize {
    let mut count = 0;
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !SKIP_DIRS.contains(&name.as_str()) {
                count += rewrite_idents(&path);
            }
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| EXTS.contains(&e))
        {
            if let Ok(content) = fs::read_to_string(&path) {
                let rewritten = prefix_idents(&content);
                if rewritten != content {
                    fs::write(&path, rewritten).expect("write file");
                    count += 1;
                }
            }
        }
    }
    count
}

/// Prefix every `react_compiler` identifier with `oxc_`, without double-prefixing
/// an existing `oxc_react_compiler` (so it is idempotent). The npm-style hyphenated
/// `react-compiler` is untouched. Uses NUL sentinels that never occur in text files.
fn prefix_idents(s: &str) -> String {
    const SENTINEL: &str = "\u{0}OXC_RC\u{0}";
    s.replace("oxc_react_compiler", SENTINEL)
        .replace("react_compiler", "oxc_react_compiler")
        .replace(SENTINEL, "oxc_react_compiler")
}
