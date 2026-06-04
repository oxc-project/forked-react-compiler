//! Transforms the freshly-vendored React Compiler tree so the crates can be
//! published to crates.io. Run on every `just sync`; idempotent.
//!
//! 1. renames every crate to `forked_react_compiler_*` (dirs, package names, path
//!    deps, source);
//! 2. sets up workspace inheritance like the main oxc repo — `[workspace.package]`
//!    (version/edition/license/description/repository) and `[workspace.dependencies]`
//!    (internal crates, with versions) — and points each crate at them;
//! 3. writes React's MIT LICENSE (kept as a local copy and linked in below);
//! 4. drops Cargo.lock (regenerated locally, git-ignored).
//!
//! Usage: `codemod [TREE_DIR]`   (default: `react-compiler`)

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use toml_edit::{value, DocumentMut, InlineTable, Item, Table, Value};

/// Published version for every crate. Bump before each publish — crates.io
/// rejects re-publishing an already-published version.
const VERSION: &str = "0.1.0";
const EDITION: &str = "2024";
const LICENSE: &str = "MIT";
const DESCRIPTION: &str = "Rust port of the React Compiler, vendored from facebook/react.";
const REPOSITORY: &str = "https://github.com/oxc-project/oxc-react-compiler";
/// React's MIT LICENSE, kept as a local copy (`./LICENSE`) and linked into the
/// tool so syncing needs no network for it.
const LICENSE_TEXT: &str = include_str!("../../LICENSE");

/// File extensions whose contents may reference crate identifiers.
const EXTS: &[&str] = &["rs", "toml", "md", "ts", "tsx", "js", "mjs", "cjs", "sh"];
/// Directories to skip while walking the tree.
const SKIP_DIRS: &[&str] = &["node_modules", "target", ".git"];

/// A workspace member crate.
struct Member {
    /// Package name, e.g. `forked_react_compiler_ast`.
    name: String,
    /// Path relative to the workspace root, e.g. `crates/forked_react_compiler_ast`.
    rel_path: String,
    /// Absolute path to the crate's `Cargo.toml`.
    manifest: PathBuf,
}

fn main() {
    let root = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "react-compiler".to_string()),
    );

    let renamed = rename_crate_dirs(&root.join("crates"));
    let rewritten = rewrite_idents(&root);

    let members = members(&root);
    set_up_workspace(&root, &members);

    fs::write(root.join("LICENSE"), LICENSE_TEXT).expect("write LICENSE");

    let dropped_lock = fs::remove_file(root.join("Cargo.lock")).is_ok();

    println!(
        "codemod: renamed {renamed} dir(s), rewrote {rewritten} file(s), prepared {} crate(s), wrote LICENSE{}",
        members.len(),
        if dropped_lock { ", dropped Cargo.lock" } else { "" },
    );
}

// --- 1. renaming ------------------------------------------------------------

