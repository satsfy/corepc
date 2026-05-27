// SPDX-License-Identifier: CC0-1.0

//! The JSON-RPC API for Bitcoin Core `v31` - blockchain.
//!
//! Types for methods found under the `== Blockchain ==` section of the API docs.

mod into;

pub use self::into::GetMempoolClusterError;

use serde::{Deserialize, Serialize};

/// Result of JSON-RPC method `getmempoolcluster`.
///
/// > getmempoolcluster "txid"
/// >
/// > Returns mempool data for given cluster
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct GetMempoolCluster {
    /// Total sigops-adjusted weight (as defined in BIP 141 and modified by `-bytespersigop`).
    #[serde(rename = "clusterweight")]
    pub cluster_weight: u64,
    /// Number of transactions.
    #[serde(rename = "txcount")]
    pub tx_count: u64,
    /// Chunks in this cluster (in mining order).
    pub chunks: Vec<Chunk>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Chunk {
    /// Fees of the transactions in this chunk, in BTC.
    #[serde(rename = "chunkfee")]
    pub chunk_fee: f64,
    /// Sigops-adjusted weight of all transactions in this chunk.
    #[serde(rename = "chunkweight")]
    pub chunk_weight: u64,
    /// Transactions in this chunk in mining order (txids).
    pub txs: Vec<String>,
}
