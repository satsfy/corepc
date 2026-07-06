// SPDX-License-Identifier: CC0-1.0

//! Auto-generated types for Bitcoin Core `31` - hidden.
//!
//! Produced by `codegen`. Do not edit by hand, re-run
//! `just codegen` to regenerate. Holds the RPC return types for this section; the
//! `*Options` request structs live with the call surface in `corepc-client`.

#![allow(non_camel_case_types, non_snake_case, clippy::large_enum_variant)]

mod into;

use serde::{Deserialize, Serialize};

pub use self::into::{
    EstimateRawFeeError, FeerateDiagramEntryError, GenerateBlockError, GenerateError,
    GenerateToAddressError, GenerateToDescriptorError, GetMempoolFeerateDiagramError,
    GetOrphanTxsError, GetOrphanTxsVerboseOneEntryError, GetOrphanTxsVerboseOneError,
    GetOrphanTxsVerboseTwoEntryError, GetOrphanTxsVerboseTwoError, RawFeeDetailError,
    RawFeeRangeError,
};

/// Open an outbound connection to a specified node. This RPC is for testing only.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct AddConnection {
    /// Address of newly added connection.
    pub address: String,
    /// Type of connection opened.
    pub connection_type: String,
}

/// Add the address of a potential peer to an address manager table. This RPC is for testing only.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct AddPeerAddress {
    /// error description, if the address could not be added
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// whether the peer address was successfully added to the address manager table
    pub success: bool,
}

/// Simply echo back the input arguments. This command is for testing.
///
/// It will return an internal bug report when arg9='trigger_internal_bug' is passed.
///
/// The difference between echo and echojson is that echojson has argument conversion enabled in the client-side table in bitcoin-cli and the GUI. There is no server-side difference.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Echo {}

/// Result of the JSON-RPC method `echoipc`.
///
/// > echoipc
/// >
/// > Echo back the input argument, passing it through a spawned process in a multiprocess build.
/// > This command is for testing.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct EchoIpc(pub String);

impl std::ops::Deref for EchoIpc {
    type Target = String;
    fn deref(&self) -> &Self::Target { &self.0 }
}

/// Simply echo back the input arguments. This command is for testing.
///
/// It will return an internal bug report when arg9='trigger_internal_bug' is passed.
///
/// The difference between echo and echojson is that echojson has argument conversion enabled in the client-side table in bitcoin-cli and the GUI. There is no server-side difference.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct EchoJson {}

/// WARNING: This interface is unstable and may disappear or change!
///
/// WARNING: This is an advanced API call that is tightly coupled to the specific
/// implementation of fee estimation. The parameters it can be called with
/// and the results it returns will change if the internal implementation changes.
///
/// Estimates the approximate fee per kilobyte needed for a transaction to begin
/// confirmation within conf_target blocks if possible. Uses virtual transaction size as
/// defined in BIP 141 (witness data is discounted).
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct EstimateRawFee {
    /// estimate for long time horizon
    pub long: EstimateRawFeeLong,
    /// estimate for medium time horizon
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medium: Option<EstimateRawFeeMedium>,
    /// estimate for short time horizon
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short: Option<EstimateRawFeeShort>,
}

/// estimate for long time horizon
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct EstimateRawFeeLong {
    #[serde(flatten)]
    pub extra: std::collections::BTreeMap<String, serde_json::Value>,
}

/// estimate for medium time horizon
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct EstimateRawFeeMedium {
    #[serde(flatten)]
    pub extra: std::collections::BTreeMap<String, serde_json::Value>,
}

/// estimate for short time horizon
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct EstimateRawFeeShort {
    /// exponential decay (per block) for historical moving average of confirmation data
    pub decay: f64,
    /// Errors encountered during processing (if there are any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
    /// information about the highest range of feerates to fail to meet the threshold
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail: Option<EstimateRawFeeShortFail>,
    /// estimate fee rate in BTC/kvB
    #[serde(rename = "feerate", skip_serializing_if = "Option::is_none")]
    pub fee_rate: Option<f64>,
    /// information about the lowest range of feerates to succeed in meeting the threshold
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pass: Option<EstimateRawFeeShortPass>,
    /// The resolution of confirmation targets at this time horizon
    pub scale: i64,
}

/// information about the highest range of feerates to fail to meet the threshold
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct EstimateRawFeeShortFail {
    #[serde(flatten)]
    pub extra: std::collections::BTreeMap<String, serde_json::Value>,
}

