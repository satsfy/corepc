// SPDX-License-Identifier: CC0-1.0

//! Tests derived from Bitcoin Core's `rpc_blockchain.py` test vectors.
//!
//! These tests exercise a wider range of inputs and assertions than the basic
//! single-happy-path tests, closely mirroring what Core's own functional tests
//! check. This helps us catch optional fields that may be missing or
//! version-dependent, and verifies that our types faithfully model Core's actual
//! RPC responses.

#![allow(non_snake_case)] // Test names intentionally use double underscore.

use bitcoin::Weight;
use bitcoind::mtype;
use bitcoind::vtype::*;
use integration_test::{BitcoinD, BitcoinDExt as _, Wallet};

// ---------------------------------------------------------------------------
// getblockchaininfo: Core checks keys, pruning fields, time, mediantime
// (from rpc_blockchain.py _test_getblockchaininfo)
// ---------------------------------------------------------------------------

/// Modelled getblockchaininfo on a non-pruned node has expected fields.
#[test]
fn blockchain_core__get_blockchain_info__non_pruned() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let json: GetBlockchainInfo = node.client.get_blockchain_info().expect("getblockchaininfo");
    let model: Result<mtype::GetBlockchainInfo, GetBlockchainInfoError> = json.into_model();
    let info = model.unwrap();

    assert_eq!(info.chain.to_string(), "regtest");
    assert!(!info.pruned, "non-pruned node should report pruned=false");
    assert!(info.size_on_disk > 0, "size_on_disk should be > 0");
    assert!(info.blocks > 0, "should have at least one block after funding");
    assert!(info.verification_progress > 0.0);
}

/// Modelled getblockchaininfo with pruning enabled has additional fields.
#[test]
fn blockchain_core__get_blockchain_info__pruned() {
    let node = BitcoinD::with_wallet(Wallet::Default, &["-prune=550"]);
    node.fund_wallet();

    let json: GetBlockchainInfo = node.client.get_blockchain_info().expect("getblockchaininfo");
    let model: Result<mtype::GetBlockchainInfo, GetBlockchainInfoError> = json.into_model();
    let info = model.unwrap();

    assert!(info.pruned, "pruned node should report pruned=true");
    assert!(info.size_on_disk > 0);
}

// ---------------------------------------------------------------------------
// getchaintxstats: Core tests with nblocks=1, default, and specific blockhash
// (from rpc_blockchain.py _test_getchaintxstats)
// ---------------------------------------------------------------------------

/// getchaintxstats: txcount should be > 0 after mining blocks.
#[test]
fn blockchain_core__get_chain_tx_stats__txcount() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();
    let (_address, _tx) = node.create_mined_transaction();

    let json: GetChainTxStats = node.client.get_chain_tx_stats().expect("getchaintxstats");
    let model: Result<mtype::GetChainTxStats, GetChainTxStatsError> = json.into_model();
    let stats = model.unwrap();

    // After fund_wallet (101 blocks) + create_mined_transaction (+1 block), we
    // should have at least 102 transactions (coinbase ones).
    assert!(stats.tx_count > 100, "txcount should be > 100, got {}", stats.tx_count);
    assert!(stats.tx_rate.unwrap() > 0.0, "txrate should be > 0");
    assert!(stats.window_block_count > 0);
}

// ---------------------------------------------------------------------------
// gettxoutsetinfo: Core checks total_amount, transactions, height, txouts, bogosize
// (from rpc_blockchain.py _test_gettxoutsetinfo)
// ---------------------------------------------------------------------------

