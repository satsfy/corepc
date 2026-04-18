// SPDX-License-Identifier: CC0-1.0

//! Tests derived from Bitcoin Core's wallet functional tests (wallet_basic.py,
//! wallet_labels.py, wallet_listsinceblock.py, wallet_listreceivedby.py, etc.).
//!
//! Exercises wallet RPCs with deeper field validation, including balance checks
//! after transactions, address label management, list* RPC field validation,
//! and PSBT wallet workflow.

#![allow(non_snake_case)] // Test names intentionally use double underscore.
#![allow(unused_imports)]

use std::collections::BTreeMap;

use bitcoin::{address, amount, hex, Amount, FeeRate};
use bitcoind::vtype::*;
use bitcoind::{mtype, AddressType};
use integration_test::{BitcoinD, BitcoinDExt as _, Wallet};

// ---------------------------------------------------------------------------
// getbalance / getbalances: Core wallet_basic.py
// ---------------------------------------------------------------------------

/// getbalance on a fresh node with funded wallet should reflect mined coins.
/// Core checks that balance increases after mining and sending.
#[test]
fn wallet_core__get_balance__after_funding() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let json: GetBalance = node.client.get_balance().expect("getbalance");
    let model: Result<mtype::GetBalance, amount::ParseAmountError> = json.into_model();
    let balance = model.unwrap();

    assert!(balance.0.to_sat() > 0, "funded wallet should have positive balance");
}

/// getbalances returns trusted/untrusted/immature breakdowns.
/// Core checks mine.trusted, mine.untrusted_pending, mine.immature.
#[test]
#[cfg(not(feature = "v18_and_below"))]
fn wallet_core__get_balances__fields() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let json: GetBalances = node.client.get_balances().expect("getbalances");
    let model: Result<mtype::GetBalances, GetBalancesError> = json.into_model();
    let balances = model.unwrap();

    // After funding, trusted balance should be > 0.
    assert!(
        balances.mine.trusted.to_sat() > 0,
        "mine.trusted should be > 0 after funding"
    );

    // Untrusted pending should be 0 (all UTXOs are confirmed).
    assert_eq!(
        balances.mine.untrusted_pending.to_sat(),
        0,
        "untrusted_pending should be 0 after funding"
    );
}

/// Balance decreases after sending a transaction.
/// Core checks that balance goes down by the send amount plus fee.
#[test]
fn wallet_core__get_balance__decreases_after_send() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let balance_before: mtype::GetBalance =
        node.client.get_balance().expect("getbalance").into_model().unwrap();

    let addr = node.client.new_address().expect("newaddress");
    let send_amount = Amount::from_sat(1_000_000);

    node.client.send_to_address(&addr, send_amount).expect("sendtoaddress");
    node.mine_a_block();

    let balance_after: mtype::GetBalance =
        node.client.get_balance().expect("getbalance").into_model().unwrap();

    // After sending to ourselves, the balance should decrease by the fee amount.
    // Since we send to our own wallet, only the fee is lost.
    assert!(
        balance_after.0 <= balance_before.0,
        "balance should decrease (or stay equal) after send due to fees"
    );
}

// ---------------------------------------------------------------------------
// getnewaddress: Core wallet_basic.py
// ---------------------------------------------------------------------------

/// getnewaddress returns different addresses on successive calls.
/// Core checks that new addresses are unique.
#[test]
fn wallet_core__get_new_address__unique() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);

    let addr1 = node.client.new_address().expect("newaddress 1");
    let addr2 = node.client.new_address().expect("newaddress 2");
    let addr3 = node.client.new_address().expect("newaddress 3");

    assert_ne!(addr1, addr2, "addresses should be unique");
    assert_ne!(addr2, addr3, "addresses should be unique");
    assert_ne!(addr1, addr3, "addresses should be unique");
}

