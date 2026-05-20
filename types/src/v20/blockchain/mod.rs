// SPDX-License-Identifier: CC0-1.0

//! The JSON-RPC API for Bitcoin Core `v0.20` - blockchain.
//!
//! Types for methods found under the `== Blockchain ==` section of the API docs.

mod into;

use serde::{Deserialize, Serialize};

pub use crate::v17::GetBlockStatsError;

/// Result of JSON-RPC method `getblockstats`.
///
/// > getblockstats hash_or_height ( stats )
/// >
/// > Compute per block statistics for a given window. All amounts are in satoshis.
/// > It won't work for some heights with pruning.
/// > It won't work without -txindex for utxo_size_inc, *fee or *feerate stats.
/// >
/// > Arguments:
/// > 1. "hash_or_height"     (string or numeric, required) The block hash or height of the target block
/// > 2. "stats"              (json array, optional, default=all values) Values to plot (see result below)
/// >    [
/// >        "height",     (string) Selected statistic
/// >        "time",       (string) Selected statistic
/// >        ...
/// >    ]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct GetBlockStats {
    /// Average fee in the block.
    #[serde(rename = "avgfee")]
    pub average_fee: Option<u64>,
    // FIXME: Remember these docs will become silently stale when unit changes in a later version of Core.
    /// Average feerate (in satoshis per virtual byte).
    #[serde(rename = "avgfeerate")]
    pub average_fee_rate: Option<u64>,
    /// Average transaction size.
    #[serde(rename = "avgtxsize")]
    pub average_tx_size: Option<i64>,
    /// The block hash (to check for potential reorgs).
    #[serde(rename = "blockhash")]
    pub block_hash: Option<String>,
    /// Feerates at the 10th, 25th, 50th, 75th, and 90th percentile weight unit (in satoshis per
    /// virtual byte).
    #[serde(rename = "feerate_percentiles")]
    pub fee_rate_percentiles: Option<[u64; 5]>,
    /// The height of the block.
    pub height: Option<i64>,
    /// The number of inputs (excluding coinbase).
    #[serde(rename = "ins")]
    pub inputs: Option<i64>,
    /// Maximum fee in the block.
    #[serde(rename = "maxfee")]
    pub max_fee: Option<u64>,
    /// Maximum feerate (in satoshis per virtual byte).
    #[serde(rename = "maxfeerate")]
    pub max_fee_rate: Option<u64>,
    /// Maximum transaction size.
    #[serde(rename = "maxtxsize")]
    pub max_tx_size: Option<i64>,
    /// Truncated median fee in the block.
    #[serde(rename = "medianfee")]
    pub median_fee: Option<u64>,
    /// The block median time past.
    #[serde(rename = "mediantime")]
    pub median_time: Option<i64>,
    /// Truncated median transaction size.
    #[serde(rename = "mediantxsize")]
    pub median_tx_size: Option<i64>,
    /// Minimum fee in the block.
    #[serde(rename = "minfee")]
    pub minimum_fee: Option<u64>,
    /// Minimum feerate (in satoshis per virtual byte).
    #[serde(rename = "minfeerate")]
    pub minimum_fee_rate: Option<u64>,
    /// Minimum transaction size.
    #[serde(rename = "mintxsize")]
    pub minimum_tx_size: Option<i64>,
    /// The number of outputs.
    #[serde(rename = "outs")]
    pub outputs: Option<i64>,
    /// The block subsidy.
    pub subsidy: Option<u64>,
    /// Total size of all segwit transactions.
    #[serde(rename = "swtotal_size")]
    pub segwit_total_size: Option<i64>,
    /// Total weight of all segwit transactions divided by segwit scale factor (4).
    #[serde(rename = "swtotal_weight")]
    pub segwit_total_weight: Option<u64>,
    /// The number of segwit transactions.
    #[serde(rename = "swtxs")]
    pub segwit_txs: Option<i64>,
    /// The block time.
    pub time: Option<i64>,
    /// Total amount in all outputs (excluding coinbase and thus reward [ie subsidy + totalfee]).
    pub total_out: Option<u64>,
    /// Total size of all non-coinbase transactions.
    pub total_size: Option<i64>,
    /// Total weight of all non-coinbase transactions divided by segwit scale factor (4).
    pub total_weight: Option<u64>,
    /// The fee total.
    #[serde(rename = "totalfee")]
    pub total_fee: Option<u64>,
    /// The number of transactions (including coinbase).
    pub txs: Option<i64>,
    /// The increase/decrease in the number of unspent outputs.
    pub utxo_increase: Option<i32>,
    /// The increase/decrease in size for the utxo index (not discounting op_return and similar).
    #[serde(rename = "utxo_size_inc")]
    pub utxo_size_increase: Option<i32>,
}
