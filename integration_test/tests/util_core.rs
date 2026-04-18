// SPDX-License-Identifier: CC0-1.0

//! Tests derived from Bitcoin Core's `rpc_validateaddress.py`, `rpc_deriveaddresses.py`,
//! `rpc_getdescriptorinfo.py`, `rpc_signmessagewithprivkey.py`, `rpc_misc.py`,
//! `rpc_estimatefee.py`, and `rpc_createmultisig.py` test vectors.
//!
//! Exercises a wider range of inputs for the util RPCs, including address
//! validation edge cases, descriptor derivation, and sign/verify message flows.

#![allow(non_snake_case)] // Test names intentionally use double underscore.
#![allow(unused_imports)]

use bitcoin::{address, amount, sign_message, PrivateKey, PublicKey};
use bitcoind::vtype::*;
use bitcoind::{mtype, FeeEstimateMode};
use integration_test::{BitcoinD, BitcoinDExt as _, Wallet};

// ---------------------------------------------------------------------------
// validateaddress: Core BIP 173 + BIP 350 test vectors
// (from rpc_validateaddress.py)
// ---------------------------------------------------------------------------

/// validateaddress rejects known-invalid BIP 173 / BIP 350 addresses.
#[test]
fn util_core__validate_address__invalid_bech32() {
    let node = BitcoinD::with_wallet(Wallet::None, &[]);

    // A selection of invalid addresses from Core's rpc_validateaddress.py INVALID_DATA.
    // These test various failure modes: wrong hrp, wrong checksum, mixed case, etc.
    let invalid_addresses = [
        // Wrong hrp
        "tc1qw508d6qejxtdg4y5r3zarvary0c5xw7kg3g4ty",
        // Invalid Bech32 checksum
        "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t5",
        // Invalid program length (v1+ must use Bech32m)
        "bc1rw5uspcuh",
        // Invalid Bech32 v0 address program size
        "BC1QR508D6QEJXTDG4Y5R3ZARVARYV98GJ9P",
        // Empty Bech32 data section
        "bc1gmk9yu",
        // Bech32 instead of Bech32m for v1+
        "bc1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vqh2y7hd",
        // Bech32m instead of Bech32 for v0
        "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kemeawh",
        // Invalid program size (1 byte)
        "bc1pw5dgrnzv",
        // Invalid program size (41 bytes)
        "bc1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7v8n0nx0muaewav253zgeav",
        // Invalid witness version
        "BC130XLXVLHEMJA6C4DQV22UAPCTQUPFHLXM9H8Z3K2E72Q4K9HCZ7VQ7ZWS8R",
    ];

    for addr_str in &invalid_addresses {
        // These are invalid mainnet addresses, so they shouldn't parse as valid
        // regtest addresses either. The important thing is that our validate_address
        // RPC handles them without crashing.
        // We use call() directly since validate_address requires a NetworkChecked address.
        let result: bitcoind::serde_json::Value = node
            .client
            .call("validateaddress", &[bitcoind::serde_json::json!(addr_str)])
            .expect("validateaddress should not error on invalid input");
        assert_eq!(
            result["isvalid"].as_bool().unwrap(),
            false,
            "address '{}' should be invalid",
            addr_str
        );
    }
}

/// validateaddress accepts known-valid BIP 173 / BIP 350 addresses.
/// We test regtest equivalents since Core runs on regtest.
#[test]
fn util_core__validate_address__valid_wallet_address() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    // Get addresses from the wallet (which will be valid regtest addresses).
    let addr = node.client.new_address().expect("new_address");
    let json: ValidateAddress = node.client.validate_address(&addr).expect("validateaddress");
    let model: Result<mtype::ValidateAddress, ValidateAddressError> = json.into_model();
    let result = model.unwrap();

    assert!(result.is_valid, "wallet address should be valid");
    assert!(
        !result.address.assume_checked().to_string().is_empty(),
        "valid address should have address field"
    );
    assert!(!result.script_pubkey.is_empty(), "valid address should have scriptPubKey");
    assert!(result.is_witness, "modern wallet addresses should be witness");
}

