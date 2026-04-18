// SPDX-License-Identifier: CC0-1.0

//! Tests derived from Bitcoin Core's `rpc_generate.py`, mining_basic.py,
//! and `rpc_getblockstats.py` test vectors.
//!
//! Exercises mining and generating RPCs with deeper field validation,
//! mirroring assertions from Core's functional tests.

#![allow(non_snake_case)] // Test names intentionally use double underscore.
#![allow(unused_imports)]

use bitcoin::hashes::Hash as _;
use bitcoin::SignedAmount;
use bitcoind::vtype::*;
use bitcoind::{mtype, TemplateRequest, TemplateRules};
use integration_test::{BitcoinD, BitcoinDExt as _, Wallet};

// ---------------------------------------------------------------------------
// getmininginfo: Core field validation
// ---------------------------------------------------------------------------

/// getmininginfo returns expected fields: blocks, difficulty, networkhashps.
/// Core checks chain, blocks, difficulty, and networkhashps fields.
#[test]
fn mining_core__get_mining_info__fields() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let json: GetMiningInfo = node.client.get_mining_info().expect("getmininginfo");

    #[cfg(feature = "v28_and_below")]
    let model: mtype::GetMiningInfo = json.into_model();

    #[cfg(not(feature = "v28_and_below"))]
    let model: mtype::GetMiningInfo = {
        let result: Result<mtype::GetMiningInfo, GetMiningInfoError> = json.into_model();
        result.unwrap()
    };

    // blocks should be > 0 after funding (which mines blocks).
    assert!(model.blocks > 0, "blocks should be > 0 after mining");
    // difficulty should be > 0 in regtest.
    assert!(model.difficulty > 0.0, "difficulty should be > 0");
    // network hash rate should be > 0 after mining blocks.
    assert!(model.network_hash_ps > 0.0, "networkhashps should be > 0");
}

/// getmininginfo chain should be "regtest" in test environment.
#[test]
fn mining_core__get_mining_info__chain_regtest() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    let json: GetMiningInfo = node.client.get_mining_info().expect("getmininginfo");

    #[cfg(feature = "v28_and_below")]
    let model: mtype::GetMiningInfo = json.into_model();

    #[cfg(not(feature = "v28_and_below"))]
    let model: mtype::GetMiningInfo = {
        let result: Result<mtype::GetMiningInfo, GetMiningInfoError> = json.into_model();
        result.unwrap()
    };

    assert_eq!(model.chain, "regtest", "chain should be regtest");
}

// ---------------------------------------------------------------------------
// getnetworkhashps: Core tests
// ---------------------------------------------------------------------------

/// getnetworkhashps returns a value > 0 after mining blocks.
#[test]
fn mining_core__get_network_hash_ps__positive() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let hashps: f64 = node.client.get_network_hash_ps().expect("getnetworkhashps");
    assert!(hashps > 0.0, "networkhashps should be > 0 after mining");
}

// ---------------------------------------------------------------------------
// prioritisetransaction: Core test from mining_basic.py
// ---------------------------------------------------------------------------

/// prioritisetransaction changes the fee delta for a mempool transaction.
/// Core tests that prioritised transactions get higher priority in block template.
#[test]
fn mining_core__prioritise_transaction__positive_delta() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let (_, txid) = node.create_mempool_transaction();

    // Apply a positive fee delta.
    let fee_delta = SignedAmount::from_sat(50_000);
    let result = node.client.prioritise_transaction(&txid, fee_delta).expect("prioritisetransaction");
    assert!(result, "prioritisetransaction should return true");
}

/// prioritisetransaction with negative fee delta.
#[test]
fn mining_core__prioritise_transaction__negative_delta() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let (_, txid) = node.create_mempool_transaction();

    // Negative delta to de-prioritise.
    let fee_delta = SignedAmount::from_sat(-5_000);
    let result = node.client.prioritise_transaction(&txid, fee_delta).expect("prioritisetransaction");
    assert!(result, "prioritisetransaction should return true even with negative delta");
}

/// getprioritisedtransactions lists transactions with fee deltas.
#[test]
#[cfg(not(feature = "v25_and_below"))]
fn mining_core__get_prioritised_transactions__after_prioritise() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let (_, txid) = node.create_mempool_transaction();
    let fee_delta = SignedAmount::from_sat(10_000);
    node.client.prioritise_transaction(&txid, fee_delta).expect("prioritisetransaction");

    let json: GetPrioritisedTransactions =
        node.client.get_prioritised_transactions().expect("getprioritisedtransactions");

    // The prioritised transaction should appear in the result.
    assert!(!json.0.is_empty(), "should have at least one prioritised transaction");
    let entry = json.0.get(&txid.to_string()).expect("txid should be in prioritised list");
    assert_eq!(
        entry.fee_delta, fee_delta.to_sat(),
        "fee_delta should match what we set"
    );
}

// ---------------------------------------------------------------------------
// getblocktemplate: Core test - field validation
// ---------------------------------------------------------------------------

