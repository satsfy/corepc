// SPDX-License-Identifier: CC0-1.0

//! Tests derived from Bitcoin Core's `rpc_rawtransaction.py`, `rpc_psbt.py`,
//! and `rpc_signrawtransactionwithkey.py` test vectors.
//!
//! Exercises raw transaction RPCs with deeper field validation, including
//! decode_raw_transaction field checks, PSBT lifecycle (create → update →
//! process → finalize → extract), and testmempoolaccept edge cases.

#![allow(non_snake_case)] // Test names intentionally use double underscore.
#![allow(unused_imports)]

use bitcoin::consensus::encode;
use bitcoin::hex::FromHex as _;
use bitcoin::opcodes::all::*;
use bitcoin::{
    absolute, consensus, hex, psbt, script, transaction, Amount, ScriptBuf, Transaction, TxOut,
};
use bitcoind::vtype::*;
use bitcoind::{mtype, Input, Output};
use integration_test::{BitcoinD, BitcoinDExt as _, Wallet};

// ---------------------------------------------------------------------------
// getrawtransaction: Core test vectors from rpc_rawtransaction.py
// ---------------------------------------------------------------------------

/// getrawtransaction verbose=true returns in_active_chain for mined tx.
/// Core checks that in_active_chain is present when tx is in a block.
#[test]
fn raw_transactions_core__get_raw_transaction_verbose__in_active_chain() {
    let node = BitcoinD::with_wallet(Wallet::Default, &["-txindex"]);
    node.fund_wallet();

    let (_, tx) = node.create_mined_transaction();
    let txid = tx.compute_txid();

    let json: GetRawTransactionVerbose =
        node.client.get_raw_transaction_verbose(txid).expect("getrawtransaction verbose");
    let model: Result<mtype::GetRawTransactionVerbose, GetRawTransactionVerboseError> =
        json.into_model();
    let result = model.unwrap();

    // Mined transactions should have confirmations > 0.
    assert!(
        result.confirmations.unwrap_or(0) > 0,
        "mined tx should have confirmations > 0"
    );

    // blockhash should be present for a mined transaction.
    assert!(result.block_hash.is_some(), "mined tx should have blockhash");
}

/// getrawtransaction verbose for mempool tx has no confirmations/blockhash.
/// Core checks that mempool transactions don't have block-related fields.
#[test]
fn raw_transactions_core__get_raw_transaction_verbose__mempool_tx() {
    let node = BitcoinD::with_wallet(Wallet::Default, &["-txindex"]);
    node.fund_wallet();

    let (_, txid) = node.create_mempool_transaction();

    let json: GetRawTransactionVerbose =
        node.client.get_raw_transaction_verbose(txid).expect("getrawtransaction verbose");
    let model: Result<mtype::GetRawTransactionVerbose, GetRawTransactionVerboseError> =
        json.into_model();
    let result = model.unwrap();

    // Mempool transactions typically show 0 confirmations.
    assert_eq!(
        result.confirmations.unwrap_or(0),
        0,
        "mempool tx should have 0 confirmations"
    );
}

/// getrawtransaction non-verbose returns hex that decodes to the same tx.
#[test]
fn raw_transactions_core__get_raw_transaction__hex_roundtrip() {
    let node = BitcoinD::with_wallet(Wallet::Default, &["-txindex"]);
    node.fund_wallet();

    let (_, tx) = node.create_mined_transaction();
    let txid = tx.compute_txid();

    let json: GetRawTransaction =
        node.client.get_raw_transaction(txid).expect("getrawtransaction");
    let model: Result<mtype::GetRawTransaction, encode::FromHexError> = json.into_model();
    let result = model.unwrap();

    assert_eq!(result.0.compute_txid(), txid, "decoded tx should have the same txid");
}

// ---------------------------------------------------------------------------
// decoderawtransaction: Core test vectors from rpc_rawtransaction.py
// ---------------------------------------------------------------------------