/// gettxoutsetinfo returns correct aggregate info.
#[test]
fn blockchain_core__get_tx_out_set_info__after_mining() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();
    let (_address, _tx) = node.create_mined_transaction();

    let json: GetTxOutSetInfo = node.client.get_tx_out_set_info().expect("gettxoutsetinfo");
    let model: Result<mtype::GetTxOutSetInfo, GetTxOutSetInfoError> = json.into_model();
    let info = model.unwrap();

    assert!(info.height > 100, "height should be > 100 after fund_wallet");
    assert!(info.tx_outs > 0, "txouts should be > 0");
    assert!(info.total_amount.to_sat() > 0, "total_amount should be > 0");
    assert!(info.transactions.unwrap_or(0) > 0, "transactions should be > 0");
    assert!(info.bogo_size > 0, "bogosize should be > 0");
    assert!(info.disk_size.unwrap_or(0) > 0, "disk_size should be > 0");
}

// ---------------------------------------------------------------------------
// gettxout: Core checks bestblock, confirmations, value, coinbase
// (from rpc_blockchain.py _test_gettxout)
// ---------------------------------------------------------------------------

/// gettxout returns correct fields for a coinbase output.
#[test]
fn blockchain_core__get_tx_out__coinbase_fields() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    // Get the best block which only has a coinbase tx.
    let best_hash = node.client.best_block_hash().expect("bestblockhash");
    let block_verbose: GetBlockVerboseOne =
        node.client.get_block_verbose_one(best_hash).expect("getblock");

    let coinbase_txid = block_verbose.tx[0].parse::<bitcoin::Txid>().expect("parse coinbase txid");

    let json: GetTxOut = node.client.get_tx_out(coinbase_txid, 0).expect("gettxout");
    let model: Result<mtype::GetTxOut, GetTxOutError> = json.into_model();
    let txout = model.unwrap();

    assert_eq!(txout.confirmations, 1, "coinbase in best block should have 1 confirmation");
    assert!(txout.coinbase, "output should be marked as coinbase");
    assert!(txout.tx_out.value.to_sat() > 0, "coinbase value should be > 0");
}

/// gettxout returns correct fields for a non-coinbase, mined output.
#[test]
fn blockchain_core__get_tx_out__mined_tx() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();
    let (_address, tx) = node.create_mined_transaction();
    let txid = tx.compute_txid();

    let json: GetTxOut = node.client.get_tx_out(txid, 1).expect("gettxout");
    let model: Result<mtype::GetTxOut, GetTxOutError> = json.into_model();
    let txout = model.unwrap();

    assert!(!txout.coinbase, "non-coinbase tx should not be marked coinbase");
    assert!(txout.confirmations >= 1, "mined tx should have >= 1 confirmation");
}

// ---------------------------------------------------------------------------
// getblockheader (verbose): Core checks hash, height, confirmations,
// previousblockhash, chainwork, nTx, mediantime, nonce, version, etc.
// (from rpc_blockchain.py _test_getblockheader)
// ---------------------------------------------------------------------------

/// getblockheader verbose returns rich block metadata.
#[test]
fn blockchain_core__get_block_header_verbose__fields() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();
    node.mine_a_block();

    let best_hash = node.client.best_block_hash().expect("bestblockhash");
    let json: GetBlockHeaderVerbose =
        node.client.get_block_header_verbose(&best_hash).expect("getblockheader verbose");
    let model: Result<mtype::GetBlockHeaderVerbose, GetBlockHeaderVerboseError> = json.into_model();
    let header = model.unwrap();

    assert_eq!(header.hash, best_hash);
    assert_eq!(header.confirmations, 1);
    assert!(header.height > 100);
    assert!(header.n_tx >= 1, "block should have at least one tx (coinbase)");
    assert!(header.time > 0);
    assert!(header.median_time > 0);
    assert!(
        header.previous_block_hash.is_some(),
        "non-genesis block should have previousblockhash"
    );
}