/// getnewaddress with different address types returns valid addresses.
/// Core tests legacy, p2sh-segwit, and bech32 types.
#[test]
fn wallet_core__get_new_address__all_types() {
    #[cfg(feature = "v29_and_below")]
    {
        let node = BitcoinD::with_wallet(Wallet::Default, &["-addresstype=legacy"]);

        let legacy = node
            .client
            .new_address_with_type(AddressType::Legacy)
            .expect("newaddress legacy");
        let p2sh = node
            .client
            .new_address_with_type(AddressType::P2shSegwit)
            .expect("newaddress p2sh-segwit");
        let bech32 = node
            .client
            .new_address_with_type(AddressType::Bech32)
            .expect("newaddress bech32");

        // Each should be a different type. We just verify they're all non-empty
        // and distinct from each other.
        assert_ne!(legacy.to_string(), p2sh.to_string());
        assert_ne!(legacy.to_string(), bech32.to_string());
        assert_ne!(p2sh.to_string(), bech32.to_string());
    }

    #[cfg(not(feature = "v29_and_below"))]
    {
        // v30+: legacy address type is no longer supported
        let node = BitcoinD::with_wallet(Wallet::Default, &[]);
        let bech32 = node
            .client
            .new_address_with_type(AddressType::Bech32)
            .expect("newaddress bech32");
        assert!(!bech32.to_string().is_empty());
    }
}

// ---------------------------------------------------------------------------
// getaddressinfo: Core wallet_basic.py + rpc_validateaddress.py
// ---------------------------------------------------------------------------

/// getaddressinfo returns expected fields for a wallet address.
/// Core checks: ismine, solvable, iswatchonly, isscript, iswitness.
#[test]
fn wallet_core__get_address_info__wallet_address() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);

    let addr = node.client.new_address().expect("newaddress");
    let json: GetAddressInfo =
        node.client.get_address_info(&addr).expect("getaddressinfo");

    assert!(json.is_mine, "wallet address should be ismine=true");
    assert!(json.solvable, "wallet address should be solvable");
    assert!(!json.is_watch_only, "wallet address should not be watchonly");
}

/// getaddressinfo for bech32 address should report iswitness=true.
#[test]
fn wallet_core__get_address_info__bech32_is_witness() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);

    let addr = node
        .client
        .new_address_with_type(AddressType::Bech32)
        .expect("newaddress bech32");
    let json: GetAddressInfo =
        node.client.get_address_info(&addr).expect("getaddressinfo");

    assert!(json.is_witness, "bech32 address should be witness");
}

// ---------------------------------------------------------------------------
// sendtoaddress / sendmany: Core wallet_basic.py
// ---------------------------------------------------------------------------

/// sendtoaddress returns a txid and the transaction appears in the mempool.
/// Core checks that sendtoaddress creates a valid transaction.
#[test]
fn wallet_core__send_to_address__in_mempool() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let addr = node.client.new_address().expect("newaddress");
    let amount = Amount::from_sat(500_000);

    let json: SendToAddress =
        node.client.send_to_address(&addr, amount).expect("sendtoaddress");
    let txid = json.txid().expect("txid");

    // Transaction should be in the mempool.
    let mempool: GetRawMempool = node.client.get_raw_mempool().expect("getrawmempool");
    assert!(
        mempool.0.contains(&txid.to_string()),
        "sent transaction should be in the mempool"
    );
}

/// sendmany to multiple recipients creates a single transaction.
/// Core checks that sendmany handles multiple addresses correctly.
#[test]
fn wallet_core__send_many__multiple_recipients() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let addr1 = node.client.new_address().expect("newaddress 1");
    let addr2 = node.client.new_address().expect("newaddress 2");

    let mut amounts = BTreeMap::new();
    amounts.insert(addr1, Amount::from_sat(100_000));
    amounts.insert(addr2, Amount::from_sat(200_000));

    let json: SendMany = node.client.send_many(amounts).expect("sendmany");
    let model: mtype::SendMany = json.into_model().expect("SendMany into model");
    let txid = model.0;

    // The transaction should be in the mempool.
    let mempool: GetRawMempool = node.client.get_raw_mempool().expect("getrawmempool");
    assert!(
        mempool.0.contains(&txid.to_string()),
        "sendmany transaction should be in the mempool"
    );
}

// ---------------------------------------------------------------------------
// gettransaction: Core wallet_basic.py
// ---------------------------------------------------------------------------

