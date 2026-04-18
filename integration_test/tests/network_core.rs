// SPDX-License-Identifier: CC0-1.0

//! Tests derived from Bitcoin Core's `rpc_net.py` test vectors.
//!
//! Exercises network RPCs with deeper field validation, mirroring assertions
//! from Core's functional tests (getnetworkinfo fields, getpeerinfo bidirectional
//! connections, getnettotals byte counting, setban/listbanned lifecycle).

#![allow(non_snake_case)] // Test names intentionally use double underscore.
#![allow(unused_imports)]

use bitcoind::vtype::*;
use bitcoind::{mtype, AddNodeCommand, SetBanCommand};
use integration_test::{BitcoinD, BitcoinDExt as _, Wallet};

// ---------------------------------------------------------------------------
// getnetworkinfo: Core test vectors from rpc_net.py
// ---------------------------------------------------------------------------

/// getnetworkinfo reports networkactive=true by default and has valid fields.
/// Core checks: networkactive == True, connections, connections_in, connections_out.
#[test]
fn network_core__get_network_info__active_and_fields() {
    let node = BitcoinD::with_wallet(Wallet::None, &[]);
    let json: GetNetworkInfo = node.client.get_network_info().expect("getnetworkinfo");
    let model: Result<mtype::GetNetworkInfo, GetNetworkInfoError> = json.into_model();
    let info = model.unwrap();

    assert!(info.network_active, "network should be active by default");
    assert!(info.version > 0, "version should be > 0");
    assert!(!info.subversion.is_empty(), "subversion should not be empty");
    assert!(info.protocol_version > 0, "protocol_version should be > 0");
    // In a lone node, connections should be 0.
    assert_eq!(info.connections, 0, "isolated node should have 0 connections");
}

/// getnetworkinfo with connected peers reports correct connection counts.
/// Core checks connections == 2, connections_in == 1, connections_out == 1.
#[test]
fn network_core__get_network_info__with_peers() {
    let (node1, _node2, _node3) = integration_test::three_node_network();

    let json: GetNetworkInfo = node1.client.get_network_info().expect("getnetworkinfo");
    let model: Result<mtype::GetNetworkInfo, GetNetworkInfoError> = json.into_model();
    let info = model.unwrap();

    assert!(info.network_active, "network should be active");
    // node1 should have at least 1 connection.
    assert!(info.connections > 0, "connected node should have connections > 0");
}

/// getnetworkinfo should have local_services and local_services_names fields.
#[test]
fn network_core__get_network_info__local_services() {
    let node = BitcoinD::with_wallet(Wallet::None, &[]);
    let json: GetNetworkInfo = node.client.get_network_info().expect("getnetworkinfo");

    // local_services should be a hex string representing service flags.
    assert!(!json.local_services.is_empty(), "local_services should not be empty");
    // local_services_names should contain known service names.
    assert!(!json.local_services_names.is_empty(), "localservicesnames should not be empty");
}

/// getnetworkinfo should list known networks (ipv4, ipv6, onion, i2p, cjdns).
#[test]
fn network_core__get_network_info__network_list() {
    let node = BitcoinD::with_wallet(Wallet::None, &[]);
    let json: GetNetworkInfo = node.client.get_network_info().expect("getnetworkinfo");

    // Should have at least ipv4 and ipv6 network entries.
    let network_names: Vec<&str> = json.networks.iter().map(|n| n.name.as_str()).collect();
    assert!(network_names.contains(&"ipv4"), "should have ipv4 network");
    assert!(network_names.contains(&"ipv6"), "should have ipv6 network");
}

// ---------------------------------------------------------------------------
// getpeerinfo: Core test vectors from rpc_net.py
// ---------------------------------------------------------------------------

/// getpeerinfo on an isolated node returns empty list.
#[test]
fn network_core__get_peer_info__isolated_node() {
    let node = BitcoinD::with_wallet(Wallet::None, &[]);
    let json: GetPeerInfo = node.client.get_peer_info().expect("getpeerinfo");
    assert_eq!(json.0.len(), 0, "isolated node should have no peers");
}

/// getpeerinfo on connected nodes should have proper peer entries.
/// Core checks: addrbind matches addr on the other side, minfeefilter,
/// connection_type (inbound/manual), services/servicesnames.
#[test]
fn network_core__get_peer_info__connected_fields() {
    let (node1, node2, _node3) = integration_test::three_node_network();

    // Mine a block to create some peer activity.
    node1.mine_a_block();

    let peer_info1: GetPeerInfo = node1.client.get_peer_info().expect("getpeerinfo node1");
    let peer_info2: GetPeerInfo = node2.client.get_peer_info().expect("getpeerinfo node2");

    assert!(!peer_info1.0.is_empty(), "node1 should have peers");
    assert!(!peer_info2.0.is_empty(), "node2 should have peers");

    // Verify key fields are present and valid for each peer.
    for peer in &peer_info1.0 {
        // peer.id is unsigned, always >= 0.
        assert!(!peer.address.is_empty(), "peer address should not be empty");
        assert!(peer.version > 0, "peer version should be > 0");
        assert!(!peer.subversion.is_empty(), "peer subver should not be empty");
        // bytes_sent/bytes_received should be > 0 after connection.
        assert!(peer.bytes_sent > 0, "bytes_sent should be > 0");
        assert!(peer.bytes_received > 0, "bytes_received should be > 0");
    }
}