/// decoderawtransaction returns correct vout and vin counts.
/// Core checks vin/vout array lengths, txid, and version fields.
#[test]
fn raw_transactions_core__decode_raw_transaction__field_validation() {
    let node = BitcoinD::with_wallet(Wallet::Default, &["-txindex"]);
    node.fund_wallet();

    let (_, tx) = node.create_mined_transaction();

    let json: DecodeRawTransaction =
        node.client.decode_raw_transaction(&tx).expect("decoderawtransaction");
    let model: Result<mtype::DecodeRawTransaction, RawTransactionError> = json.into_model();
    let decoded = model.unwrap();

    // A standard transaction should have at least 1 input and 1 output.
    assert!(!decoded.0.input.is_empty(), "decoded tx should have inputs");
    assert!(!decoded.0.output.is_empty(), "decoded tx should have outputs");

    // txid from decoded should match the original.
    assert_eq!(decoded.0.compute_txid(), tx.compute_txid(), "decoded txid should match");

    // Version should be 1 or 2 for standard transactions.
    assert!(
        decoded.0.version == transaction::Version::ONE || decoded.0.version == transaction::Version::TWO,
        "version should be 1 or 2"
    );
}

// ---------------------------------------------------------------------------
// decodescript: Core test vectors from rpc_decodescript.py
// ---------------------------------------------------------------------------

/// decodescript for P2PKH script identifies type correctly.
/// Core checks: type == "pubkeyhash", reqSigs == 1, p2sh wrapping.
#[test]
fn raw_transactions_core__decode_script__p2pkh() {
    let node = BitcoinD::with_wallet(Wallet::None, &[]);

    // P2PKH: OP_DUP OP_HASH160 <20 byte hash> OP_EQUALVERIFY OP_CHECKSIG
    let pubkey_hash = "16e1ae70ff0fa102905d4af297f6912bda6cce19";
    let script_hex = format!("76a914{}88ac", pubkey_hash);

    let json: DecodeScript = node.client.decode_script(&script_hex).expect("decodescript");
    let model: Result<mtype::DecodeScript, DecodeScriptError> = json.into_model();
    let result = model.unwrap();

    assert_eq!(result.type_.as_str(), "pubkeyhash");
}

/// decodescript for P2PK script identifies type correctly.
/// Core test: OP_PUSHBYTES_33 <compressed pubkey> OP_CHECKSIG -> "pubkey"
#[test]
fn raw_transactions_core__decode_script__p2pk() {
    let node = BitcoinD::with_wallet(Wallet::None, &[]);

    // P2PK: <33-byte compressed pubkey> OP_CHECKSIG
    let pubkey = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
    let script_hex = format!("21{}ac", pubkey);

    let json: DecodeScript = node.client.decode_script(&script_hex).expect("decodescript");
    let model: Result<mtype::DecodeScript, DecodeScriptError> = json.into_model();
    let result = model.unwrap();

    assert_eq!(result.type_.as_str(), "pubkey");
}

/// decodescript for multisig script identifies type correctly.
/// Core test: OP_1 <pubkey1> <pubkey2> OP_2 OP_CHECKMULTISIG -> "multisig"
#[test]
fn raw_transactions_core__decode_script__multisig() {
    let node = BitcoinD::with_wallet(Wallet::None, &[]);

    let pk1 = "022afc20bf379bc96a2f4e9e63ffceb8652b2b6a097f63fbee6ecec2a49a48010e";
    let pk2 = "03a767c7221e9f15f870f1ad9311f5ab937d79fcaeee15bb2c722bca515581b4c0";

    let script = script::Builder::new()
        .push_opcode(OP_PUSHNUM_1)
        .push_opcode(OP_PUSHBYTES_33)
        .push_slice(<[u8; 33]>::from_hex(pk1).unwrap())
        .push_opcode(OP_PUSHBYTES_33)
        .push_slice(<[u8; 33]>::from_hex(pk2).unwrap())
        .push_opcode(OP_PUSHNUM_2)
        .push_opcode(OP_CHECKMULTISIG)
        .into_script();

    let json: DecodeScript =
        node.client.decode_script(&script.to_hex_string()).expect("decodescript");
    let model: Result<mtype::DecodeScript, DecodeScriptError> = json.into_model();
    let result = model.unwrap();

    assert_eq!(result.type_.as_str(), "multisig");
}

/// decodescript for empty script returns "nonstandard" type.
#[test]
fn raw_transactions_core__decode_script__empty() {
    let node = BitcoinD::with_wallet(Wallet::None, &[]);

    let json: DecodeScript = node.client.decode_script("").expect("decodescript empty");
    let model: Result<mtype::DecodeScript, DecodeScriptError> = json.into_model();
    let result = model.unwrap();

    assert_eq!(result.type_.as_str(), "nonstandard");
}