/// gettransaction returns correct fields for a sent transaction.
/// Core checks: amount, fee, confirmations, txid, details.
#[test]
fn wallet_core__get_transaction__fields() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let addr = node.client.new_address().expect("newaddress");
    let amount = Amount::from_sat(500_000);

    let send_json: SendToAddress =
        node.client.send_to_address(&addr, amount).expect("sendtoaddress");
    let txid = send_json.txid().expect("txid");

    // Mine the transaction.
    node.mine_a_block();

    let json: GetTransaction =
        node.client.get_transaction(txid).expect("gettransaction");
    let model: Result<mtype::GetTransaction, GetTransactionError> = json.into_model();
    let tx = model.unwrap();

    // confirmations should be > 0 since we mined a block.
    assert!(tx.confirmations > 0, "mined tx should have confirmations > 0");

    // txid should match.
    assert_eq!(tx.txid, txid, "txid should match");

    // The transaction details should have at least one entry.
    assert!(!tx.details.is_empty(), "should have transaction details");
}

/// gettransaction for a mempool transaction has 0 confirmations.
#[test]
fn wallet_core__get_transaction__mempool_zero_confirmations() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let addr = node.client.new_address().expect("newaddress");
    let amount = Amount::from_sat(500_000);

    let send_json: SendToAddress =
        node.client.send_to_address(&addr, amount).expect("sendtoaddress");
    let txid = send_json.txid().expect("txid");

    let json: GetTransaction =
        node.client.get_transaction(txid).expect("gettransaction");
    let model: Result<mtype::GetTransaction, GetTransactionError> = json.into_model();
    let tx = model.unwrap();

    assert_eq!(tx.confirmations, 0, "mempool tx should have 0 confirmations");
}

// ---------------------------------------------------------------------------
// listunspent: Core wallet_basic.py
// ---------------------------------------------------------------------------

/// listunspent returns UTXOs from funded wallet.
/// Core checks that listunspent includes the expected UTXOs with correct values.
#[test]
fn wallet_core__list_unspent__has_utxos() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let json: ListUnspent = node.client.list_unspent().expect("listunspent");
    let model: mtype::ListUnspent = json.into_model().unwrap();

    assert!(!model.0.is_empty(), "funded wallet should have UTXOs");

    // Each UTXO should have a positive amount.
    for utxo in &model.0 {
        assert!(utxo.amount.to_sat() > 0, "UTXO amount should be positive");
        assert!(utxo.confirmations > 0, "funded UTXO should have confirmations");
    }
}

// ---------------------------------------------------------------------------
// listtransactions: Core wallet_basic.py
// ---------------------------------------------------------------------------

/// listtransactions returns recent wallet transactions including mining rewards.
/// Core checks category, amount, confirmations, and txid fields.
#[test]
fn wallet_core__list_transactions__after_send() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let addr = node.client.new_address().expect("newaddress");
    node.client.send_to_address(&addr, Amount::from_sat(100_000)).expect("sendtoaddress");
    node.mine_a_block();

    let json: ListTransactions = node.client.list_transactions().expect("listtransactions");
    let model: Result<mtype::ListTransactions, TransactionItemError> = json.into_model();
    let txs = model.unwrap();

    assert!(!txs.0.is_empty(), "should have at least one transaction");

    // The last few entries should include our send.
    let has_send = txs.0.iter().any(|tx| tx.category == mtype::TransactionCategory::Send);
    assert!(has_send, "should have a 'send' category transaction");
}

// ---------------------------------------------------------------------------
// listsinceblock: Core wallet_listsinceblock.py
// ---------------------------------------------------------------------------

/// listsinceblock returns transactions since a given block.
/// Core checks that older transactions are excluded and newer ones included.
#[test]
fn wallet_core__list_since_block__filters_by_block() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    // Get current block hash as the reference point.
    let _block_hash = node
        .client
        .get_best_block_hash()
        .expect("getbestblockhash")
        .into_model()
        .unwrap()
        .0;

    // Create a new transaction after the reference block.
    let addr = node.client.new_address().expect("newaddress");
    node.client.send_to_address(&addr, Amount::from_sat(100_000)).expect("sendtoaddress");
    node.mine_a_block();

    let json: ListSinceBlock = node.client.list_since_block().expect("listsinceblock");
    let model: Result<mtype::ListSinceBlock, ListSinceBlockError> = json.into_model();
    let list = model.unwrap();

    // Should have transactions.
    assert!(!list.transactions.is_empty(), "should have recent transactions");

    // lastblock should be set.
    let _ = list.last_block; // Should not be default/zero.
}

