// SPDX-License-Identifier: CC0-1.0

//! Auto-generated types for Bitcoin Core `31` - generating.
//!
//! Produced by `codegen`. Do not edit by hand, re-run
//! `just codegen` to regenerate. Holds the RPC return types for this section; the
//! `*Options` request structs live with the call surface in `corepc-client`.

#![allow(non_camel_case_types, non_snake_case, clippy::large_enum_variant)]

mod into;

pub use self::into::{GenerateBlockError, GenerateToAddressError, GenerateToDescriptorError};

use serde::{Deserialize, Serialize};

/// Mine a set of ordered transactions to a specified address or descriptor and return the block hash.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct GenerateBlock {
    /// hash of generated block
    pub hash: String,
    /// hex of generated block, only present when submit=false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hex: Option<String>,
}

/// Result of the JSON-RPC method `generatetoaddress`.
///
/// > generatetoaddress
/// >
/// > Mine to a specified address and return the block hashes.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct GenerateToAddress(pub Vec<String>);

/// Result of the JSON-RPC method `generatetodescriptor`.
///
/// > generatetodescriptor
/// >
/// > Mine to a specified descriptor and return the block hashes.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct GenerateToDescriptor(pub Vec<String>);
