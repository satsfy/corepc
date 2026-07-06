// SPDX-License-Identifier: CC0-1.0

//! Generate Rust bindings for the Bitcoin Core JSON-RPC API from its OpenRPC spec.
//!
//! Usage:
//!
//! ```text
//! btc-codegen <version>      # For example, `30`, regenerates one version
//! btc-codegen all            # Regenerates every spec under specs/
//! ```
//!
//! Output is split across the two sibling crates: the response types and their model
//! conversions go to `../types/src/v{N}/generated/` and the call surface (request structs +
//! method wrappers) to `../client/src/client_async/v{N}/` (relative to the manifest dir).

use std::path::{Path, PathBuf};
use std::{env, fs, process};

use btc_codegen::generate;

/// Parse the version argument and regenerate the bindings for each selected version.
fn main() {
    let mut args = env::args().skip(1);
    let version = args.next().unwrap_or_else(|| {
        eprintln!("usage: btc-codegen <version|all>");
        process::exit(2)
    });

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let specs_dir = manifest_dir.join("specs");
    let types_root = manifest_dir.join("../types/src");
    let client_async_root = manifest_dir.join("../client/src/client_async");

    let versions = if version == "all" {
        list_versions(&specs_dir).unwrap_or_else(|e| exit_with_msg(&e))
    } else {
        vec![version]
    };

    if versions.is_empty() {
        exit_with_msg(&format!("no specs found under {}", specs_dir.display()));
    }

    for v in versions {
        let spec_path = match find_spec(&specs_dir, &v) {
            Some(p) => p,
            None => {
                eprintln!("[codegen] no spec for v{v}, skipping");
                continue;
            }
        };
        let types_dir = types_root.join(format!("v{v}/generated"));
        let client_dir = client_async_root.join(format!("v{v}"));
        if let Err(e) = generate(&spec_path, &types_dir, &client_dir, &v) {
            exit_with_msg(&format!("[codegen] v{v}: {e}"));
        }
    }
}

/// Print `msg` to stderr and exit with a failure code.
fn exit_with_msg(msg: &str) -> ! {
    eprintln!("{msg}");
    process::exit(1)
}

/// The major version of a spec file named `v{X}_{Y}_{Z}_openrpc.json`,
/// so `v17_2_0_openrpc.json` is major `17`.
fn spec_major(file_name: &str) -> Option<&str> {
    let rest = file_name.strip_prefix('v')?.strip_suffix("_openrpc.json")?;
    rest.split('_').next()
}

/// Every spec file under `specs/` (see `specs/README.md`).
fn spec_files(specs_dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(specs_dir) else { return out };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if spec_major(name).is_some() {
            out.push(entry.path());
        }
    }
    out.sort();
    out
}

/// Find the spec file for one major version, the lowest match if several.
fn find_spec(specs_dir: &Path, version: &str) -> Option<PathBuf> {
    spec_files(specs_dir)
        .into_iter()
        .find(|p| p.file_name().and_then(|n| n.to_str()).and_then(spec_major) == Some(version))
}

/// List the distinct major versions that have a spec file under `specs_dir`.
fn list_versions(specs_dir: &Path) -> Result<Vec<String>, String> {
    let mut out: Vec<String> = Vec::new();
    for path in spec_files(specs_dir) {
        let Some(major) = path.file_name().and_then(|n| n.to_str()).and_then(spec_major) else {
            continue;
        };
        if !out.iter().any(|v| v == major) {
            out.push(major.to_owned());
        }
    }
    out.sort_by_key(|v| v.parse::<u32>().unwrap_or(0));
    Ok(out)
}