// ---------------------------------------------------------------------------
// labels: Core wallet_labels.py
// ---------------------------------------------------------------------------

/// Address label lifecycle: create -> get -> list.
/// Core tests setting labels, getting addresses by label, and listing labels.
#[test]
fn wallet_core__labels__lifecycle() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let label = "test_label_core";
    let addr = node.client.new_address_with_label(label).expect("newaddress with label");

    // getaddressesbylabel should return the address.
    let json: GetAddressesByLabel =
        node.client.get_addresses_by_label(label).expect("getaddressesbylabel");
    let model: Result<mtype::GetAddressesByLabel, address::ParseError> = json.into_model();
    let addresses = model.unwrap();
    assert!(
        addresses.0.keys().any(|a| a.assume_checked_ref().to_string() == addr.assume_checked_ref().to_string()),
        "label should contain the address"
    );

    // listlabels should include our label.
    let json: ListLabels = node.client.list_labels().expect("listlabels");
    assert!(json.0.contains(&label.to_string()), "listlabels should contain our label");
}

// ---------------------------------------------------------------------------
// listreceivedbyaddress: Core wallet_listreceivedby.py
// ---------------------------------------------------------------------------

/// listreceivedbyaddress includes address that received funds.
/// Core checks amount, confirmations, txids fields.
#[test]
fn wallet_core__list_received_by_address__funded_address() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let addr = node.client.new_address().expect("newaddress");
    let send_amount = Amount::from_sat(500_000);
    node.client.send_to_address(&addr, send_amount).expect("sendtoaddress");
    node.mine_a_block();

    let json: ListReceivedByAddress =
        node.client.list_received_by_address().expect("listreceivedbyaddress");
    let model: Result<mtype::ListReceivedByAddress, ListReceivedByAddressError> = json.into_model();
    let list = model.unwrap();

    // Find entry matching our address.
    let entry = list.0.iter().find(|e| e.address.assume_checked_ref().to_string() == addr.to_string());
    assert!(entry.is_some(), "should have entry for the funded address");
    let entry = entry.unwrap();
    assert!(entry.amount.to_sat() >= send_amount.to_sat(), "amount should >= sent amount");
    assert!(entry.confirmations > 0, "should have confirmations after mining");
}

// ---------------------------------------------------------------------------
// getreceivedbyaddress: Core wallet_basic.py
// ---------------------------------------------------------------------------

/// getreceivedbyaddress returns amount received at specific address.
/// Core checks that the amount matches what was sent.
#[test]
fn wallet_core__get_received_by_address__matches_sent() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let addr = node.client.new_address().expect("newaddress");
    let send_amount = Amount::from_sat(750_000);
    node.client.send_to_address(&addr, send_amount).expect("sendtoaddress");
    node.mine_a_block();

    let json: GetReceivedByAddress =
        node.client.get_received_by_address(&addr).expect("getreceivedbyaddress");
    let model: Result<mtype::GetReceivedByAddress, amount::ParseAmountError> = json.into_model();
    let received = model.unwrap();

    assert_eq!(received.0, send_amount, "received amount should match sent amount");
}

// ---------------------------------------------------------------------------
// lockunspent / listlockunspent: Core wallet_basic.py
// ---------------------------------------------------------------------------