/// Rename `crates/react_compiler*` -> `crates/forked_react_compiler*`.
fn rename_crate_dirs(crates: &Path) -> usize {
    let mut n = 0;
    let Ok(entries) = fs::read_dir(crates) else {
        return 0;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.path().is_dir() && name.starts_with("react_compiler") {
            fs::rename(entry.path(), crates.join(format!("forked_{name}")))
                .expect("rename crate dir");
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

/// Rename every `react_compiler` identifier to `forked_react_compiler`, without
/// double-prefixing an existing `forked_react_compiler` (so it is idempotent). The
/// npm-style hyphenated `react-compiler` is untouched. Uses NUL sentinels that never
/// occur in text files.
fn prefix_idents(s: &str) -> String {
    const SENTINEL: &str = "\u{0}FORKED_RC\u{0}";
    s.replace("forked_react_compiler", SENTINEL)
        .replace("react_compiler", "forked_react_compiler")
        .replace(SENTINEL, "forked_react_compiler")
}

// --- 2. workspace inheritance (oxc style) -----------------------------------

fn set_up_workspace(root: &Path, members: &[Member]) {
    edit_root_manifest(root, members);
    let internal: BTreeSet<&str> = members.iter().map(|m| m.name.as_str()).collect();
    for member in members {
        edit_member_manifest(&member.manifest, &internal);
    }
}

/// Resolve workspace `members` (expanding a trailing `/*`) to `Member`s, sorted by name.
fn members(root: &Path) -> Vec<Member> {
    let doc = read_doc(&root.join("Cargo.toml"));
    let entries = doc
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(Item::as_array);

    let mut out = Vec::new();
    let Some(entries) = entries else {
        return out;
    };
    for member in entries.iter().filter_map(|m| m.as_str()) {
        let rel_paths = match member.strip_suffix("/*") {
            Some(parent) => match fs::read_dir(root.join(parent)) {
                Ok(rd) => rd
                    .flatten()
                    .filter(|e| e.path().is_dir())
                    .map(|e| format!("{parent}/{}", e.file_name().to_string_lossy()))
                    .collect(),
                Err(_) => Vec::new(),
            },
            None => vec![member.to_string()],
        };
        for rel_path in rel_paths {
            let manifest = root.join(&rel_path).join("Cargo.toml");
            if let Some(name) = package_name(&manifest) {
                out.push(Member { name, rel_path, manifest });
            }
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

fn package_name(manifest: &Path) -> Option<String> {
    let doc: DocumentMut = fs::read_to_string(manifest).ok()?.parse().ok()?;
    Some(doc.get("package")?.get("name")?.as_str()?.to_string())
}

/// Add `[workspace.package]` and `[workspace.dependencies]` to the root manifest.
fn edit_root_manifest(root: &Path, members: &[Member]) {
    let path = root.join("Cargo.toml");
    let mut doc = read_doc(&path);

    let mut package = Table::new();
    package.insert("version", value(VERSION));
    package.insert("edition", value(EDITION));
    package.insert("license", value(LICENSE));
    package.insert("description", value(DESCRIPTION));
    package.insert("repository", value(REPOSITORY));

    let mut dependencies = Table::new();
    for member in members {
        let mut dep = InlineTable::new();
        dep.insert("version", Value::from(VERSION));
        dep.insert("path", Value::from(member.rel_path.as_str()));
        dep.fmt();
        dependencies.insert(&member.name, value(dep));
    }

    let workspace = doc["workspace"]
        .as_table_mut()
        .expect("[workspace] table");
    workspace.insert("package", Item::Table(package));
    workspace.insert("dependencies", Item::Table(dependencies));

    fs::write(&path, doc.to_string()).expect("write workspace Cargo.toml");
}

/// Inherit publishing fields from the workspace and use `{ workspace = true }`
/// for internal dependencies.
fn edit_member_manifest(path: &Path, internal: &BTreeSet<&str>) {
    let mut doc = read_doc(path);

    if let Some(pkg) = doc.get_mut("package").and_then(Item::as_table_mut) {
        for field in ["version", "edition", "license", "description", "repository"] {
            pkg.insert(field, workspace_inherited());
        }
    }

    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        let keys: Vec<String> = match doc.get(section).and_then(Item::as_table) {
            Some(table) => table
                .iter()
                .map(|(k, _)| k.to_string())
                .filter(|k| internal.contains(k.as_str()))
                .collect(),
            None => continue,
        };
        for key in keys {
            let dep = workspace_dep(&doc[section][key.as_str()]);
            if let Some(table) = doc[section].as_table_mut() {
                table.insert(&key, dep);
            }
        }
    }

    fs::write(path, doc.to_string()).expect("write manifest");
}

/// A dotted `field.workspace = true` item.
fn workspace_inherited() -> Item {
    let mut table = Table::new();
    table.set_dotted(true);
    table.insert("workspace", value(true));
    Item::Table(table)
}

/// `{ workspace = true }`, preserving `features` / `optional` / `default-features`.
fn workspace_dep(old: &Item) -> Item {
    let mut dep = InlineTable::new();
    dep.insert("workspace", Value::from(true));
    for key in ["features", "optional", "default-features"] {
        let existing = old
            .as_inline_table()
            .and_then(|t| t.get(key).cloned())
            .or_else(|| {
                old.as_table()
                    .and_then(|t| t.get(key))
                    .and_then(Item::as_value)
                    .cloned()
            });
        if let Some(v) = existing {
            dep.insert(key, v);
        }
    }
    dep.fmt();
    value(dep)
}

// --- helpers ----------------------------------------------------------------

fn read_doc(path: &Path) -> DocumentMut {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
        .parse()
        .unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}
