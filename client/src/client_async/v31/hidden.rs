// SPDX-License-Identifier: CC0-1.0

//! Auto-generated method wrappers for Bitcoin Core `31` - hidden.
//!
//! Produced by `codegen`. Do not edit by hand, re-run
//! `just codegen` to regenerate. Defines the `*Options` request structs these methods
//! consume; the response types live in the `corepc-types` crate
//! (`types::v31::generated`).

#![allow(unused_imports, clippy::needless_pass_by_value, clippy::too_many_arguments)]

use serde::{Deserialize, Serialize};
use serde_json::json;
use types::v31::generated::{
    AddConnection, AddPeerAddress, Echo, EchoIpc, EchoJson, EstimateRawFee, Generate,
    GenerateBlock, GenerateToAddress, GenerateToDescriptor, GetMempoolFeeRateDiagram,
    GetOrphanTxsVerbose0, GetOrphanTxsVerbose1, GetOrphanTxsVerbose2, GetRawAddrMan, SendmsgToPeer,
};

use crate::client_async::error::Result;
use crate::client_async::Client;

/// Optional parameters for the `addpeeraddress` JSON-RPC method (consumed by `Client::add_peer_address_with`).
#[derive(Clone, Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddPeerAddressOptions {
    /// If true, attempt to add the peer to the tried addresses table
    ///
    /// Default in Bitcoin Core: `false`.
    pub tried: Option<bool>,
}

/// Optional parameters for the `echo` JSON-RPC method (consumed by `Client::echo_with`).
#[derive(Clone, Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EchoOptions {
    pub arg0: Option<String>,
    pub arg1: Option<String>,
    pub arg2: Option<String>,
    pub arg3: Option<String>,
    pub arg4: Option<String>,
    pub arg5: Option<String>,
    pub arg6: Option<String>,
    pub arg7: Option<String>,
    pub arg8: Option<String>,
    pub arg9: Option<String>,
}

/// Optional parameters for the `echojson` JSON-RPC method (consumed by `Client::echo_json_with`).
#[derive(Clone, Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EchoJsonOptions {
    pub arg0: Option<String>,
    pub arg1: Option<String>,
    pub arg2: Option<String>,
    pub arg3: Option<String>,
    pub arg4: Option<String>,
    pub arg5: Option<String>,
    pub arg6: Option<String>,
    pub arg7: Option<String>,
    pub arg8: Option<String>,
    pub arg9: Option<String>,
}

/// Optional parameters for the `estimaterawfee` JSON-RPC method (consumed by `Client::estimate_raw_fee_with`).
#[derive(Clone, Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstimateRawFeeOptions {
    /// The proportion of transactions in a given feerate range that must have been
    /// confirmed within conf_target in order to consider those feerates as high enough and proceed to check
    /// lower buckets.
    ///
    /// Default in Bitcoin Core: `0.95`.
    pub threshold: Option<f64>,
}

/// Optional parameters for the `generateblock` JSON-RPC method (consumed by `Client::generate_block_with`).
#[derive(Clone, Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateBlockOptions {
    /// Whether to submit the block before the RPC call returns or to return it as hex.
    ///
    /// Default in Bitcoin Core: `true`.
    pub submit: Option<bool>,
}

/// Optional parameters for the `generatetoaddress` JSON-RPC method (consumed by `Client::generate_to_address_with`).
#[derive(Clone, Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateToAddressOptions {
    /// How many iterations to try.
    ///
    /// Default in Bitcoin Core: `1000000`.
    pub maxtries: Option<i64>,
}

/// Optional parameters for the `generatetodescriptor` JSON-RPC method (consumed by `Client::generate_to_descriptor_with`).
#[derive(Clone, Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateToDescriptorOptions {
    /// How many iterations to try.
    ///
    /// Default in Bitcoin Core: `1000000`.
    pub maxtries: Option<i64>,
}

impl Client {
    /// `addconnection` with required arguments only.
    ///
    /// Open an outbound connection to a specified node. This RPC is for testing only.
    pub async fn add_connection(
        &self,
        address: String,
        connection_type: String,
        v2transport: bool,
    ) -> Result<AddConnection> {
        self.call_raw(
            "addconnection",
            &[json!(address), json!(connection_type), json!(v2transport)],
        )
        .await
    }

    /// `addpeeraddress` with required arguments only.
    ///
    /// Add the address of a potential peer to an address manager table. This RPC is for testing only.
    pub async fn add_peer_address(&self, address: String, port: i64) -> Result<AddPeerAddress> {
        self.call_raw("addpeeraddress", &[json!(address), json!(port)]).await
    }