/// getblocktemplate returns a template with required fields.
/// Core checks version, previousblockhash, transactions, coinbasevalue, target, etc.
#[test]
fn mining_core__get_block_template__fields() {
    let (node1, node2, node3) = integration_test::three_node_network();

    node1.mine_a_block();
    node2.mine_a_block();
    node3.mine_a_block();

    let options = match () {
        #[cfg(feature = "v28_and_below")]
        () => TemplateRequest { rules: vec![TemplateRules::Segwit] },
        #[cfg(not(feature = "v28_and_below"))]
        () => TemplateRequest {
            rules: vec![TemplateRules::Segwit],
            mode: Some("template".to_string()),
            ..Default::default()
        },
    };

    let json: GetBlockTemplate =
        node1.client.get_block_template(&options).expect("getblocktemplate");
    let model: Result<mtype::GetBlockTemplate, GetBlockTemplateError> = json.into_model();
    let template = model.unwrap();

    // Version should be set.
    assert!(template.version.to_consensus() > 0, "version should be > 0");

    // Height should reflect the next block.
    assert!(template.height > 0, "height should be > 0");

    // previousblockhash should exist.
    assert_ne!(
        template.previous_block_hash,
        bitcoin::BlockHash::from_byte_array([0u8; 32]),
        "previousblockhash should not be all zeros"
    );

    // coinbasevalue should be > 0.
    assert!(template.coinbase_value.to_sat() > 0, "coinbasevalue should be > 0");
}

// ---------------------------------------------------------------------------
// generatetoaddress: Core test from rpc_generate.py
// ---------------------------------------------------------------------------

/// generatetoaddress returns block hash and new block is the best block.
/// Core checks that generated block hash matches getbestblockhash.
#[test]
fn mining_core__generate_to_address__matches_best_block() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    let address = node.client.new_address().expect("newaddress");

    let json: GenerateToAddress =
        node.client.generate_to_address(1, &address).expect("generatetoaddress");
    let model: Result<mtype::GenerateToAddress, bitcoin::hex::HexToArrayError> = json.into_model();
    let hashes = model.unwrap();

    assert_eq!(hashes.0.len(), 1, "should have exactly 1 block hash");

    // The generated block should be the new best block.
    let best: GetBestBlockHash = node.client.get_best_block_hash().expect("getbestblockhash");
    let best_model: Result<mtype::GetBestBlockHash, bitcoin::hex::HexToArrayError> =
        best.into_model();
    let best_hash = best_model.unwrap().0;

    assert_eq!(hashes.0[0], best_hash, "generated block should be the best block");
}

/// generatetoaddress with multiple blocks increases block count correctly.
#[test]
fn mining_core__generate_to_address__multiple_blocks() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    let address = node.client.new_address().expect("newaddress");

    let count_before: GetBlockCount = node.client.get_block_count().expect("getblockcount");

    let nblocks = 10;
    let json: GenerateToAddress =
        node.client.generate_to_address(nblocks, &address).expect("generatetoaddress");
    let model: Result<mtype::GenerateToAddress, bitcoin::hex::HexToArrayError> = json.into_model();
    let hashes = model.unwrap();

    assert_eq!(hashes.0.len(), nblocks);

    let count_after: GetBlockCount = node.client.get_block_count().expect("getblockcount");
    assert_eq!(
        count_after.0 - count_before.0,
        nblocks as u64,
        "block count should increase by nblocks"
    );
}

// ---------------------------------------------------------------------------
// generateblock: Core test from rpc_generate.py
// ---------------------------------------------------------------------------

/// generateblock with an empty transaction list mines a block with only coinbase.
/// Core checks that empty block has len(tx) == 1.
#[test]
#[cfg(not(feature = "v20_and_below"))]
fn mining_core__generate_block__empty_block() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let mining_addr = node.client.new_address().expect("newaddress");

    #[cfg(feature = "v24_and_below")]
    let json: GenerateBlock = node
        .client
        .generate_block(&mining_addr.to_string(), &[])
        .expect("generateblock empty");

    #[cfg(not(feature = "v24_and_below"))]
    let json: GenerateBlock = node
        .client
        .generate_block(&mining_addr.to_string(), &[], true)
        .expect("generateblock empty");

    let model = json.into_model();

    #[cfg(feature = "v24_and_below")]
    {
        let result: Result<mtype::GenerateBlock, bitcoin::hex::HexToArrayError> = model;
        let block = result.unwrap();
        // Verify the block was mined by checking it's the best block.
        let best: GetBestBlockHash =
            node.client.get_best_block_hash().expect("getbestblockhash");
        let best_hash: mtype::GetBestBlockHash = best.into_model().unwrap();
        assert_eq!(block.hash, best_hash.0);
    }

    #[cfg(not(feature = "v24_and_below"))]
    {
        let result: Result<mtype::GenerateBlock, GenerateBlockError> = model;
        let block = result.unwrap();
        let best: GetBestBlockHash =
            node.client.get_best_block_hash().expect("getbestblockhash");
        let best_hash: mtype::GetBestBlockHash = best.into_model().unwrap();
        assert_eq!(block.hash, best_hash.0);
    }
}

/// generateblock with a mempool transaction includes it in the block.
/// Core checks that block contains the specified transaction.
#[test]
#[cfg(not(feature = "v20_and_below"))]
fn mining_core__generate_block__with_transaction() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let mining_addr = node.client.new_address().expect("newaddress");
    let (_, txid) = node.create_mempool_transaction();

    let transactions = vec![txid.to_string()];

    #[cfg(feature = "v24_and_below")]
    let _: GenerateBlock = node
        .client
        .generate_block(&mining_addr.to_string(), &transactions)
        .expect("generateblock with tx");

    #[cfg(not(feature = "v24_and_below"))]
    let _: GenerateBlock = node
        .client
        .generate_block(&mining_addr.to_string(), &transactions, true)
        .expect("generateblock with tx");

    // The transaction should no longer be in the mempool.
    let mempool_json: GetRawMempool = node.client.get_raw_mempool().expect("getrawmempool");
    assert!(
        !mempool_json.0.contains(&txid.to_string()),
        "transaction should be mined out of mempool"
    );
}
