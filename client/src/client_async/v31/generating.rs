// SPDX-License-Identifier: CC0-1.0

//! Auto-generated method wrappers for Bitcoin Core `31` - generating.
//!
//! Produced by `codegen`. Do not edit by hand, re-run
//! `just codegen` to regenerate. Defines the `*Options` request structs these methods
//! consume; the response types live in the `corepc-types` crate
//! (`types::v31::generated`).

#![allow(unused_imports, clippy::needless_pass_by_value, clippy::too_many_arguments)]

use serde::{Deserialize, Serialize};
use serde_json::json;

use types::v31::generated::{GenerateBlock, GenerateToAddress, GenerateToDescriptor};

use crate::client_async::error::Result;
use crate::client_async::Client;

/// Optional parameters for the `generateblock` JSON-RPC method (consumed by `Client::generate_block_with`).
#[derive(Clone, Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateBlockOptions {
    /// Whether to submit the block before the RPC call returns or to return it as hex.
    pub submit: Option<bool>,
}

/// Optional parameters for the `generatetoaddress` JSON-RPC method (consumed by `Client::generate_to_address_with`).
#[derive(Clone, Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateToAddressOptions {
    /// How many iterations to try.
    pub maxtries: Option<f64>,
}

/// Optional parameters for the `generatetodescriptor` JSON-RPC method (consumed by `Client::generate_to_descriptor_with`).
#[derive(Clone, Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateToDescriptorOptions {
    /// How many iterations to try.
    pub maxtries: Option<f64>,
}

impl Client {
    /// `generateblock` with required arguments only.
    ///
    /// Mine a set of ordered transactions to a specified address or descriptor and return the block hash.
    pub async fn generate_block(
        &self,
        output: String,
        transactions: Vec<String>,
    ) -> Result<GenerateBlock> {
        self.call_raw("generateblock", &[json!(output), json!(transactions)]).await
    }

    /// `generateblock` with all optional arguments via [`GenerateBlockOptions`].
    ///
    /// Mine a set of ordered transactions to a specified address or descriptor and return the block hash.
    pub async fn generate_block_with(
        &self,
        output: String,
        transactions: Vec<String>,
        opts: GenerateBlockOptions,
    ) -> Result<GenerateBlock> {
        self.call_raw("generateblock", &[json!(output), json!(transactions), json!(opts.submit)])
            .await
    }

    /// `generatetoaddress` with required arguments only.
    ///
    /// Mine to a specified address and return the block hashes.
    pub async fn generate_to_address(
        &self,
        n_blocks: i64,
        address: String,
    ) -> Result<GenerateToAddress> {
        self.call_raw("generatetoaddress", &[json!(n_blocks), json!(address)]).await
    }

    /// `generatetoaddress` with all optional arguments via [`GenerateToAddressOptions`].
    ///
    /// Mine to a specified address and return the block hashes.
    pub async fn generate_to_address_with(
        &self,
        n_blocks: i64,
        address: String,
        opts: GenerateToAddressOptions,
    ) -> Result<GenerateToAddress> {
        self.call_raw("generatetoaddress", &[json!(n_blocks), json!(address), json!(opts.maxtries)])
            .await
    }

    /// `generatetodescriptor` with required arguments only.
    ///
    /// Mine to a specified descriptor and return the block hashes.
    pub async fn generate_to_descriptor(
        &self,
        num_blocks: i64,
        descriptor: String,
    ) -> Result<GenerateToDescriptor> {
        self.call_raw("generatetodescriptor", &[json!(num_blocks), json!(descriptor)]).await
    }

    /// `generatetodescriptor` with all optional arguments via [`GenerateToDescriptorOptions`].
    ///
    /// Mine to a specified descriptor and return the block hashes.
    pub async fn generate_to_descriptor_with(
        &self,
        num_blocks: i64,
        descriptor: String,
        opts: GenerateToDescriptorOptions,
    ) -> Result<GenerateToDescriptor> {
        self.call_raw(
            "generatetodescriptor",
            &[json!(num_blocks), json!(descriptor), json!(opts.maxtries)],
        )
        .await
    }

    /// `invalidateblock` with required arguments only.
    ///
    /// Permanently marks a block as invalid, as if it violated a consensus rule.
    pub async fn invalidate_block(&self, block_hash: String) -> Result<()> {
        self.call_raw("invalidateblock", &[json!(block_hash)]).await
    }
}