    /// `addpeeraddress` with all optional arguments via [`AddPeerAddressOptions`].
    ///
    /// Add the address of a potential peer to an address manager table. This RPC is for testing only.
    pub async fn add_peer_address_with(
        &self,
        address: String,
        port: i64,
        opts: AddPeerAddressOptions,
    ) -> Result<AddPeerAddress> {
        self.call_raw("addpeeraddress", &[json!(address), json!(port), json!(opts.tried)]).await
    }

    /// `echo` with required arguments only.
    ///
    /// Simply echo back the input arguments. This command is for testing.
    ///
    /// It will return an internal bug report when arg9='trigger_internal_bug' is passed.
    ///
    /// The difference between echo and echojson is that echojson has argument conversion enabled in the client-side table in bitcoin-cli and the GUI. There is no server-side difference.
    pub async fn echo(&self) -> Result<Echo> { self.call_raw("echo", &[(); 0] as &[()]).await }

    /// `echo` with all optional arguments via [`EchoOptions`].
    ///
    /// Simply echo back the input arguments. This command is for testing.
    ///
    /// It will return an internal bug report when arg9='trigger_internal_bug' is passed.
    ///
    /// The difference between echo and echojson is that echojson has argument conversion enabled in the client-side table in bitcoin-cli and the GUI. There is no server-side difference.
    pub async fn echo_with(&self, opts: EchoOptions) -> Result<Echo> {
        self.call_raw(
            "echo",
            &[
                json!(opts.arg0),
                json!(opts.arg1),
                json!(opts.arg2),
                json!(opts.arg3),
                json!(opts.arg4),
                json!(opts.arg5),
                json!(opts.arg6),
                json!(opts.arg7),
                json!(opts.arg8),
                json!(opts.arg9),
            ],
        )
        .await
    }

    /// `echoipc` with required arguments only.
    ///
    /// Echo back the input argument, passing it through a spawned process in a multiprocess build.
    /// This command is for testing.
    pub async fn echo_ipc(&self, arg: String) -> Result<EchoIpc> {
        self.call_raw("echoipc", &[json!(arg)]).await
    }

    /// `echojson` with required arguments only.
    ///
    /// Simply echo back the input arguments. This command is for testing.
    ///
    /// It will return an internal bug report when arg9='trigger_internal_bug' is passed.
    ///
    /// The difference between echo and echojson is that echojson has argument conversion enabled in the client-side table in bitcoin-cli and the GUI. There is no server-side difference.
    pub async fn echo_json(&self) -> Result<EchoJson> {
        self.call_raw("echojson", &[(); 0] as &[()]).await
    }

    /// `echojson` with all optional arguments via [`EchoJsonOptions`].
    ///
    /// Simply echo back the input arguments. This command is for testing.
    ///
    /// It will return an internal bug report when arg9='trigger_internal_bug' is passed.
    ///
    /// The difference between echo and echojson is that echojson has argument conversion enabled in the client-side table in bitcoin-cli and the GUI. There is no server-side difference.
    pub async fn echo_json_with(&self, opts: EchoJsonOptions) -> Result<EchoJson> {
        self.call_raw(
            "echojson",
            &[
                json!(opts.arg0),
                json!(opts.arg1),
                json!(opts.arg2),
                json!(opts.arg3),
                json!(opts.arg4),
                json!(opts.arg5),
                json!(opts.arg6),
                json!(opts.arg7),
                json!(opts.arg8),
                json!(opts.arg9),
            ],
        )
        .await
    }

    /// `estimaterawfee` with required arguments only.
    ///
    /// WARNING: This interface is unstable and may disappear or change!
    ///
    /// WARNING: This is an advanced API call that is tightly coupled to the specific
    /// implementation of fee estimation. The parameters it can be called with
    /// and the results it returns will change if the internal implementation changes.
    ///
    /// Estimates the approximate fee per kilobyte needed for a transaction to begin
    /// confirmation within conf_target blocks if possible. Uses virtual transaction size as
    /// defined in BIP 141 (witness data is discounted).
    pub async fn estimate_raw_fee(&self, conf_target: i64) -> Result<EstimateRawFee> {
        self.call_raw("estimaterawfee", &[json!(conf_target)]).await
    }

    /// `estimaterawfee` with all optional arguments via [`EstimateRawFeeOptions`].
    ///
    /// WARNING: This interface is unstable and may disappear or change!
    ///
    /// WARNING: This is an advanced API call that is tightly coupled to the specific
    /// implementation of fee estimation. The parameters it can be called with
    /// and the results it returns will change if the internal implementation changes.
    ///
    /// Estimates the approximate fee per kilobyte needed for a transaction to begin
    /// confirmation within conf_target blocks if possible. Uses virtual transaction size as
    /// defined in BIP 141 (witness data is discounted).
    pub async fn estimate_raw_fee_with(
        &self,
        conf_target: i64,
        opts: EstimateRawFeeOptions,
    ) -> Result<EstimateRawFee> {
        self.call_raw("estimaterawfee", &[json!(conf_target), json!(opts.threshold)]).await
    }