/// information about the lowest range of feerates to succeed in meeting the threshold
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct EstimateRawFeeShortPass {
    /// end of feerate range
    pub endrange: f64,
    /// current number of txs in mempool in the feerate range unconfirmed for at least target blocks
    #[serde(rename = "inmempool")]
    pub in_mempool: f64,
    /// number of txs over history horizon in the feerate range that left mempool unconfirmed after target
    #[serde(rename = "leftmempool")]
    pub left_mempool: f64,
    /// start of feerate range
    pub startrange: f64,
    /// number of txs over history horizon in the feerate range that were confirmed at any point
    #[serde(rename = "totalconfirmed")]
    pub total_confirmed: f64,
    /// number of txs over history horizon in the feerate range that were confirmed within target
    #[serde(rename = "withintarget")]
    pub within_target: f64,
}

/// has been replaced by the -generate cli option. Refer to -help for more information.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Generate {}

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

/// Result of the JSON-RPC method `getmempoolfeeratediagram`.
///
/// > getmempoolfeeratediagram
/// >
/// > Returns the feerate diagram for the whole mempool.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct GetMempoolFeeRateDiagram(pub Vec<GetMempoolFeeRateDiagramItem>);

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct GetMempoolFeeRateDiagramItem {
    /// cumulative fee
    pub fee: f64,
    /// cumulative sigops-adjusted weight
    pub weight: i64,
}

/// Shows transactions in the tx orphanage.
///
/// EXPERIMENTAL warning: this call may be changed in future releases.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct GetOrphanTxsVerbose0(pub Vec<String>);

/// Shows transactions in the tx orphanage.
///
/// EXPERIMENTAL warning: this call may be changed in future releases.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct GetOrphanTxsVerbose1(pub Vec<GetOrphanTxsVerbose1Item>);

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct GetOrphanTxsVerbose1Item {
    /// The serialized transaction size in bytes
    pub bytes: u64,
    pub from: Vec<i64>,
    /// The transaction hash in hex
    pub txid: String,
    /// The virtual transaction size as defined in BIP 141. This is different from actual serialized size for witness transactions as witness data is discounted.
    pub vsize: i64,
    /// The transaction weight as defined in BIP 141.
    pub weight: i64,
    /// The transaction witness hash in hex
    pub wtxid: String,
}

/// Shows transactions in the tx orphanage.
///
/// EXPERIMENTAL warning: this call may be changed in future releases.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct GetOrphanTxsVerbose2(pub Vec<GetOrphanTxsVerbose2Item>);

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct GetOrphanTxsVerbose2Item {
    /// The serialized transaction size in bytes
    pub bytes: u64,
    pub from: Vec<i64>,
    /// The serialized, hex-encoded transaction data
    pub hex: String,
    /// The transaction hash in hex
    pub txid: String,
    /// The virtual transaction size as defined in BIP 141. This is different from actual serialized size for witness transactions as witness data is discounted.
    pub vsize: i64,
    /// The transaction weight as defined in BIP 141.
    pub weight: i64,
    /// The transaction witness hash in hex
    pub wtxid: String,
}

/// Result of the JSON-RPC method `getrawaddrman`.
///
/// > getrawaddrman
/// >
/// > EXPERIMENTAL warning: this call may be changed in future releases.
/// >
/// > Returns information on all address manager entries for the new and tried tables.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct GetRawAddrMan(
    /// Map entries
    pub std::collections::BTreeMap<String, std::collections::BTreeMap<String, GetRawAddrManEntry>>,
);

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct GetRawAddrManEntry {
    /// The address of the node
    pub address: String,
    /// Mapped AS (Autonomous System) number at the end of the BGP route to the peer, used for diversifying peer selection (only displayed if the -asmap config option is set)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapped_as: Option<i64>,
    /// The network (ipv4, ipv6, onion, i2p, cjdns) of the address
    pub network: String,
    /// The port number of the node
    pub port: i64,
    /// The services offered by the node
    pub services: u64,
    /// The address that relayed the address to us
    pub source: String,
    /// Mapped AS (Autonomous System) number at the end of the BGP route to the source, used for diversifying peer selection (only displayed if the -asmap config option is set)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_mapped_as: Option<i64>,
    /// The network (ipv4, ipv6, onion, i2p, cjdns) of the source address
    pub source_network: String,
    /// The UNIX epoch time when the node was last seen
    pub time: i64,
}

/// Send a p2p message to a peer specified by id.
/// The message type and body must be provided, the message header will be generated.
/// This RPC is for testing only.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct SendmsgToPeer {}