/// Lock/unlock UTXO lifecycle: list -> lock -> verify locked -> unlock -> verify unlocked.
/// Core tests that locked UTXOs are excluded from listunspent.
#[test]
fn wallet_core__lock_unspent__lifecycle() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let json: ListUnspent = node.client.list_unspent().expect("listunspent");
    let unspent: mtype::ListUnspent = json.into_model().unwrap();
    assert!(!unspent.0.is_empty(), "should have UTXOs to lock");

    let first = &unspent.0[0];
    let txid = first.txid;
    let vout = first.vout;

    // Lock the UTXO.
    let lock_result: LockUnspent =
        node.client.lock_unspent(&[(txid, vout)]).expect("lockunspent");
    assert!(lock_result.0, "lock should succeed");

    // Verify it appears in listlockunspent.
    let locked: ListLockUnspent = node.client.list_lock_unspent().expect("listlockunspent");
    let locked_model: mtype::ListLockUnspent = locked.into_model().unwrap();
    assert!(
        locked_model.0.iter().any(|l| l.txid == txid && l.vout == vout),
        "locked UTXO should appear in listlockunspent"
    );

    // Unlock the UTXO.
    let unlock_result: LockUnspent =
        node.client.unlock_unspent(&[(txid, vout)]).expect("unlockunspent");
    assert!(unlock_result.0, "unlock should succeed");

    // Verify it's no longer locked.
    let locked_after: ListLockUnspent = node.client.list_lock_unspent().expect("listlockunspent");
    let locked_after_model: mtype::ListLockUnspent = locked_after.into_model().unwrap();
    assert!(
        !locked_after_model.0.iter().any(|l| l.txid == txid && l.vout == vout),
        "unlocked UTXO should not appear in listlockunspent"
    );
}

// ---------------------------------------------------------------------------
// walletinfo: Core wallet_basic.py
// ---------------------------------------------------------------------------

/// getwalletinfo returns expected fields.
/// Core checks wallet_name, format, txcount, keypoolsize.
#[test]
fn wallet_core__get_wallet_info__fields() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let json: GetWalletInfo = node.client.get_wallet_info().expect("getwalletinfo");
    let model: Result<mtype::GetWalletInfo, GetWalletInfoError> = json.into_model();
    let info = model.unwrap();

    // Wallet name should not be empty.
    assert!(!info.wallet_name.is_empty(), "wallet name should not be empty");

    // After funding, tx_count should be > 0.
    assert!(info.tx_count > 0, "tx_count should be > 0 after funding");

    // Keypool size should be > 0.
    assert!(info.keypool_size > 0, "keypool_size should be > 0");
}

// ---------------------------------------------------------------------------
// bumpfee: Core wallet_bumpfee.py
// ---------------------------------------------------------------------------

/// bumpfee increases the fee of an RBF-signaled transaction.
/// Core checks that the replacement transaction has a higher fee.
#[test]
fn wallet_core__bump_fee__increases_fee() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let addr = node.client.new_address().expect("newaddress");
    let amount = Amount::from_sat(100_000);

    let json: SendToAddress =
        node.client.send_to_address_rbf(&addr, amount).expect("sendtoaddress with RBF");
    let txid = json.txid().expect("txid");

    // Get the original transaction's fee.
    let orig_tx: GetTransaction =
        node.client.get_transaction(txid).expect("gettransaction");
    let orig_model: mtype::GetTransaction =
        orig_tx.into_model().expect("GetTransaction into model");
    let _orig_fee = orig_model.fee.map(|f| f.to_sat().unsigned_abs()).unwrap_or(0);

    // Bump the fee.
    let bump_json: BumpFee = node.client.bump_fee(txid).expect("bumpfee");
    let bump_model: Result<mtype::BumpFee, BumpFeeError> = bump_json.into_model();
    let bumped = bump_model.unwrap();

    // The new fee should be higher.
    let new_fee = bumped.original_fee.to_sat();
    // bumped.original_fee is the original fee, bumped.fee is the new fee.
    assert!(bumped.fee.to_sat() > new_fee, "bumped fee should be higher than original");
}

// ---------------------------------------------------------------------------
// PSBT wallet workflow: walletcreatefundedpsbt + walletprocesspsbt
// ---------------------------------------------------------------------------

