// SPDX-License-Identifier: CC0-1.0

//! Codegen-owned additions layered onto the pristine Core OpenRPC dumps.
//!
//! The spec files under `specs/` are byte-for-byte as produced by Core and must never be
//! edited. Core models the `deriveaddresses` result in a shape the generator cannot express,
//! so the generator patches the parsed JSON here before lowering.

use serde_json::Value;

/// Applies codegen-owned fixups to a parsed spec document, in place.
///
/// - Flattens the `deriveaddresses` result: the dump models it as a `oneOf` over
///   `string[]` (single derivation) and `string[][]` (multipath), which the generator
///   cannot express; the flat `string[]` shape is what the client consumes.
pub fn apply(spec: &mut Value) {
    let Some(methods) = spec.get_mut("methods").and_then(Value::as_array_mut) else {
        return;
    };

    for method in methods.iter_mut() {
        if method.get("name").and_then(Value::as_str) != Some("deriveaddresses") {
            continue;
        }
        let Some(schema) = method.pointer_mut("/result/schema") else { continue };
        if schema.get("oneOf").is_some() {
            *schema = serde_json::json!({
                "type": "array",
                "items": { "type": "string" },
                "description": "the derived addresses"
            });
        }
    }
}