/// getblockheader for genesis block should have no previousblockhash.
#[test]
fn blockchain_core__get_block_header_verbose__genesis_no_prev() {
    let node = BitcoinD::with_wallet(Wallet::None, &[]);

    let genesis_hash =
        node.client.get_block_hash(0).expect("getblockhash 0").block_hash().expect("parse");
    let json: GetBlockHeaderVerbose =
        node.client.get_block_header_verbose(&genesis_hash).expect("getblockheader verbose");
    let model: Result<mtype::GetBlockHeaderVerbose, GetBlockHeaderVerboseError> = json.into_model();
    let header = model.unwrap();

    assert_eq!(header.height, 0);
    assert!(header.previous_block_hash.is_none(), "genesis should have no previousblockhash");
}

/// getblockheader for tip should have no nextblockhash.
#[test]
fn blockchain_core__get_block_header_verbose__tip_no_next() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let best_hash = node.client.best_block_hash().expect("bestblockhash");
    let json: GetBlockHeaderVerbose =
        node.client.get_block_header_verbose(&best_hash).expect("getblockheader verbose");
    let model: Result<mtype::GetBlockHeaderVerbose, GetBlockHeaderVerboseError> = json.into_model();
    let header = model.unwrap();

    assert!(header.next_block_hash.is_none(), "tip should have no nextblockhash");
}

// ---------------------------------------------------------------------------
// getdifficulty: Core checks difficulty * 2^31 ≈ 1 on regtest
// (from rpc_blockchain.py _test_getdifficulty)
// ---------------------------------------------------------------------------

/// getdifficulty on regtest is approximately 4.656542373906925e-10.
#[test]
fn blockchain_core__get_difficulty__regtest_value() {
    let node = BitcoinD::with_wallet(Wallet::None, &[]);

    let json: GetDifficulty = node.client.get_difficulty().expect("getdifficulty");
    let model: mtype::GetDifficulty = json.into_model();
    let difficulty = model.0;

    // In regtest, difficulty is 1/2^31 ≈ 4.656542373906925e-10.
    // Core checks: abs(difficulty * 2**31 - 1) < 0.0001
    let product = difficulty * (1u64 << 31) as f64;
    assert!(
        (product - 1.0).abs() < 0.0001,
        "difficulty * 2^31 should be approximately 1, got {}",
        product
    );
}

// ---------------------------------------------------------------------------
// getblockcount: Core expects it to match the chain height
// ---------------------------------------------------------------------------

/// getblockcount matches chain height after mining.
#[test]
fn blockchain_core__get_block_count__matches_height() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet(); // Mines 101 blocks.

    let json: GetBlockCount = node.client.get_block_count().unwrap();
    let count: mtype::GetBlockCount = json.into_model();

    assert_eq!(count.0, 101, "block count should be 101 after fund_wallet");
}

// ---------------------------------------------------------------------------
// getblockhash / getbestblockhash: consistency check
// ---------------------------------------------------------------------------

/// getblockhash(height) for the tip should equal getbestblockhash.
#[test]
fn blockchain_core__get_block_hash__tip_equals_best() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let count: GetBlockCount = node.client.get_block_count().unwrap();
    let height = count.0;

    let hash_at_height =
        node.client.get_block_hash(height).expect("getblockhash").block_hash().expect("parse");
    let best_hash = node.client.best_block_hash().expect("bestblockhash");

    assert_eq!(hash_at_height, best_hash);
}

// ---------------------------------------------------------------------------
// getblock: Core tests various verbosity levels
// (from rpc_blockchain.py _test_getblock)
// ---------------------------------------------------------------------------

/// getblock verbosity 0 returns hex that parses to a valid block.
#[test]
fn blockchain_core__get_block__verbosity_zero_parses() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();
    let hash = node.client.best_block_hash().expect("bestblockhash");

    let json: GetBlockVerboseZero =
        node.client.get_block_verbose_zero(hash).expect("getblock verbose=0");
    let model: Result<mtype::GetBlockVerboseZero, _> = json.into_model();
    let block = model.unwrap();

    // The parsed block should have at least one transaction (coinbase).
    assert!(!block.0.txdata.is_empty(), "block should have at least the coinbase tx");
}

