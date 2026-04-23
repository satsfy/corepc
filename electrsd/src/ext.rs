//! Extra functions for the electrs rpc
//!

use std::thread;
use std::time::Duration;

use electrum_client::bitcoin::Txid;
use electrum_client::ElectrumApi;

use crate::ElectrsD;

impl ElectrsD {
    #[cfg(not(feature = "electrs_0_8_10"))]
    /// wait up to a minute the electrum server has indexed up to the given height.
    pub fn wait_height(&self, height: usize) {
        for _ in 0..600 {
            match self.client.block_header_raw(height) {
                Ok(_) => break,
                Err(_) => thread::sleep(Duration::from_millis(100)),
            }
        }
    }

    /// wait up to a minute the electrum server has indexed the given transaction
    pub fn wait_tx(&self, txid: &Txid) {
        'main_loop: for _ in 0..600 {
            match self.client.transaction_get(txid) {
                Ok(tx) => {
                    // having the raw tx doesn't mean the scripts has been indexed
                    let txid = tx.compute_txid();
                    if let Some(output) = tx.output.first() {
                        let history =
                            self.client.script_get_history(&output.script_pubkey).unwrap();
                        for el in history {
                            if el.tx_hash == txid {
                                // the tx has to be updated atomically, so founding one is enough
                                return;
                            }
                        }
                        // the tx output has not been yet found
                        continue 'main_loop;
                    }
                    // the tx has 0 ouptut, no need to ensure script_pubkey are indexed
                    return;
                }
                Err(_) => thread::sleep(Duration::from_millis(100)),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use electrum_client::bitcoin::Amount;
    use electrum_client::ElectrumApi;

    use crate::test::setup_nodes;

    #[cfg(not(feature = "electrs_0_8_10"))]
    #[test]
    fn test_wait_height() {
        let (_, bitcoind, electrsd) = setup_nodes();
        let header = electrsd.client.block_headers_subscribe().unwrap();

        let address = bitcoind.client.new_address().unwrap();
        let blocks_to_generate = 101_usize.saturating_sub(header.height);
        if blocks_to_generate > 0 {
            bitcoind.client.generate_to_address(blocks_to_generate, &address).unwrap();
        }
        electrsd.wait_height(101);
        let header = electrsd.client.block_headers_subscribe().unwrap();
        assert!(header.height >= 101, "header height is {}", header.height);
    }

    #[test]
    fn test_wait_tx() {
        let (_, bitcoind, electrsd) = setup_nodes();
        let header = electrsd.client.block_headers_subscribe().unwrap();

        let generate_address = bitcoind.client.new_address().unwrap();
        let blocks_to_generate = 101_usize.saturating_sub(header.height);
        if blocks_to_generate > 0 {
            bitcoind.client.generate_to_address(blocks_to_generate, &generate_address).unwrap();
        }
        #[cfg(not(feature = "electrs_0_8_10"))]
        electrsd.wait_height(101);

        let address = bitcoind.client.new_address().unwrap();
        let txid = bitcoind
            .client
            .send_to_address(&address, Amount::from_sat(10000))
            .unwrap()
            .txid()
            .unwrap();

        electrsd.wait_tx(&txid);
        let history = electrsd.client.script_get_history(&address.script_pubkey()).unwrap();
        assert_eq!(history.len(), 1);
    }
}