// ---------------------------------------------------------------------------
// createrawtransaction: Core test vectors from rpc_rawtransaction.py
// ---------------------------------------------------------------------------

/// createrawtransaction with basic inputs and outputs produces valid tx hex.
/// Core tests various combinations of inputs/outputs, sequence numbers.
#[test]
fn raw_transactions_core__create_raw_transaction__basic() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let (_, tx) = node.create_mined_transaction();
    let txid = tx.compute_txid();

    let spend_addr = node.client.new_address().expect("newaddress");
    let inputs = vec![Input { txid, vout: 0, sequence: None }];
    let outputs = vec![Output::new(spend_addr, Amount::from_sat(50_000))];

    let json: CreateRawTransaction =
        node.client.create_raw_transaction(&inputs, &outputs).expect("createrawtransaction");
    let created_tx = json.transaction().unwrap();

    assert_eq!(created_tx.input.len(), 1, "should have 1 input");
    assert_eq!(created_tx.output.len(), 1, "should have 1 output");
    assert_eq!(created_tx.input[0].previous_output.txid, txid);
    assert_eq!(created_tx.input[0].previous_output.vout, 0);
}

/// createrawtransaction with multiple outputs.
/// Core tests both array and object output formats.
#[test]
fn raw_transactions_core__create_raw_transaction__multiple_outputs() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let (_, tx) = node.create_mined_transaction();
    let txid = tx.compute_txid();

    let addr1 = node.client.new_address().expect("newaddress 1");
    let addr2 = node.client.new_address().expect("newaddress 2");

    let inputs = vec![Input { txid, vout: 0, sequence: None }];
    let outputs = vec![
        Output::new(addr1, Amount::from_sat(30_000)),
        Output::new(addr2, Amount::from_sat(20_000)),
    ];

    let json: CreateRawTransaction =
        node.client.create_raw_transaction(&inputs, &outputs).expect("createrawtransaction");
    let created_tx = json.transaction().unwrap();

    assert_eq!(created_tx.input.len(), 1);
    assert_eq!(created_tx.output.len(), 2);
    assert_eq!(created_tx.output[0].value.to_sat(), 30_000);
    assert_eq!(created_tx.output[1].value.to_sat(), 20_000);
}

// ---------------------------------------------------------------------------
// testmempoolaccept: Core test vectors from rpc_rawtransaction.py
// ---------------------------------------------------------------------------

/// testmempoolaccept with a valid signed transaction reports allowed=true.
/// Core tests maxfeerate parameter and already-in-chain rejection.
#[test]
fn raw_transactions_core__test_mempool_accept__valid_tx() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let (_, tx) = node.create_mined_transaction();
    let txid = tx.compute_txid();

    // Find the UTXO.
    let tx_out = node
        .client
        .get_tx_out(txid, 0)
        .expect("gettxout")
        .into_model()
        .expect("GetTxOut into model")
        .tx_out;

    let spend_amount = Amount::from_sat(100_000);
    let fee = Amount::from_sat(1000);
    let change_amount = tx_out.value - spend_amount - fee;

    let addr = node.client.new_address().expect("newaddress");
    let change_addr = node
        .client
        .get_raw_change_address()
        .expect("getrawchangeaddress")
        .into_model()
        .unwrap()
        .0
        .assume_checked();

    let inputs = vec![Input { txid, vout: 0, sequence: None }];
    let outputs = vec![
        Output::new(addr, spend_amount),
        Output::new(change_addr, change_amount),
    ];

    let json: CreateRawTransaction =
        node.client.create_raw_transaction(&inputs, &outputs).expect("createrawtransaction");
    let raw_tx = json.transaction().unwrap();

    let signed: SignRawTransactionWithWallet =
        node.client.sign_raw_transaction_with_wallet(&raw_tx).expect("signrawtransactionwithwallet");
    let signed_tx = signed.into_model().unwrap().tx;

    let json: TestMempoolAccept =
        node.client.test_mempool_accept(&[signed_tx.clone()]).expect("testmempoolaccept");

    #[cfg(feature = "v20_and_below")]
    type TestMempoolAcceptError = hex::HexToArrayError;
    let model: Result<mtype::TestMempoolAccept, TestMempoolAcceptError> = json.into_model();
    let result = model.unwrap();

    assert_eq!(result.results.len(), 1);
    assert!(result.results[0].allowed, "valid signed tx should be accepted by mempool");
    assert_eq!(result.results[0].txid, signed_tx.compute_txid());
}

