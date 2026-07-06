// SPDX-License-Identifier: CC0-1.0

//! Codegen-owned additions layered onto the pristine Core OpenRPC dumps.
//!
//! The spec files under `specs/` are byte-for-byte as produced by Core and must never be
//! edited. A few results come out of the dump in shapes the generator cannot express (or
//! with recursively-reused schemas flattened to empty objects), so the generator patches
//! the parsed JSON here before lowering.

use serde_json::Value;

/// Applies codegen-owned fixups to a parsed spec document, in place.
///
/// - Flattens the `deriveaddresses` result: the dump models it as a `oneOf` over
///   `string[]` (single derivation) and `string[][]` (multipath), which the generator
///   cannot express; the flat `string[]` shape is what the client consumes.
/// - Fleshes out `estimaterawfee`: the dump only details the `short` horizon and leaves
///   `short.fail`, `medium` and `long` as empty objects ("same as pass/short" in Core's
///   docs); all horizons share one shape, so mirror `pass` onto `fail` and `short` onto
///   the other horizons.
/// - Reshapes `getrawaddrman`: the dump models the two address-manager tables as a generic
///   map-of-maps; Core only ever returns `new` and `tried`, and the tests (and curated
///   type) read them as named fields.
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

    for method in methods.iter_mut() {
        if method.get("name").and_then(Value::as_str) != Some("estimaterawfee") {
            continue;
        }
        let Some(horizons) =
            method.pointer_mut("/result/schema/properties").and_then(Value::as_object_mut)
        else {
            continue;
        };
        let empty =
            |v: &Value| v.get("properties").and_then(Value::as_object).is_none_or(|p| p.is_empty());
        let with_desc = |mut schema: Value, from: &Value| {
            if let (Some(d), Some(obj)) = (from.get("description").cloned(), schema.as_object_mut())
            {
                obj.insert("description".to_owned(), d);
            }
            schema
        };
        if let Some(short) = horizons.get_mut("short") {
            if let Some(props) = short.pointer_mut("/properties").and_then(Value::as_object_mut) {
                if let Some(pass) = props.get("pass").cloned() {
                    if let Some(fail) = props.get_mut("fail") {
                        if empty(fail) && pass.get("properties").is_some() {
                            *fail = with_desc(pass, fail);
                        }
                    }
                }
            }
        }
        let Some(short) = horizons.get("short").cloned() else { continue };
        if short.get("properties").is_some() {
            for name in ["medium", "long"] {
                if let Some(horizon) = horizons.get_mut(name) {
                    if empty(horizon) {
                        *horizon = with_desc(short.clone(), horizon);
                    }
                }
            }
        }
    }

    for method in methods.iter_mut() {
        if method.get("name").and_then(Value::as_str) != Some("getrawaddrman") {
            continue;
        }
        let Some(schema) = method.pointer_mut("/result/schema") else { continue };
        let generic_table = schema.get("additionalProperties").cloned();
        if let (Some(table), None) = (generic_table, schema.get("properties")) {
            let describe = |desc: &str| {
                let mut t = table.clone();
                if let Some(obj) = t.as_object_mut() {
                    obj.insert("description".to_owned(), Value::String(desc.to_owned()));
                }
                t
            };
            *schema = serde_json::json!({
                "type": "object",
                "properties": {
                    "new": describe("addresses in the \"new\" table, keyed by bucket/position"),
                    "tried": describe("addresses in the \"tried\" table, keyed by bucket/position"),
                },
                "additionalProperties": false,
                "required": ["new", "tried"],
            });
        }
    }
}