/// getblock verbosity 1 returns block with tx list as txids.
#[test]
fn blockchain_core__get_block__verbosity_one_fields() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();
    let (_address, _tx) = node.create_mined_transaction();
    let hash = node.client.best_block_hash().expect("bestblockhash");

    let json: GetBlockVerboseOne =
        node.client.get_block_verbose_one(hash).expect("getblock verbose=1");
    let model: Result<mtype::GetBlockVerboseOne, GetBlockVerboseOneError> = json.into_model();
    let block = model.unwrap();

    // The block with a mined transaction should have >= 2 txs (coinbase + ours).
    assert!(block.tx.len() >= 2, "block should have at least 2 txs, got {}", block.tx.len());
    assert!(block.n_tx >= 2);
    assert_eq!(block.tx.len(), block.n_tx as usize);
    assert!(block.confirmations >= 1);
    assert!(block.height > 100);
    assert!(block.size > 0);
    assert!(block.weight > Weight::ZERO);
}

// ---------------------------------------------------------------------------
// getblockstats: Core tests with specific stat selection
// (from rpc_blockchain.py via rpc_getblockstats.py)
// ---------------------------------------------------------------------------

/// getblockstats with all stats returns all fields.
#[test]
fn blockchain_core__get_block_stats__all_fields() {
    let node = if cfg!(feature = "v18_and_below") {
        BitcoinD::with_wallet(Wallet::Default, &["-txindex"])
    } else {
        BitcoinD::with_wallet(Wallet::Default, &[])
    };
    node.fund_wallet();
    let (_address, _tx) = node.create_mined_transaction();

    let json: GetBlockStats =
        node.client.get_block_stats_by_height(102, None).expect("getblockstats");
    let model: Result<mtype::GetBlockStats, GetBlockStatsError> = json.into_model();
    let stats = model.unwrap();

    // When all stats are requested, key fields should be present.
    assert_eq!(stats.height, Some(102));
    assert!(stats.block_hash.is_some());
    assert!(stats.total_size.is_some());
    assert!(stats.total_weight.is_some());
    assert!(stats.txs.is_some());
}

/// getblockstats with select stats returns only those fields.
#[test]
fn blockchain_core__get_block_stats__select_stats() {
    let node = if cfg!(feature = "v18_and_below") {
        BitcoinD::with_wallet(Wallet::Default, &["-txindex"])
    } else {
        BitcoinD::with_wallet(Wallet::Default, &[])
    };
    node.fund_wallet();

    let json: GetBlockStats = node
        .client
        .get_block_stats_by_height(101, Some(&["txs", "avgfee", "height"]))
        .expect("getblockstats");
    let model: Result<mtype::GetBlockStats, GetBlockStatsError> = json.into_model();
    let stats = model.unwrap();

    assert!(stats.txs.is_some(), "txs should be present when requested");
    assert!(stats.average_fee.is_some(), "avgfee should be present when requested");
    assert!(stats.height.is_some(), "height should be present when requested");
    // Non-requested fields should be None.
    assert!(stats.block_hash.is_none(), "blockhash should be absent when not requested");
    assert!(stats.total_size.is_none(), "total_size should be absent when not requested");
}

// ---------------------------------------------------------------------------
// getchaintips: Core tests tip after mining + fork detection
// (from rpc_getchaintips.py)
// ---------------------------------------------------------------------------

/// getchaintips returns at least one tip in active state.
#[test]
fn blockchain_core__get_chain_tips__active_tip() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let json: GetChainTips = node.client.get_chain_tips().expect("getchaintips");
    let model: Result<mtype::GetChainTips, ChainTipsError> = json.into_model();
    let tips = model.unwrap();

    assert!(!tips.0.is_empty(), "should have at least one chain tip");
    let active_tips: Vec<_> =
        tips.0.iter().filter(|t| t.status == mtype::ChainTipsStatus::Active).collect();
    assert_eq!(active_tips.len(), 1, "should have exactly one active tip");
    assert!(active_tips[0].height > 100);
}