/// testmempoolaccept with an already-broadcast transaction reports it's not allowed.
/// Core tests that a transaction already in the mempool is rejected.
#[test]
fn raw_transactions_core__test_mempool_accept__in_mempool() {
    let node = BitcoinD::with_wallet(Wallet::Default, &["-txindex"]);
    node.fund_wallet();

    let (_, txid) = node.create_mempool_transaction();

    // Get the raw transaction that's now in the mempool.
    let json: GetRawTransaction =
        node.client.get_raw_transaction(txid).expect("getrawtransaction");
    let tx = json.into_model().unwrap().0;

    let json: TestMempoolAccept =
        node.client.test_mempool_accept(&[tx]).expect("testmempoolaccept");

    #[cfg(feature = "v20_and_below")]
    type TestMempoolAcceptError = hex::HexToArrayError;
    let model: Result<mtype::TestMempoolAccept, TestMempoolAcceptError> = json.into_model();
    let result = model.unwrap();

    assert_eq!(result.results.len(), 1);
    assert!(!result.results[0].allowed, "in-mempool tx should be rejected");
}

// ---------------------------------------------------------------------------
// PSBT lifecycle: create → walletprocesspsbt → finalize → extract
// (from rpc_psbt.py multisig and basic workflows)
// ---------------------------------------------------------------------------

/// Full PSBT lifecycle: create → process → finalize → send.
/// Core tests this with 2-of-3 multisig, but here we do a simpler wallet flow.
#[test]
fn raw_transactions_core__psbt_lifecycle__create_process_finalize() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    // Create PSBT.
    let psbt = create_a_psbt(&node);

    // Process with wallet.
    let json: WalletProcessPsbt =
        node.client.wallet_process_psbt(&psbt).expect("walletprocesspsbt");
    let model: mtype::WalletProcessPsbt = json.into_model().expect("WalletProcessPsbt into model");

    assert!(model.complete, "wallet should fully sign the PSBT");

    // Finalize.
    let json: FinalizePsbt =
        node.client.finalize_psbt(&model.psbt).expect("finalizepsbt");
    let model: Result<mtype::FinalizePsbt, FinalizePsbtError> = json.into_model();
    let finalized = model.unwrap();

    assert!(finalized.complete, "finalized PSBT should be complete");
    assert!(finalized.tx.is_some(), "finalized PSBT should have tx");

    // Extract and send.
    let tx = finalized.tx.unwrap();
    let _: SendRawTransaction =
        node.client.send_raw_transaction(&tx).expect("sendrawtransaction");
}

/// PSBT without signing reports complete=false in analyze.
/// Core checks that analyzepsbt reports proper next steps for unsigned PSBT.
#[test]
#[cfg(not(feature = "v17"))]
fn raw_transactions_core__analyze_psbt__unsigned() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let psbt = create_a_psbt(&node);

    let json: AnalyzePsbt = node.client.analyze_psbt(&psbt).expect("analyzepsbt");
    let model: Result<mtype::AnalyzePsbt, AnalyzePsbtError> = json.into_model();
    let analysis = model.unwrap();

    // Unsigned PSBT should not be ready for extraction.
    assert!(
        !analysis.next.is_empty(),
        "unsigned PSBT should have a next action (updater/signer)"
    );
}

/// PSBT combined from two copies is identical to each.
/// Core tests that combining two identical PSBTs gives the same PSBT.
#[test]
fn raw_transactions_core__combine_psbt__identical() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let psbt = create_a_psbt(&node);

    let json: CombinePsbt =
        node.client.combine_psbt(&[psbt.clone(), psbt.clone()]).expect("combinepsbt");
    let model: Result<mtype::CombinePsbt, psbt::PsbtParseError> = json.into_model();
    let combined = model.unwrap();

    assert_eq!(combined.0, psbt, "combining identical PSBTs should yield the same PSBT");
}