// ---------------------------------------------------------------------------
// signmessagewithprivkey + verifymessage: Core test vectors
// (from rpc_signmessagewithprivkey.py)
// ---------------------------------------------------------------------------

/// signmessagewithprivkey produces correct signature that verifies.
/// Core vector: privkey cUeKHd5orzT3mz8P9pxyREHfsWtVfgsfDjiZZBcjUBAaGk1BTj7N,
/// expected P2PKH address mpLQjfK79b7CCV4VMJWEWAj5Mpx8Up5zxB (testnet).
#[test]
fn util_core__sign_message_with_privkey__core_vector() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    let message = "This is just a test message";

    // Use the Core test vector private key.
    let privkey =
        PrivateKey::from_wif("cUeKHd5orzT3mz8P9pxyREHfsWtVfgsfDjiZZBcjUBAaGk1BTj7N").unwrap();

    let json: SignMessageWithPrivKey =
        node.client.sign_message_with_privkey(&privkey, message).expect("signmessagewithprivkey");
    let model: Result<mtype::SignMessageWithPrivKey, sign_message::MessageSignatureError> =
        json.into_model();
    let sig = model.unwrap();

    // The expected signature from Core's test:
    let expected_sig =
        "INbVnW4e6PeRmsv2Qgu8NuopvrVjkcxob+sX8OcZG0SALhWybUjzMLPdAsXI46YZGb0KQTRii+wWIQzRpG/U+S0=";
    assert_eq!(sig.0.to_string(), expected_sig, "signature should match Core's expected value");

    // Derive P2PKH address and verify.
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let pubkey = privkey.public_key(&secp);
    let addr = bitcoin::Address::p2pkh(pubkey, privkey.network);

    let verified: VerifyMessage =
        node.client.verify_message(&addr, &sig.0, message).expect("verifymessage");
    assert!(verified.0, "signature should verify with the correct P2PKH address");
}

/// signmessagewithprivkey + verifymessage roundtrip with a different message.
#[test]
fn util_core__sign_verify_message__roundtrip() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);

    let privkey =
        PrivateKey::from_wif("cVt4o7BGAig1UXywgGSmARhxMdzP5qvQsxKkSsc1XEkw3tDTQFpy").unwrap();
    let messages = [
        "Hello, Bitcoin!",
        "",
        "A longer message with special chars: !@#$%^&*()",
        "Unicode: ñ α β γ",
    ];

    let secp = bitcoin::secp256k1::Secp256k1::new();
    let pubkey = privkey.public_key(&secp);
    let addr = bitcoin::Address::p2pkh(pubkey, privkey.network);

    for message in &messages {
        let json: SignMessageWithPrivKey = node
            .client
            .sign_message_with_privkey(&privkey, message)
            .expect("signmessagewithprivkey");
        let sig = json.into_model().unwrap();

        let verified: VerifyMessage =
            node.client.verify_message(&addr, &sig.0, message).expect("verifymessage");
        assert!(verified.0, "signature should verify for message: {:?}", message);
    }
}

// ---------------------------------------------------------------------------
// deriveaddresses: Core test vectors
// (from rpc_deriveaddresses.py)
// ---------------------------------------------------------------------------