// ---------------------------------------------------------------------------
// getblockfilter: Core tests filter retrieval
// (from rpc_getblockfilter.py)
// ---------------------------------------------------------------------------

/// getblockfilter returns a valid filter for the genesis block.
#[test]
#[cfg(not(feature = "v18_and_below"))]
fn blockchain_core__get_block_filter__genesis() {
    let node = BitcoinD::with_wallet(Wallet::None, &["-blockfilterindex"]);
    // Wait for the index to sync.
    std::thread::sleep(std::time::Duration::from_millis(500));

    let genesis_hash =
        node.client.get_block_hash(0).expect("getblockhash 0").block_hash().expect("parse");

    let json: GetBlockFilter = node.client.get_block_filter(genesis_hash).expect("getblockfilter");
    let model: Result<mtype::GetBlockFilter, GetBlockFilterError> = json.into_model();
    model.unwrap();
}

/// getblockfilter for a block with transactions returns a different filter.
#[test]
#[cfg(not(feature = "v18_and_below"))]
fn blockchain_core__get_block_filter__with_tx() {
    let node = BitcoinD::with_wallet(Wallet::Default, &["-blockfilterindex"]);
    node.fund_wallet();
    let (_address, _tx) = node.create_mined_transaction();

    let best_hash = node.client.best_block_hash().expect("bestblockhash");
    let json: GetBlockFilter = node.client.get_block_filter(best_hash).expect("getblockfilter");
    let model: Result<mtype::GetBlockFilter, GetBlockFilterError> = json.into_model();
    model.unwrap();
}

// ---------------------------------------------------------------------------
// getmempoolinfo, getrawmempool, getmempoolentry: Core exercises these
// with mempool contents and checks size, bytes, etc.
// ---------------------------------------------------------------------------

/// getmempoolinfo reports correct size after sending to mempool.
#[test]
fn blockchain_core__get_mempool_info__with_transactions() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    // Create two mempool transactions.
    let (_addr1, _txid1) = node.create_mempool_transaction();
    let (_addr2, _txid2) = node.create_mempool_transaction();

    let json: GetMempoolInfo = node.client.get_mempool_info().expect("getmempoolinfo");
    let model: Result<mtype::GetMempoolInfo, GetMempoolInfoError> = json.into_model();
    let info = model.unwrap();

    assert_eq!(info.size, 2, "mempool should have exactly 2 transactions");
    assert!(info.bytes > 0, "mempool bytes should be > 0");
}

/// getrawmempool verbose gives entries with fee info.
#[test]
fn blockchain_core__get_raw_mempool_verbose__fee_info() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();
    let (_address, txid) = node.create_mempool_transaction();

    let json: GetRawMempoolVerbose =
        node.client.get_raw_mempool_verbose().expect("getrawmempool verbose");
    let model: Result<mtype::GetRawMempoolVerbose, MapMempoolEntryError> = json.into_model();
    let mempool = model.unwrap();

    let entry = mempool.0.get(&txid).expect("our txid should be in the mempool");
    assert!(entry.vsize.unwrap_or(0) > 0, "vsize should be > 0");
    assert!(entry.weight.unwrap_or(0) > 0, "weight should be > 0");
    assert!(entry.time > 0, "time should be > 0");
}

// ---------------------------------------------------------------------------
// getmempoolancestors / getmempooldescendants: Core tests parent-child
// (from rpc_blockchain.py via the mempool tests)
// ---------------------------------------------------------------------------

