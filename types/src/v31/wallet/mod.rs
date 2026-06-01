// SPDX-License-Identifier: CC0-1.0

//! The JSON-RPC API for Bitcoin Core `v31` - wallet.
//!
//! Types for methods found under the `== Wallet ==` section of the API docs.

mod into;

use serde::{Deserialize, Serialize};

/// Result of the JSON-RPC method `getwalletinfo`.
///
/// > getwalletinfo
/// >
/// > Returns an object containing various wallet state info.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde-deny-unknown-fields", serde(deny_unknown_fields))]
pub struct GetWalletInfo {
    /// The wallet name
    #[serde(rename = "walletname")]
    pub wallet_name: String,
    /// Only related to unsupported legacy wallet, returns the latest version 169900 for backwards compatibility. (DEPRECATED)
    #[serde(rename = "walletversion")]
    pub wallet_version: i64,
    /// The database format (only sqlite)
    pub format: String,
    /// The total number of transactions in the wallet
    #[serde(rename = "txcount")]
    pub tx_count: i64,
    /// How many new keys are pre-generated (only counts external keys)
    #[serde(rename = "keypoolsize")]
    pub keypool_size: i64,
    /// How many new keys are pre-generated for internal use (used for change outputs, only appears if the wallet is using this feature, otherwise external keys are used)
    #[serde(rename = "keypoolsize_hd_internal")]
    pub keypool_size_hd_internal: Option<i64>,
    /// The UNIX epoch time until which the wallet is unlocked for transfers, or 0 if the wallet is locked (only present for passphrase-encrypted wallets)
    pub unlocked_until: Option<u32>,
    /// False if privatekeys are disabled for this wallet (enforced watch-only wallet)
    pub private_keys_enabled: bool,
    /// Whether this wallet tracks clean/dirty coins in terms of reuse
    pub avoid_reuse: bool,
    /// Current scanning details, or false if no scan is in progress
    pub scanning: crate::v30::GetWalletInfoScanning,
    /// Whether this wallet uses descriptors for output script management
    pub descriptors: bool,
    /// Whether this wallet is configured to use an external signer such as a hardware wallet
    pub external_signer: bool,
    /// Whether this wallet intentionally does not contain any keys, scripts, or descriptors
    pub blank: bool,
    /// The start time for blocks scanning. It could be modified by (re)importing any descriptor with an earlier timestamp.
    pub birthtime: Option<u32>,
    /// The flags currently set on the wallet.
    pub flags: Vec<String>,
    /// Hash and height of the block this information was generated on
    #[serde(rename = "lastprocessedblock")]
    pub last_processed_block: Option<crate::v30::LastProcessedBlock>,
}