/// deriveaddresses derives correct address from a wpkh descriptor.
#[test]
#[cfg(not(feature = "v17"))]
fn util_core__derive_addresses__wpkh_descriptor() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);

    // Core test vector: wpkh descriptor with private key.
    let descriptor = "wpkh(tprv8ZgxMBicQKsPd7Uf69XL1XwhmjHopUGep8GuEiJDZmbQz6o58LninorQAfcKZWARbtRtfnLcJ5MQ2AtHcQJCCRUcMRvmDUjyEmNUWwx8UbK/1/1/0)#t6wfjs64";
    let expected_address = "bcrt1qjqmxmkpmxt80xz4y3746zgt0q3u3ferr34acd5";

    let json: DeriveAddresses = node.client.derive_addresses(descriptor).expect("deriveaddresses");
    let model: Result<mtype::DeriveAddresses, address::ParseError> = json.into_model();
    let addresses = model.unwrap();

    assert_eq!(addresses.addresses.len(), 1);
    assert_eq!(
        addresses.addresses[0].assume_checked_ref().to_string(),
        expected_address,
        "derived address should match Core's expected value"
    );
}

/// deriveaddresses with public key descriptor gives same result.
#[test]
#[cfg(not(feature = "v17"))]
fn util_core__derive_addresses__wpkh_pubkey_descriptor() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);

    let descriptor = "wpkh(tpubD6NzVbkrYhZ4WaWSyoBvQwbpLkojyoTZPRsgXELWz3Popb3qkjcJyJUGLnL4qHHoQvao8ESaAstxYSnhyswJ76uZPStJRJCTKvosUCJZL5B/1/1/0)#s9ga3alw";
    let expected_address = "bcrt1qjqmxmkpmxt80xz4y3746zgt0q3u3ferr34acd5";

    let json: DeriveAddresses = node.client.derive_addresses(descriptor).expect("deriveaddresses");
    let model: Result<mtype::DeriveAddresses, address::ParseError> = json.into_model();
    let addresses = model.unwrap();

    assert_eq!(addresses.addresses.len(), 1);
    assert_eq!(addresses.addresses[0].assume_checked_ref().to_string(), expected_address);
}

/// deriveaddresses with a ranged descriptor.
#[test]
#[cfg(not(feature = "v17"))]
fn util_core__derive_addresses__ranged_descriptor() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);

    // Core test: ranged descriptor with range [1, 2].
    let descriptor = "wpkh(tprv8ZgxMBicQKsPd7Uf69XL1XwhmjHopUGep8GuEiJDZmbQz6o58LninorQAfcKZWARbtRtfnLcJ5MQ2AtHcQJCCRUcMRvmDUjyEmNUWwx8UbK/1/1/*)#kft60nuy";

    // Use call() directly since derive_addresses doesn't take a range param in the simple form.
    let result: Vec<String> = node
        .client
        .call(
            "deriveaddresses",
            &[bitcoind::serde_json::json!(descriptor), bitcoind::serde_json::json!([1, 2])],
        )
        .expect("deriveaddresses with range");

    let expected = [
        "bcrt1qhku5rq7jz8ulufe2y6fkcpnlvpsta7rq4442dy",
        "bcrt1qpgptk2gvshyl0s9lqshsmx932l9ccsv265tvaq",
    ];
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], expected[0]);
    assert_eq!(result[1], expected[1]);
}

// ---------------------------------------------------------------------------
// getdescriptorinfo: Core test vectors for various descriptor types
// (from rpc_getdescriptorinfo.py)
// ---------------------------------------------------------------------------

/// getdescriptorinfo for P2PK descriptor.
#[test]
#[cfg(not(feature = "v17"))]
fn util_core__get_descriptor_info__p2pk() {
    let node = BitcoinD::with_wallet(Wallet::None, &[]);
    let descriptor = "pk(0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798)";

    let json: GetDescriptorInfo =
        node.client.get_descriptor_info(descriptor).expect("getdescriptorinfo");
    assert!(!json.is_range);
    assert!(json.is_solvable);
    assert!(!json.has_private_keys);
}

/// getdescriptorinfo for P2PKH descriptor.
#[test]
#[cfg(not(feature = "v17"))]
fn util_core__get_descriptor_info__p2pkh() {
    let node = BitcoinD::with_wallet(Wallet::None, &[]);
    let descriptor = "pkh(02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5)";

    let json: GetDescriptorInfo =
        node.client.get_descriptor_info(descriptor).expect("getdescriptorinfo");
    assert!(!json.is_range);
    assert!(json.is_solvable);
    assert!(!json.has_private_keys);
}