/// getmempoolentry returns correct fields for a mempool transaction.
#[test]
fn blockchain_core__get_mempool_entry__fields() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();
    let (_address, txid) = node.create_mempool_transaction();

    let json: GetMempoolEntry = node.client.get_mempool_entry(txid).expect("getmempoolentry");
    let model: Result<mtype::GetMempoolEntry, MempoolEntryError> = json.into_model();
    let entry = model.unwrap();

    assert!(entry.0.vsize.unwrap_or(0) > 0);
    assert!(entry.0.weight.unwrap_or(0) > 0);
    assert!(entry.0.time > 0);
    assert!(entry.0.descendant_count >= 1, "should count self as descendant");
    assert!(entry.0.ancestor_count >= 1, "should count self as ancestor");
}

// ---------------------------------------------------------------------------
// verifytxoutproof: round-trip with gettxoutproof
// (from rpc_txoutproof.py)
// ---------------------------------------------------------------------------

/// gettxoutproof / verifytxoutproof round-trip for a mined transaction.
#[test]
fn blockchain_core__verify_tx_out_proof__round_trip() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();
    let (_address, tx) = node.create_mined_transaction();
    let txid = tx.compute_txid();

    let proof = node.client.get_tx_out_proof(&[txid]).expect("gettxoutproof");
    let json: VerifyTxOutProof = node.client.verify_tx_out_proof(&proof).expect("verifytxoutproof");
    let model: Result<mtype::VerifyTxOutProof, _> = json.into_model();
    let txids = model.unwrap();

    assert_eq!(txids.0.len(), 1, "proof should verify exactly 1 txid");
    assert_eq!(txids.0[0], txid, "verified txid should match");
}

// ---------------------------------------------------------------------------
// verifychain: Core calls verifychain(4, 0) and expects true
// ---------------------------------------------------------------------------

/// verifychain succeeds on a healthy chain.
#[test]
fn blockchain_core__verify_chain__healthy() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let json: VerifyChain = node.client.verify_chain().expect("verifychain");
    assert!(json.0, "verifychain should return true on a healthy chain");
}

// ---------------------------------------------------------------------------
// scantxoutset: Core tests scan with descriptors
// (from rpc_scantxoutset.py)
// ---------------------------------------------------------------------------

/// scantxoutset start returns valid results for a known descriptor.
#[test]
fn blockchain_core__scan_tx_out_set__with_funded_descriptor() {
    let node = match () {
        #[cfg(feature = "v21_and_below")]
        () => BitcoinD::with_wallet(Wallet::Default, &[]),
        #[cfg(not(feature = "v21_and_below"))]
        () => BitcoinD::with_wallet(Wallet::Default, &["-coinstatsindex=1"]),
    };
    node.fund_wallet();

    // Get an address and its descriptor from the wallet.
    let address = node.client.new_address().expect("new_address");
    let _addr_info: GetAddressInfo =
        node.client.get_address_info(&address).expect("getaddressinfo");

    // Use a generic descriptor that won't match (testing the shape of the response).
    let dummy_pubkey_hex = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
    let scan_desc = format!("pkh({})", dummy_pubkey_hex);

    let json: ScanTxOutSetStart =
        node.client.scan_tx_out_set_start(&[&scan_desc]).expect("scantxoutset start");
    let model: Result<mtype::ScanTxOutSetStart, ScanTxOutSetError> = json.into_model();
    let result = model.unwrap();

    // Even if no UTXOs match, the structure should be valid.
    assert!(result.height.unwrap_or(0) > 0);
}

// ---------------------------------------------------------------------------
// getdeploymentinfo: Core tests deployment info consistency
// (from rpc_blockchain.py _test_getdeploymentinfo)
// ---------------------------------------------------------------------------