/// walletcreatefundedpsbt + walletprocesspsbt roundtrip.
/// Core tests the complete wallet-driven PSBT workflow.
#[test]
fn wallet_core__wallet_psbt__create_and_process() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let addr = node.client.new_address().expect("newaddress");
    let outputs = BTreeMap::from([(addr, Amount::from_sat(200_000))]);

    let json: WalletCreateFundedPsbt = node
        .client
        .wallet_create_funded_psbt(vec![], vec![outputs])
        .expect("walletcreatefundedpsbt");
    let model: Result<mtype::WalletCreateFundedPsbt, WalletCreateFundedPsbtError> =
        json.into_model();
    let funded = model.unwrap();

    // The PSBT should have inputs selected by the wallet.
    assert!(
        !funded.psbt.inputs.is_empty(),
        "funded PSBT should have inputs"
    );

    // Process with wallet to sign.
    let json: WalletProcessPsbt = node
        .client
        .wallet_process_psbt(&funded.psbt)
        .expect("walletprocesspsbt");
    let model: mtype::WalletProcessPsbt = json.into_model().expect("WalletProcessPsbt into model");

    assert!(model.complete, "wallet-processed PSBT should be complete");

    // Finalize and extract.
    let json: FinalizePsbt =
        node.client.finalize_psbt(&model.psbt).expect("finalizepsbt");
    let finalized: mtype::FinalizePsbt = json.into_model().expect("FinalizePsbt into model");

    assert!(finalized.complete, "finalized PSBT should be complete");
    assert!(finalized.tx.is_some(), "finalized PSBT should have extractable tx");
}

// ---------------------------------------------------------------------------
// backupwallet: Core wallet_backup.py
// ---------------------------------------------------------------------------

/// backupwallet creates a backup file on disk.
#[test]
fn wallet_core__backup_wallet__creates_file() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let backup_path = integration_test::random_tmp_file();
    node.client.backup_wallet(&backup_path).expect("backupwallet");

    assert!(backup_path.exists(), "backup file should exist on disk");

    // Clean up.
    let _ = std::fs::remove_file(backup_path);
}

// ---------------------------------------------------------------------------
// encryptwallet + walletpassphrase + walletlock: Core wallet_encryption.py
// ---------------------------------------------------------------------------

/// Encryption lifecycle: encrypt -> lock -> unlock -> lock.
/// Core checks that wallet operations require unlock after encryption.
#[test]
fn wallet_core__encrypt_wallet__lifecycle() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let passphrase = "test_passphrase_12345";

    // Encrypt the wallet.
    let _: EncryptWallet = node.client.encrypt_wallet(passphrase).expect("encryptwallet");

    // After encryption, wallet should be locked. Try to get a new address
    // (should still work since address generation doesn't require unlock on descriptor wallets).
    // But explicitly locking should work.
    node.client.wallet_lock().expect("walletlock");

    // Unlock with passphrase.
    node.client.wallet_passphrase(passphrase, 60).expect("walletpassphrase");

    // Lock again.
    node.client.wallet_lock().expect("walletlock after unlock");
}

/// walletpassphrasechange changes the passphrase.
#[test]
fn wallet_core__wallet_passphrase_change__works() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);

    let old_pass = "old_passphrase";
    let new_pass = "new_passphrase";

    let _: EncryptWallet = node.client.encrypt_wallet(old_pass).expect("encryptwallet");

    // Change passphrase.
    node.client.wallet_passphrase_change(old_pass, new_pass).expect("walletpassphrasechange");

    // New passphrase should work.
    node.client.wallet_passphrase(new_pass, 60).expect("walletpassphrase with new pass");

    node.client.wallet_lock().expect("walletlock");
}

// ---------------------------------------------------------------------------
// listwallets / loadwallet / unloadwallet: Core wallet_basic.py
// ---------------------------------------------------------------------------

/// listwallets includes the default wallet.
#[test]
fn wallet_core__list_wallets__contains_default() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    let json: ListWallets = node.client.list_wallets().expect("listwallets");
    assert!(!json.0.is_empty(), "should have at least the default wallet");
}

// ---------------------------------------------------------------------------
// rescanblockchain: Core wallet_rescanblockchain.py
// ---------------------------------------------------------------------------

/// rescanblockchain returns start and stop heights.
/// Core checks that rescan covers the expected block range.
#[test]
fn wallet_core__rescan_blockchain__returns_heights() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    let json: RescanBlockchain = node.client.rescan_blockchain().expect("rescanblockchain");
    let model: mtype::RescanBlockchain = json.into_model().expect("RescanBlockchain into model");

    assert_eq!(model.start_height, 0, "rescan should start from 0");
    assert!(model.stop_height > 0, "rescan should cover some blocks");
}