/// getdescriptorinfo for P2WPKH descriptor.
#[test]
#[cfg(not(feature = "v17"))]
fn util_core__get_descriptor_info__p2wpkh() {
    let node = BitcoinD::with_wallet(Wallet::None, &[]);
    let descriptor = "wpkh(02f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9)";

    let json: GetDescriptorInfo =
        node.client.get_descriptor_info(descriptor).expect("getdescriptorinfo");
    assert!(!json.is_range);
    assert!(json.is_solvable);
    assert!(!json.has_private_keys);
}

/// getdescriptorinfo for SH-WPKH descriptor.
#[test]
#[cfg(not(feature = "v17"))]
fn util_core__get_descriptor_info__sh_wpkh() {
    let node = BitcoinD::with_wallet(Wallet::None, &[]);
    let descriptor = "sh(wpkh(03fff97bd5755eeea420453a14355235d382f6472f8568a18b2f057a1460297556))";

    let json: GetDescriptorInfo =
        node.client.get_descriptor_info(descriptor).expect("getdescriptorinfo");
    assert!(!json.is_range);
    assert!(json.is_solvable);
    assert!(!json.has_private_keys);
}

/// getdescriptorinfo for combo descriptor.
#[test]
#[cfg(not(feature = "v17"))]
fn util_core__get_descriptor_info__combo() {
    let node = BitcoinD::with_wallet(Wallet::None, &[]);
    let descriptor = "combo(0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798)";

    let json: GetDescriptorInfo =
        node.client.get_descriptor_info(descriptor).expect("getdescriptorinfo");
    assert!(!json.is_range);
    assert!(json.is_solvable);
    assert!(!json.has_private_keys);
}

/// getdescriptorinfo for a ranged xpub descriptor.
#[test]
#[cfg(not(feature = "v17"))]
fn util_core__get_descriptor_info__ranged_xpub() {
    let node = BitcoinD::with_wallet(Wallet::None, &[]);
    let descriptor = "pkh([d34db33f/44h/0h/0h]tpubD6NzVbkrYhZ4WaWSyoBvQwbpLkojyoTZPRsgXELWz3Popb3qkjcJyJUGLnL4qHHoQvao8ESaAstxYSnhyswJ76uZPStJRJCTKvosUCJZL5B/1/*)";

    let json: GetDescriptorInfo =
        node.client.get_descriptor_info(descriptor).expect("getdescriptorinfo");
    assert!(json.is_range, "ranged descriptor should report isrange=true");
    assert!(json.is_solvable);
    assert!(!json.has_private_keys);
}

/// getdescriptorinfo for P2WSH multisig.
#[test]
#[cfg(not(feature = "v17"))]
fn util_core__get_descriptor_info__wsh_multisig() {
    let node = BitcoinD::with_wallet(Wallet::None, &[]);
    let descriptor = "wsh(multi(2,03a0434d9e47f3c86235477c7b1ae6ae5d3442d49b1943c2b752a68e2a47e247c7,03774ae7f858a9411e5ef4246b70c65aac5649980be5c17891bbec17895da008cb,03d01115d548e7561b15c38f004d734633687cf4419620095bc5b0f47070afe85a))";

    let json: GetDescriptorInfo =
        node.client.get_descriptor_info(descriptor).expect("getdescriptorinfo");
    assert!(!json.is_range);
    assert!(json.is_solvable);
    assert!(!json.has_private_keys);
}

// ---------------------------------------------------------------------------
// createmultisig: Core tests creation of 2-of-3 multisig
// (from rpc_createmultisig.py)
// ---------------------------------------------------------------------------