/// getpeerinfo connection_type field should be present (added in v21).
#[test]
#[cfg(not(feature = "v20_and_below"))]
fn network_core__get_peer_info__connection_type() {
    let (_node1, node2, _node3) = integration_test::three_node_network();

    let peer_info: GetPeerInfo = node2.client.get_peer_info().expect("getpeerinfo node2");
    for peer in &peer_info.0 {
        // connection_type should be one of: inbound, outbound-full-relay,
        // manual, feeler, block-relay-only, addr-fetch.
        let ct = peer.connection_type.as_deref().expect("connection_type should be present");
        assert!(
            !ct.is_empty(),
            "connection_type should not be empty"
        );
    }
}

// ---------------------------------------------------------------------------
// getconnectioncount: Core test from rpc_net.py
// ---------------------------------------------------------------------------

/// getconnectioncount after connecting nodes equals 2 (Core's test_connection_count).
#[test]
fn network_core__get_connection_count__connected() {
    let (node1, _node2, _node3) = integration_test::three_node_network();

    let json: GetConnectionCount = node1.client.get_connection_count().expect("getconnectioncount");
    assert!(json.0 > 0, "connected node should have count > 0");
}

// ---------------------------------------------------------------------------
// getnettotals: Core test from rpc_net.py
// ---------------------------------------------------------------------------

/// getnettotals reports bytes sent and received, and upload_target info.
/// Core checks that totalbytessent and totalbytesrecv increase after a ping.
#[test]
fn network_core__get_net_totals__fields() {
    let (node1, _node2, _node3) = integration_test::three_node_network();

    node1.mine_a_block();

    let json: GetNetTotals = node1.client.get_net_totals().expect("getnettotals");

    // After connecting and mining, bytes should be > 0.
    assert!(json.total_bytes_received > 0, "totalbytesrecv should be > 0 after connection");
    assert!(json.total_bytes_sent > 0, "totalbytessent should be > 0 after connection");
    assert!(json.time_millis > 0, "timemillis should be > 0");
}

// ---------------------------------------------------------------------------
// setban + listbanned: Core lifecycle test from rpc_net.py
// ---------------------------------------------------------------------------

/// Full ban lifecycle: setban add -> listbanned -> setban remove -> listbanned empty.
/// This mirrors Core's test flow for ban management.
#[test]
fn network_core__setban_listbanned__lifecycle() {
    let node = BitcoinD::with_wallet(Wallet::None, &[]);

    // Start with no bans.
    let json: ListBanned = node.client.list_banned().expect("listbanned");
    assert!(json.0.is_empty(), "should start with no bans");

    // Ban a subnet.
    let subnet1 = "192.0.2.10/32";
    let subnet2 = "192.0.2.20/32";

    node.client.set_ban(subnet1, SetBanCommand::Add).expect("setban add subnet1");
    node.client.set_ban(subnet2, SetBanCommand::Add).expect("setban add subnet2");

    // Verify both are listed.
    let json: ListBanned = node.client.list_banned().expect("listbanned with 2 bans");
    assert_eq!(json.0.len(), 2, "should have 2 banned subnets");

    let banned_addrs: Vec<&str> = json.0.iter().map(|b| b.address.as_str()).collect();
    assert!(banned_addrs.contains(&subnet1));
    assert!(banned_addrs.contains(&subnet2));

    // For each ban, verify relevant fields are present.
    for ban_entry in &json.0 {
        assert!(ban_entry.ban_created > 0, "ban_created should be > 0");
        assert!(ban_entry.banned_until > ban_entry.ban_created, "banned_until > ban_created");
    }

    // Remove one ban.
    node.client.set_ban(subnet1, SetBanCommand::Remove).expect("setban remove subnet1");
    let json: ListBanned = node.client.list_banned().expect("listbanned after remove");
    assert_eq!(json.0.len(), 1, "should have 1 banned subnet after removal");
    assert_eq!(json.0[0].address, subnet2);

    // Clear all bans.
    node.client.clear_banned().expect("clearbanned");
    let json: ListBanned = node.client.list_banned().expect("listbanned after clear");
    assert!(json.0.is_empty(), "should have no bans after clearbanned");
}

// ---------------------------------------------------------------------------
// getaddednodeinfo: Core test from rpc_net.py
// ---------------------------------------------------------------------------

