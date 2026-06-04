//! Adds the metadata crates.io requires to publish the vendored React Compiler
//! crates: a `license` (taken from React's own LICENSE), a `version`, and a
//! `description` per crate, plus a `version` on each internal `path` dependency
//! (publishing rejects path-only deps). Idempotent. Run after `codemod`, so it
//! expects `oxc_`-prefixed crate names.
//!
//! Usage: `prepare-publish [TREE_DIR]`   (default: `react-compiler`)

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use toml_edit::{value, DocumentMut, Item};

/// Published version for every crate. Bump before each publish — crates.io
/// rejects re-publishing an already-published version.
const VERSION: &str = "0.1.0";
/// SPDX license. React (and oxc) are MIT.
const LICENSE: &str = "MIT";
/// React's LICENSE (MIT). Downloaded fresh on every run.
const LICENSE_URL: &str = "https://raw.githubusercontent.com/facebook/react/main/LICENSE";

fn main() {
    let root = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "react-compiler".to_string()),
    );

    match download(LICENSE_URL) {
        Ok(text) => {
            fs::write(root.join("LICENSE"), text).expect("write LICENSE");
            println!(
                "prepare-publish: wrote {}/LICENSE from {LICENSE_URL}",
                root.display()
            );
        }
        Err(e) => eprintln!(
            "prepare-publish: warning: could not download LICENSE ({e}); keeping existing file"
        ),
    }

    let manifests = member_manifests(&root);
    for manifest in &manifests {
        update_manifest(manifest);
    }
    println!(
        "prepare-publish: set license/version/description on {} crate(s) (license={LICENSE}, version={VERSION})",
        manifests.len()
    );
}

/// Download a URL as text via `curl` (keeps this tool dependency-light).
fn download(url: &str) -> Result<String, String> {
    let out = Command::new("curl")
        .args(["--fail", "--silent", "--show-error", "--location", url])
        .output()
        .map_err(|e| format!("failed to spawn curl: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "curl exited with {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    String::from_utf8(out.stdout).map_err(|e| format!("response was not UTF-8: {e}"))
}

/// Resolve the workspace `members` (expanding a trailing `/*`) to Cargo.toml paths.
fn member_manifests(root: &Path) -> Vec<PathBuf> {
    let text = fs::read_to_string(root.join("Cargo.toml")).expect("read workspace Cargo.toml");
    let doc: DocumentMut = text.parse().expect("parse workspace Cargo.toml");
    let members = doc
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(Item::as_array);

    let mut out = Vec::new();
    let Some(members) = members else {
        return out;
    };
    for member in members.iter().filter_map(|m| m.as_str()) {
        if let Some(parent) = member.strip_suffix("/*") {
            if let Ok(entries) = fs::read_dir(root.join(parent)) {
                for entry in entries.flatten().filter(|e| e.path().is_dir()) {
                    let manifest = entry.path().join("Cargo.toml");
                    if manifest.exists() {
                        out.push(manifest);
                    }
                }
            }
        } else {
            let manifest = root.join(member).join("Cargo.toml");
            if manifest.exists() {
                out.push(manifest);
            }
        }
    }
    out.sort();
    out
}

/// Set license/version/description on a crate and add `version` to its internal
/// path dependencies.
fn update_manifest(path: &Path) {
    let text = fs::read_to_string(path).expect("read manifest");
    let mut doc: DocumentMut = text.parse().expect("parse manifest");

    {
        let Some(pkg) = doc.get_mut("package").and_then(Item::as_table_mut) else {
            return; // virtual manifest — nothing to publish
        };
        let name = pkg
            .get("name")
            .and_then(Item::as_str)
            .unwrap_or_default()
            .to_string();
        pkg["version"] = value(VERSION);
        pkg["license"] = value(LICENSE);
        pkg["description"] = value(description_for(&name));
    }

    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        let keys: Vec<String> = match doc.get(section).and_then(Item::as_table) {
            Some(table) => table
                .iter()
                .map(|(k, _)| k.to_string())
                .filter(|k| k.starts_with("oxc_react_compiler"))
                .collect(),
            None => continue,
        };
        for key in keys {
            add_internal_version(&mut doc[section][key.as_str()]);
        }
    }

    fs::write(path, doc.to_string()).expect("write manifest");
}

/// Add `version = VERSION` to a `{ path = ... }` dependency that lacks one.
fn add_internal_version(item: &mut Item) {
    if let Some(table) = item.as_inline_table_mut() {
        if table.contains_key("path") {
            if !table.contains_key("version") {
                table.insert("version", VERSION.into());
            }
            table.fmt(); // normalize spacing: `{ path = "..", version = ".." }`
        }
    } else if let Some(table) = item.as_table_mut() {
        if table.contains_key("path") && !table.contains_key("version") {
            table["version"] = value(VERSION);
        }
    }
}

fn description_for(name: &str) -> String {
    let role = match name {
        "oxc_react_compiler" => "entrypoint and compilation pipeline",
        "oxc_react_compiler_ast" => "Babel-compatible AST types",
        "oxc_react_compiler_hir" => "high-level IR (HIR) types, environment, and visitors",
        "oxc_react_compiler_lowering" => "AST-to-HIR lowering (BuildHIR / HIRBuilder)",
        "oxc_react_compiler_inference" => "mutation, aliasing, and effect inference",
        "oxc_react_compiler_typeinference" => "type inference",
        "oxc_react_compiler_optimization" => "optimization passes",
        "oxc_react_compiler_validation" => "validation passes",
        "oxc_react_compiler_reactive_scopes" => "reactive-scope construction and codegen",
        "oxc_react_compiler_ssa" => "SSA construction",
        "oxc_react_compiler_diagnostics" => "diagnostics and code frames",
        "oxc_react_compiler_utils" => "shared utilities",
        "oxc_react_compiler_oxc" => "oxc front end",
        "oxc_react_compiler_swc" => "SWC front end",
        "oxc_react_compiler_e2e_cli" => "end-to-end test CLI",
        "oxc_react_compiler_napi" => "Node.js native binding",
        _ => "",
    };
    if role.is_empty() {
        "Rust port of the React Compiler, vendored from facebook/react by the oxc project.".to_string()
    } else {
        format!(
            "Rust port of the React Compiler — {role}. Vendored from facebook/react by the oxc project."
        )
    }
}