/// createmultisig with three deterministic keys produces valid result.
#[test]
fn util_core__create_multisig__2_of_3() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);

    let pubkey1 = "02ff12471208c14bd580709cb2358d98975247d8765f92bc25eab3b2763ed605f8"
        .parse::<PublicKey>()
        .unwrap();
    let pubkey2 = "02fe6f0a5a297eb38c391581c4413e084773ea23954d93f7753db7dc0adc188b2f"
        .parse::<PublicKey>()
        .unwrap();
    let pubkey3 = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"
        .parse::<PublicKey>()
        .unwrap();

    let json: CreateMultisig =
        node.client.create_multisig(2, vec![pubkey1, pubkey2, pubkey3]).expect("createmultisig");
    let model: Result<mtype::CreateMultisig, CreateMultisigError> = json.into_model();
    let multisig = model.unwrap();

    // The result should have a valid address and redeem script.
    assert!(!multisig.redeem_script.is_empty(), "redeemScript should not be empty");
}

/// createmultisig with 1-of-1 is valid.
#[test]
fn util_core__create_multisig__1_of_1() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);

    let pubkey = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"
        .parse::<PublicKey>()
        .unwrap();

    let json: CreateMultisig =
        node.client.create_multisig(1, vec![pubkey]).expect("createmultisig 1-of-1");
    let model: Result<mtype::CreateMultisig, CreateMultisigError> = json.into_model();
    model.unwrap();
}

// ---------------------------------------------------------------------------
// estimatesmartfee: Core tests with various confirmation targets
// (from rpc_estimatefee.py)
// ---------------------------------------------------------------------------

/// estimatesmartfee with different confirmation targets all return valid results.
#[test]
fn util_core__estimate_smart_fee__various_targets() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    // Core tests different confirmation targets.
    for target in [1, 2, 6, 25, 144, 1008] {
        let json: EstimateSmartFee =
            node.client.estimate_smart_fee(target).expect("estimatesmartfee");
        let model: Result<mtype::EstimateSmartFee, amount::ParseAmountError> = json.into_model();
        let result = model.unwrap();

        // In regtest with few transactions, fee estimation may not have enough data,
        // so fee_rate may be None. But blocks should always be present.
        assert!(result.blocks > 0, "blocks should be > 0 for target {}", target);
    }
}

/// estimatesmartfee with all fee estimate modes.
#[test]
fn util_core__estimate_smart_fee__all_modes() {
    let node = BitcoinD::with_wallet(Wallet::Default, &[]);
    node.fund_wallet();

    for mode in [FeeEstimateMode::Unset, FeeEstimateMode::Economical, FeeEstimateMode::Conservative]
    {
        let json: EstimateSmartFee =
            node.client.estimate_smart_fee_with_mode(6, mode).expect("estimatesmartfee");
        let model: Result<mtype::EstimateSmartFee, amount::ParseAmountError> = json.into_model();
        model.unwrap();
    }
}

// ---------------------------------------------------------------------------
// getindexinfo: Core tests with various index types enabled
// (from rpc_misc.py)
// ---------------------------------------------------------------------------

/// getindexinfo without any indices returns empty.
#[test]
#[cfg(not(feature = "v20_and_below"))]
fn util_core__get_index_info__empty() {
    let node = BitcoinD::with_wallet(Wallet::None, &[]);
    let json: GetIndexInfo = node.client.get_index_info().expect("getindexinfo");
    assert!(json.0.is_empty(), "no indices enabled, should be empty");
}

/// getindexinfo with txindex reports synced status.
#[test]
#[cfg(not(feature = "v20_and_below"))]
fn util_core__get_index_info__with_txindex() {
    let node = BitcoinD::with_wallet(Wallet::Default, &["-txindex"]);
    // Give time for the index to sync.
    std::thread::sleep(std::time::Duration::from_millis(500));

    let json: GetIndexInfo = node.client.get_index_info().expect("getindexinfo");
    let txindex = json.0.get("txindex").expect("txindex should be present");
    assert!(txindex.synced || txindex.best_block_height > 0);
}
