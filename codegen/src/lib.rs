// SPDX-License-Identifier: CC0-1.0

//! Bitcoin Core OpenRPC to Rust bindings generator.
//!
//! [`generate`] a single spec file and writes module trees into the two consumer crates:
//! response types (and their model conversions) into `corepc-types`, the call surface
//! (request structs + method wrappers) into `corepc-client`.

mod codegen;
mod into_model;
mod names;
mod spec;
mod supplement;

use std::fs;
use std::path::Path;
use std::process::Command;

use crate::spec::Spec;

/// Reads the OpenRPC spec at `spec_path` and writes the generated tree across two crates `version` is
/// the short Bitcoin Core version ("30", "28") used in module headers.
pub fn generate(
    spec_path: &Path,
    types_dir: &Path,
    client_dir: &Path,
    version: &str,
) -> Result<Summary, String> {
    let raw =
        fs::read_to_string(spec_path).map_err(|e| format!("read {}: {e}", spec_path.display()))?;
    let mut json: serde_json::Value =
        serde_json::from_str(&raw).map_err(|e| format!("parse {}: {e}", spec_path.display()))?;
    supplement::apply(&mut json);
    let spec: Spec = serde_json::from_value(json)
        .map_err(|e| format!("lower {}: {e}", spec_path.display()))?;

    let _ = fs::remove_dir_all(types_dir);
    let _ = fs::remove_dir_all(client_dir);
    fs::create_dir_all(types_dir).map_err(|e| format!("mkdir {}: {e}", types_dir.display()))?;
    fs::create_dir_all(client_dir).map_err(|e| format!("mkdir {}: {e}", client_dir.display()))?;

    let modules = codegen::lower(&spec);
    let summary = Summary {
        types: modules.types_count(),
        methods: modules.methods_count(),
        option_structs: modules.option_count(),
        out_dir: format!("{} + {}", types_dir.display(), client_dir.display()),
    };
    modules.write(types_dir, client_dir, version)?;
    format_tree(types_dir)?;
    format_tree(client_dir)?;

    println!(
        "[codegen] v{}: {} types, {} methods, {} option structs -> {}",
        version, summary.types, summary.methods, summary.option_structs, summary.out_dir
    );
    Ok(summary)
}

/// Runs `rustfmt` over every generated `.rs` file under `dir` so the emitted code carries
/// the repo's formatting and regeneration never produces style-only diffs.
fn format_tree(dir: &Path) -> Result<(), String> {
    let mut files = Vec::new();
    collect_rs_files(dir, &mut files)?;
    if files.is_empty() {
        return Ok(());
    }
    let status = Command::new("rustfmt")
        .arg("--edition")
        .arg("2021")
        .args(&files)
        .status()
        .map_err(|e| format!("spawn rustfmt: {e}"))?;
    if !status.success() {
        return Err(format!("rustfmt failed on {}", dir.display()));
    }
    Ok(())
}

fn collect_rs_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    Ok(())
}

/// Counts returned by [`generate`] for human-readable feedback.
#[derive(Debug, Clone)]
pub struct Summary {
    pub types: usize,
    pub methods: usize,
    pub option_structs: usize,
    pub out_dir: String,
}