    /// `generate` with required arguments only.
    ///
    /// has been replaced by the -generate cli option. Refer to -help for more information.
    pub async fn generate(&self) -> Result<Generate> {
        self.call_raw("generate", &[(); 0] as &[()]).await
    }

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

    /// `getmempoolfeeratediagram` with required arguments only.
    ///
    /// Returns the feerate diagram for the whole mempool.
    pub async fn get_mempool_fee_rate_diagram(&self) -> Result<GetMempoolFeeRateDiagram> {
        self.call_raw("getmempoolfeeratediagram", &[(); 0] as &[()]).await
    }

    /// `getorphantxs` with the result selected for verbosity `0`.
    ///
    /// Shows transactions in the tx orphanage.
    ///
    /// EXPERIMENTAL warning: this call may be changed in future releases.
    pub async fn get_orphan_txs_verbose_0(&self) -> Result<GetOrphanTxsVerbose0> {
        self.call_raw("getorphantxs", &[json!(0)]).await
    }

    /// `getorphantxs` with the result selected for verbosity `1`.
    ///
    /// Shows transactions in the tx orphanage.
    ///
    /// EXPERIMENTAL warning: this call may be changed in future releases.
    pub async fn get_orphan_txs_verbose_1(&self) -> Result<GetOrphanTxsVerbose1> {
        self.call_raw("getorphantxs", &[json!(1)]).await
    }

    /// `getorphantxs` with the result selected for verbosity `2`.
    ///
    /// Shows transactions in the tx orphanage.
    ///
    /// EXPERIMENTAL warning: this call may be changed in future releases.
    pub async fn get_orphan_txs_verbose_2(&self) -> Result<GetOrphanTxsVerbose2> {
        self.call_raw("getorphantxs", &[json!(2)]).await
    }

    /// `getrawaddrman` with required arguments only.
    ///
    /// EXPERIMENTAL warning: this call may be changed in future releases.
    ///
    /// Returns information on all address manager entries for the new and tried tables.
    pub async fn get_raw_addr_man(&self) -> Result<GetRawAddrMan> {
        self.call_raw("getrawaddrman", &[(); 0] as &[()]).await
    }

    /// `invalidateblock` with required arguments only.
    ///
    /// Permanently marks a block as invalid, as if it violated a consensus rule.
    pub async fn invalidate_block(&self, block_hash: String) -> Result<()> {
        self.call_raw("invalidateblock", &[json!(block_hash)]).await
    }

    /// `mockscheduler` with required arguments only.
    ///
    /// Bump the scheduler into the future (-regtest only)
    pub async fn mock_scheduler(&self, delta_time: i64) -> Result<()> {
        self.call_raw("mockscheduler", &[json!(delta_time)]).await
    }

    /// `reconsiderblock` with required arguments only.
    ///
    /// Removes invalidity status of a block, its ancestors and its descendants, reconsider them for activation.
    /// This can be used to undo the effects of invalidateblock.
    pub async fn reconsider_block(&self, block_hash: String) -> Result<()> {
        self.call_raw("reconsiderblock", &[json!(block_hash)]).await
    }

    /// `sendmsgtopeer` with required arguments only.
    ///
    /// Send a p2p message to a peer specified by id.
    /// The message type and body must be provided, the message header will be generated.
    /// This RPC is for testing only.
    pub async fn sendmsg_to_peer(
        &self,
        peer_id: i64,
        msg_type: String,
        msg: String,
    ) -> Result<SendmsgToPeer> {
        self.call_raw("sendmsgtopeer", &[json!(peer_id), json!(msg_type), json!(msg)]).await
    }

    /// `setmocktime` with required arguments only.
    ///
    /// Set the local time to given timestamp (-regtest only)
    pub async fn set_mock_time(&self, timestamp: i64) -> Result<()> {
        self.call_raw("setmocktime", &[json!(timestamp)]).await
    }

    /// `syncwithvalidationinterfacequeue` with required arguments only.
    ///
    /// Waits for the validation interface queue to catch up on everything that was there when we entered this function.
    pub async fn sync_with_validation_interface_queue(&self) -> Result<()> {
        self.call_raw("syncwithvalidationinterfacequeue", &[(); 0] as &[()]).await
    }
}