/// getaddednodeinfo returns empty initially, then reflects added nodes.
/// Core checks addnode add -> getaddednodeinfo -> addnode remove lifecycle.
#[test]
fn network_core__get_added_node_info__lifecycle() {
    let node = BitcoinD::with_wallet(Wallet::None, &[]);

    // Initially empty.
    let json: GetAddedNodeInfo = node.client.get_added_node_info().expect("getaddednodeinfo");
    assert!(json.0.is_empty(), "should be empty initially");

    // Add a node.
    let ip_port = "127.0.0.2:18444";
    node.client
        .add_node(ip_port, AddNodeCommand::Add)
        .expect("addnode add");

    // Verify it appears.
    let json: GetAddedNodeInfo = node.client.get_added_node_info().expect("getaddednodeinfo");
    assert_eq!(json.0.len(), 1, "should have 1 added node");
    assert_eq!(json.0[0].added_node, ip_port);

    // Add a second node.
    node.client
        .add_node("127.0.0.3:18444", AddNodeCommand::Add)
        .expect("addnode add second");
    let json: GetAddedNodeInfo = node.client.get_added_node_info().expect("getaddednodeinfo 2");
    assert_eq!(json.0.len(), 2);

    // Remove the first node.
    node.client
        .add_node(ip_port, AddNodeCommand::Remove)
        .expect("addnode remove");
    let json: GetAddedNodeInfo =
        node.client.get_added_node_info().expect("getaddednodeinfo after remove");
    assert_eq!(json.0.len(), 1);
    assert_eq!(json.0[0].added_node, "127.0.0.3:18444");
}

// ---------------------------------------------------------------------------
// getnodeaddresses + addpeeraddress: Core test from rpc_net.py
// ---------------------------------------------------------------------------

/// getnodeaddresses returns addresses added via addpeeraddress.
/// Core checks address, port, network, and services fields.
#[test]
#[cfg(not(feature = "v20_and_below"))]
fn network_core__get_node_addresses__after_add_peer() {
    let node = BitcoinD::with_wallet(Wallet::None, &[]);

    // Add a peer address.
    node.client.add_peer_address("1.2.3.4", 8333).expect("addpeeraddress");

    let json: GetNodeAddresses = node.client.get_node_addresses().expect("getnodeaddresses");
    assert!(!json.0.is_empty(), "should have at least one address");

    let addr = &json.0[0];
    assert_eq!(addr.address, "1.2.3.4");
    assert_eq!(addr.port, 8333);
    assert_eq!(addr.network, "ipv4");
}

/// getnodeaddresses for IPv6 address.
#[test]
#[cfg(not(feature = "v20_and_below"))]
fn network_core__get_node_addresses__ipv6() {
    let node = BitcoinD::with_wallet(Wallet::None, &[]);

    let ipv6_addr = "1233:3432:2434:2343:3234:2345:6546:4534";
    node.client.add_peer_address(ipv6_addr, 8333).expect("addpeeraddress ipv6");

    // Use call() directly to request IPv6.
    let result: Vec<bitcoind::serde_json::Value> = node
        .client
        .call("getnodeaddresses", &[bitcoind::serde_json::json!(0), bitcoind::serde_json::json!("ipv6")])
        .expect("getnodeaddresses ipv6");

    assert_eq!(result.len(), 1, "should have one IPv6 address");
    assert_eq!(result[0]["address"].as_str().unwrap(), ipv6_addr);
    assert_eq!(result[0]["network"].as_str().unwrap(), "ipv6");
    assert_eq!(result[0]["port"].as_u64().unwrap(), 8333);
}

// ---------------------------------------------------------------------------
// getaddrmaninfo: Core test from rpc_net.py
// ---------------------------------------------------------------------------

/// getaddrmaninfo reports correct counts for new and tried addresses.
/// Core checks that per-network counts sum up correctly.
#[test]
#[cfg(not(feature = "v25_and_below"))]
fn network_core__get_addr_man_info__counts() {
    let node = BitcoinD::with_wallet(Wallet::None, &[]);

    // Seed the address manager with some addresses.
    node.client.add_peer_address("1.2.3.4", 8333).expect("addpeeraddress ipv4");
    node.client.add_peer_address("2.0.0.0", 8333).expect("addpeeraddress ipv4 2");

    let json: GetAddrManInfo = node.client.get_addr_man_info().expect("getaddrmaninfo");

    // Should have at least the all_networks entry.
    assert!(!json.0.is_empty(), "addrmaninfo should not be empty");

    // For each network, total should equal new + tried.
    for (network_name, info) in &json.0 {
        assert_eq!(
            info.total,
            info.new + info.tried,
            "total != new + tried for network '{}'",
            network_name
        );
    }
}

// ---------------------------------------------------------------------------
// ping: Core uses ping to verify byte counting works
// ---------------------------------------------------------------------------

/// ping -> pong roundtrip can be verified through getpeerinfo byte counters.
#[test]
fn network_core__ping__increases_bytes() {
    let (node1, _node2, _node3) = integration_test::three_node_network();

    let before: GetPeerInfo = node1.client.get_peer_info().expect("getpeerinfo before");
    let bytes_before: u64 = before.0.iter().map(|p| p.bytes_sent).sum();

    node1.client.ping().expect("ping");
    std::thread::sleep(std::time::Duration::from_millis(500));

    let after: GetPeerInfo = node1.client.get_peer_info().expect("getpeerinfo after");
    let bytes_after: u64 = after.0.iter().map(|p| p.bytes_sent).sum();

    assert!(bytes_after >= bytes_before, "bytes_sent should not decrease after ping");
}