/// joinpsbts produces a PSBT with inputs from both source PSBTs.
/// Core tests that joinpsbts rejects common inputs and shuffles outputs.
#[test]
#[cfg(not(feature = "v17"))]
fn raw_transactions_core__join_psbts__distinct_inputs() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let psbt1 = create_a_psbt(&node);
    let psbt2 = create_a_psbt(&node);

    let json: JoinPsbts =
        node.client.join_psbts(&[psbt1.clone(), psbt2.clone()]).expect("joinpsbts");
    let model: Result<mtype::JoinPsbts, psbt::PsbtParseError> = json.into_model();
    let joined = model.unwrap();

    // The joined PSBT should have inputs from both source PSBTs.
    assert_eq!(
        joined.0.inputs.len(),
        psbt1.inputs.len() + psbt2.inputs.len(),
        "joined PSBT should contain all inputs"
    );
}

/// utxoupdatepsbt should populate UTXO information for inputs.
/// Core checks that after update, non-witness UTXOs are properly filled.
#[test]
#[cfg(not(feature = "v17"))]
fn raw_transactions_core__utxo_update_psbt__populates_utxo() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let psbt = create_a_psbt(&node);

    let json: UtxoUpdatePsbt =
        node.client.utxo_update_psbt(&psbt).expect("utxoupdatepsbt");
    let model: Result<mtype::UtxoUpdatePsbt, psbt::PsbtParseError> = json.into_model();
    let updated = model.unwrap();

    // After updating, at least one input should have UTXO info.
    let has_utxo = updated.0.inputs.iter().any(|i| {
        i.witness_utxo.is_some() || i.non_witness_utxo.is_some()
    });
    assert!(has_utxo, "updated PSBT should have UTXO info for at least one input");
}

// ---------------------------------------------------------------------------
// fundrawtransaction: Core test vectors
// ---------------------------------------------------------------------------

/// fundrawtransaction should add change output and select inputs.
/// Core tests that fee, changepos, and inputs are correctly set.
#[test]
fn raw_transactions_core__fund_raw_transaction__adds_change() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let addr = node.client.new_address().expect("newaddress");
    let outputs = vec![Output::new(addr, Amount::from_sat(50_000))];

    // Create a tx with no inputs (fundrawtransaction will select them).
    let json: CreateRawTransaction =
        node.client.create_raw_transaction(&[], &outputs).expect("createrawtransaction");
    let raw_tx = json.transaction().unwrap();
    assert!(raw_tx.input.is_empty(), "created tx should have no inputs yet");

    let json: FundRawTransaction =
        node.client.fund_raw_transaction(&raw_tx).expect("fundrawtransaction");
    let model: Result<mtype::FundRawTransaction, FundRawTransactionError> = json.clone().into_model();
    let funded = model.unwrap();

    // fundrawtransaction should have added at least one input.
    let funded_tx = json.transaction().unwrap();
    assert!(!funded_tx.input.is_empty(), "funded tx should have inputs");

    // Should have added a change output (so total outputs > 1).
    assert!(funded_tx.output.len() >= 2, "funded tx should have at least 2 outputs (spend + change)");

    // Fee should be positive.
    assert!(funded.fee.to_sat() > 0, "fee should be positive");
}

// ---------------------------------------------------------------------------
// Helper functions (mirroring raw_transactions.rs helpers)
// ---------------------------------------------------------------------------

fn create_a_psbt(node: &BitcoinD) -> bitcoin::Psbt {
    let (_, tx) = node.create_mined_transaction();
    let txid = tx.compute_txid();

    let tx_out = node
        .client
        .get_tx_out(txid, 0)
        .expect("gettxout")
        .into_model()
        .expect("GetTxOut into model")
        .tx_out;

    let spend_amount = Amount::from_sat(100_000);
    let fee = Amount::from_sat(1000);
    let change_amount = tx_out.value - spend_amount - fee;

    let inputs = vec![Input { txid, vout: 0, sequence: None }];

    let spend_address = node.client.new_address().expect("new_address");
    let change_address = node
        .client
        .get_raw_change_address()
        .expect("getrawchangeaddress")
        .into_model()
        .expect("GetRawChangeAddress into model")
        .0
        .assume_checked();

    let outputs = vec![
        Output::new(spend_address, spend_amount),
        Output::new(change_address, change_amount),
    ];

    let json: CreatePsbt = node.client.create_psbt(&inputs, &outputs).expect("createpsbt");
    let model: Result<mtype::CreatePsbt, psbt::PsbtParseError> = json.into_model();
    model.unwrap().0
}
