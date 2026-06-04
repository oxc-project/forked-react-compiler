//! Prepares the freshly-vendored React Compiler crates for publishing — by editing
//! ONLY `Cargo.toml` files. Source, directories, and crate import/lib names stay
//! exactly as upstream (`react_compiler*`); the only thing that changes is the
//! published `[package] name`, which becomes `forked_react_compiler*`. Run on every
//! `just sync`; idempotent.
//!
//! Per crate (`Cargo.toml` only):
//!   - `[package] name`  -> `forked_react_compiler_X`  (the published name)
//!   - `[lib] name`      -> `react_compiler_X`         (kept, so `use react_compiler_X` still works)
//!   - inherits version/edition/license/description/repository from `[workspace.package]`
//!   - internal deps become `{ workspace = true }`
//! Root `[workspace.dependencies]` maps each import name to its published package:
//!   `react_compiler_X = { package = "forked_react_compiler_X", version, path }`
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

/// A workspace member crate.
struct Member {
    /// Lib/import name, kept as upstream, e.g. `react_compiler_ast`.
    import_name: String,
    /// Published package name, e.g. `forked_react_compiler_ast`.
    package_name: String,
    /// Path relative to the workspace root, e.g. `crates/react_compiler_ast`.
    rel_path: String,
    /// Absolute path to the crate's `Cargo.toml`.
    manifest: PathBuf,
    /// Whether the crate has a library target (`src/lib.rs`).
    has_lib: bool,
}

fn main() {
    let root = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "react-compiler".to_string()),
    );

    let members = members(&root);
    edit_root_manifest(&root, &members);
    let internal: BTreeSet<&str> = members.iter().map(|m| m.import_name.as_str()).collect();
    for member in &members {
        edit_member_manifest(member, &internal);
    }

    fs::write(root.join("LICENSE"), LICENSE_TEXT).expect("write LICENSE");
    let dropped_lock = fs::remove_file(root.join("Cargo.lock")).is_ok();

    println!(
        "codemod: published name -> forked_react_compiler_* on {} crate(s) (Cargo.toml only), wrote LICENSE{}",
        members.len(),
        if dropped_lock { ", dropped Cargo.lock" } else { "" },
    );
}

/// Resolve workspace `members` (expanding a trailing `/*`) to `Member`s, sorted by import name.
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
            if let Some(raw) = package_name(&manifest) {
                // Derive the upstream import name (idempotent: strip a prior `forked_`).
                let import_name = raw.strip_prefix("forked_").unwrap_or(&raw).to_string();
                let package_name = format!("forked_{import_name}");
                let has_lib = root.join(&rel_path).join("src/lib.rs").exists();
                out.push(Member { import_name, package_name, rel_path, manifest, has_lib });
            }
        }
    }
    out.sort_by(|a, b| a.import_name.cmp(&b.import_name));
    out
}

fn package_name(manifest: &Path) -> Option<String> {
    let doc: DocumentMut = fs::read_to_string(manifest).ok()?.parse().ok()?;
    Some(doc.get("package")?.get("name")?.as_str()?.to_string())
}

/// Add `[workspace.package]` and `[workspace.dependencies]` (mapping each import name
/// to its published package) to the root manifest.
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
        dep.insert("package", Value::from(member.package_name.as_str()));
        dep.insert("version", Value::from(VERSION));
        dep.insert("path", Value::from(member.rel_path.as_str()));
        dep.fmt();
        dependencies.insert(&member.import_name, value(dep));
    }

    let workspace = doc["workspace"]
        .as_table_mut()
        .expect("[workspace] table");
    workspace.insert("package", Item::Table(package));
    workspace.insert("dependencies", Item::Table(dependencies));

    fs::write(&path, doc.to_string()).expect("write workspace Cargo.toml");
}

/// Rename the published `[package] name`, keep the `[lib] name`, inherit publishing
/// fields, and point internal deps at the workspace.
fn edit_member_manifest(member: &Member, internal: &BTreeSet<&str>) {
    let mut doc = read_doc(&member.manifest);

    if let Some(pkg) = doc.get_mut("package").and_then(Item::as_table_mut) {
        pkg.insert("name", value(member.package_name.as_str()));
        for field in ["version", "edition", "license", "description", "repository"] {
            pkg.insert(field, workspace_inherited());
        }
    }

    // Keep the importable crate name as upstream so source needs no changes.
    if member.has_lib {
        if doc.get("lib").and_then(Item::as_table).is_none() {
            doc["lib"] = Item::Table(Table::new());
        }
        if let Some(lib) = doc["lib"].as_table_mut() {
            lib.insert("name", value(member.import_name.as_str()));
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

    fs::write(&member.manifest, doc.to_string()).expect("write manifest");
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

fn read_doc(path: &Path) -> DocumentMut {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
        .parse()
        .unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}
