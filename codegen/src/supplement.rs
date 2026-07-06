// SPDX-License-Identifier: CC0-1.0

//! Codegen-owned additions layered onto the pristine Core OpenRPC dumps.
//!
//! The spec files under `specs/` are byte-for-byte as produced by Core and must never be
//! edited. Core's dump omits the `generating` RPCs (regtest-only) and models a couple of
//! results in shapes the generator cannot express, so the generator patches the parsed
//! JSON here before lowering.

use serde_json::Value;

/// RPCs missing from Core's OpenRPC dump but required by the generated client surface.
const SUPPLEMENT_METHODS: &str = include_str!("supplement_methods.json");

/// Merges the supplement into a parsed spec document, in place.
///
/// - Appends each supplement method that is not already present by name.
/// - Flattens the `deriveaddresses` result: the dump models it as a `oneOf` over
///   `string[]` (single derivation) and `string[][]` (multipath), which the generator
///   cannot express; the flat `string[]` shape is what the client consumes.
pub fn apply(spec: &mut Value) {
    let Some(methods) = spec.get_mut("methods").and_then(Value::as_array_mut) else {
        return;
    };

    let extra: Vec<Value> =
        serde_json::from_str(SUPPLEMENT_METHODS).expect("supplement_methods.json parses");
    for method in extra {
        let name = method.get("name").and_then(Value::as_str).expect("supplement method name");
        let present = methods
            .iter()
            .any(|m| m.get("name").and_then(Value::as_str) == Some(name));
        if !present {
            methods.push(method);
        }
    }

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