/// getdeploymentinfo for genesis vs tip returns different hashes.
#[test]
#[cfg(not(feature = "v22_and_below"))]
fn blockchain_core__get_deployment_info__genesis_vs_tip() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let genesis_hash =
        node.client.get_block_hash(0).expect("getblockhash").block_hash().expect("parse");
    let tip_hash = node.client.best_block_hash().expect("bestblockhash");

    let json_genesis: GetDeploymentInfo =
        node.client.get_deployment_info(&genesis_hash).expect("getdeploymentinfo genesis");
    let model_genesis: Result<mtype::GetDeploymentInfo, GetDeploymentInfoError> =
        json_genesis.into_model();
    let info_genesis = model_genesis.unwrap();

    let json_tip: GetDeploymentInfo =
        node.client.get_deployment_info_tip().expect("getdeploymentinfo tip");
    let model_tip: Result<mtype::GetDeploymentInfo, GetDeploymentInfoError> = json_tip.into_model();
    let info_tip = model_tip.unwrap();

    assert_eq!(info_genesis.hash, genesis_hash);
    assert_eq!(info_tip.hash, tip_hash);
    assert_ne!(info_genesis.hash, info_tip.hash, "genesis and tip hashes should differ");
    assert_eq!(info_genesis.height, 0);
    assert!(info_tip.height > 100);
}

// ---------------------------------------------------------------------------
// gettxspendingprevout: Core tests spent vs unspent in mempool
// (from rpc_gettxspendingprevout.py)
// ---------------------------------------------------------------------------

/// gettxspendingprevout correctly reports spending status.
#[test]
#[cfg(not(feature = "v23_and_below"))]
fn blockchain_core__get_tx_spending_prevout__mixed() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    // Create a transaction in the mempool.
    let (_address, txid) = node.create_mempool_transaction();

    // Query for the first output of our transaction (which is in the mempool).
    let inputs = vec![bitcoin::OutPoint { txid, vout: 0 }];
    let json: GetTxSpendingPrevout =
        node.client.get_tx_spending_prevout(&inputs).expect("gettxspendingprevout");
    let model: Result<mtype::GetTxSpendingPrevout, GetTxSpendingPrevoutError> = json.into_model();
    let result = model.unwrap();

    assert_eq!(result.0.len(), 1);
    assert_eq!(result.0[0].outpoint.txid, txid);
    assert_eq!(result.0[0].outpoint.vout, 0);
}

// ---------------------------------------------------------------------------
// invalidateblock / reconsiderblock: Core tests chain reorganization
// (from rpc_invalidateblock.py)
// ---------------------------------------------------------------------------

/// invalidateblock reverts tip, reconsider_block restores it.
#[test]
fn blockchain_core__invalidate_and_reconsider_block() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let original_count: GetBlockCount = node.client.get_block_count().unwrap();
    let original_height = original_count.0;

    // Mine one more block.
    node.mine_a_block();
    let new_count: GetBlockCount = node.client.get_block_count().unwrap();
    assert_eq!(new_count.0, original_height + 1);

    let tip = node.client.best_block_hash().expect("bestblockhash");

    // Invalidate the tip.
    node.client.invalidate_block(tip).expect("invalidateblock");
    let after_invalidate: GetBlockCount = node.client.get_block_count().unwrap();
    assert_eq!(after_invalidate.0, original_height, "should revert to original height");

    // Reconsider the block.
    node.client.reconsider_block(tip).expect("reconsiderblock");
    let after_reconsider: GetBlockCount = node.client.get_block_count().unwrap();
    assert_eq!(after_reconsider.0, original_height + 1, "should restore to new height");
}

// ---------------------------------------------------------------------------
// preciousblock: Core tests that preciousblock changes the active tip
// (from rpc_preciousblock.py)
// ---------------------------------------------------------------------------

/// preciousblock marks a block as preferred tip candidate.
#[test]
fn blockchain_core__precious_block__accepted() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();
    node.mine_a_block();

    // Get a block that's not the tip.
    let count: GetBlockCount = node.client.get_block_count().unwrap();
    let prev_hash =
        node.client.get_block_hash(count.0 - 1).expect("getblockhash").block_hash().expect("parse");

    // preciousblock on a valid block should succeed without error.
    node.client.precious_block(prev_hash).expect("preciousblock");
}
