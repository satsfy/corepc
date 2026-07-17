// SPDX-License-Identifier: CC0-1.0

//! Isolation bridges for the async blocking facade.
//!
//! These are the methods the facade implements *itself* on top of the async production client, so
//! nothing in the sync client's `impl_client_*` macros can reach the async path. Every method
//! routes through the GENERATED async wrapper on `self.inner` (its own argument encoding) and
//! returns the GENERATED response type, so a test's `into_model()` runs the generated conversion.
//! The only exceptions are RPCs absent from Core's OpenRPC spec (hidden/zmq/scan status shapes):
//! those go through the facade's own raw `self.call` and return the curated raw type (their tests
//! read fields directly and never call `into_model`).
//!
//! Unlike codegen string templates, this is real Rust: the compiler, `rustfmt` and `rust-analyzer`
//! all check it. The version is passed as a token (`impl_async_bridges!(v31)`) so the
//! version-specific paths resolve; `vgen`/`vcli` alias the generated types and call surface.
//!
//! Adding a bridge: add the method here AND (if its sync macro is active in `client_sync/v{N}`) add
//! its macro suffix to `BRIDGED_METHODS` in `codegen/src/codegen/blocking.rs` so the reused sync
//! macro is skipped (no duplicate definition).

/// Emit the blocking facade's bridged methods for version `$v`, e.g. `impl_async_bridges!(v31)`.
///
/// Expanded inside `client_async/v{N}/blocking.rs`, where `Client`, `Result`, `Error`, `into_json`,
/// `AddressType` and the curated response types are all in scope.
#[macro_export]
macro_rules! impl_async_bridges {
    ($v:ident) => {
        use $crate::client_async::$v as vcli;
        use $crate::types::$v::generated as vgen;

        // == Wallet ==
        impl Client {
            fn get_new_address_generated(
                &self,
                label: Option<&str>,
                ty: Option<AddressType>,
            ) -> Result<vgen::GetNewAddress> {
                let address_type = match ty {
                    Some(ty) => Some(serde_json::from_value::<String>(into_json(ty)?)?),
                    None => None,
                };
                let opts = vcli::wallet::GetNewAddressOptions {
                    label: label.map(str::to_owned),
                    address_type,
                };
                self.rt.block_on(self.inner.get_new_address_with(opts)).map_err(Self::map_err)
            }

            /// Low-level `getnewaddress` (matches the sync client's signature).
            pub fn get_new_address(
                &self,
                label: Option<&str>,
                ty: Option<AddressType>,
            ) -> Result<vgen::GetNewAddress> {
                self.get_new_address_generated(label, ty)
            }

            /// Gets a new address and parses it assuming it is correct.
            pub fn new_address(&self) -> Result<bitcoin::Address> {
                let model = self.get_new_address_generated(None, None)?.into_model().unwrap();
                Ok(model.0.assume_checked())
            }

            /// Gets a new address of the given type and parses it assuming it is correct.
            pub fn new_address_with_type(&self, ty: AddressType) -> Result<bitcoin::Address> {
                let model = self.get_new_address_generated(None, Some(ty))?.into_model().unwrap();
                Ok(model.0.assume_checked())
            }

            /// Gets a new address with a label and parses it assuming it is correct (unchecked).
            pub fn new_address_with_label(
                &self,
                label: &str,
            ) -> Result<bitcoin::Address<bitcoin::address::NetworkUnchecked>> {
                let model = self.get_new_address_generated(Some(label), None)?.into_model().unwrap();
                Ok(model.0)
            }

            pub fn create_wallet(&self, wallet: &str) -> Result<vgen::CreateWallet> {
                self.rt.block_on(self.inner.create_wallet(wallet.to_owned())).map_err(Self::map_err)
            }

            pub fn create_wallet_external_signer(&self, wallet: &str) -> Result<vgen::CreateWallet> {
                let opts = vcli::wallet::CreateWalletOptions {
                    disable_private_keys: Some(true),
                    external_signer: Some(true),
                    ..Default::default()
                };
                self.rt
                    .block_on(self.inner.create_wallet_with(wallet.to_owned(), opts))
                    .map_err(Self::map_err)
            }

            pub fn load_wallet(&self, wallet: &str) -> Result<vgen::LoadWallet> {
                self.rt.block_on(self.inner.load_wallet(wallet.to_owned())).map_err(Self::map_err)
            }

            pub fn get_balance(&self) -> Result<vgen::GetBalance> {
                self.rt.block_on(self.inner.get_balance()).map_err(Self::map_err)
            }

            pub fn get_balances(&self) -> Result<vgen::GetBalances> {
                self.rt.block_on(self.inner.get_balances()).map_err(Self::map_err)
            }

            pub fn get_transaction(&self, txid: Txid) -> Result<vgen::GetTransaction> {
                self.rt.block_on(self.inner.get_transaction(txid.to_string())).map_err(Self::map_err)
            }

            pub fn send_to_address(
                &self,
                address: &Address<NetworkChecked>,
                amount: Amount,
            ) -> Result<vgen::SendToAddressVerbose0> {
                let amount: vcli::wallet::SendToAddressAmount =
                    serde_json::from_value(into_json(amount.to_btc())?)?;
                self.rt
                    .block_on(self.inner.send_to_address_verbose_0(address.to_string(), amount))
                    .map_err(Self::map_err)
            }

            pub fn sign_raw_transaction_with_wallet(
                &self,
                tx: &bitcoin::Transaction,
            ) -> Result<vgen::SignRawTransactionWithWallet> {
                let hex = bitcoin::consensus::encode::serialize_hex(tx);
                self.rt
                    .block_on(self.inner.sign_raw_transaction_with_wallet(hex))
                    .map_err(Self::map_err)
            }
        }

        // == Generating ==
        impl Client {
            pub fn generate_block(
                &self,
                output: &str,
                transactions: &[String],
                submit: bool,
            ) -> Result<vgen::GenerateBlock> {
                let opts = vcli::hidden::GenerateBlockOptions { submit: Some(submit) };
                self.rt
                    .block_on(self.inner.generate_block_with(
                        output.to_owned(),
                        transactions.to_vec(),
                        opts,
                    ))
                    .map_err(Self::map_err)
            }

            pub fn generate_to_address(
                &self,
                nblocks: usize,
                address: &bitcoin::Address,
            ) -> Result<vgen::GenerateToAddress> {
                self.rt
                    .block_on(self.inner.generate_to_address(nblocks as i64, address.to_string()))
                    .map_err(Self::map_err)
            }

            pub fn generate_to_descriptor(
                &self,
                nblocks: usize,
                descriptor: &str,
            ) -> Result<vgen::GenerateToDescriptor> {
                self.rt
                    .block_on(
                        self.inner.generate_to_descriptor(nblocks as i64, descriptor.to_owned()),
                    )
                    .map_err(Self::map_err)
            }

            pub fn invalidate_block(&self, hash: BlockHash) -> Result<()> {
                self.rt.block_on(self.inner.invalidate_block(hash.to_string())).map_err(Self::map_err)
            }
        }

        // == Mining ==
        impl Client {
            // Returns the GENERATED `Object` variant (`GetBlockTemplateVariant2`); the facade's
            // `vtype` aliases it to `GetBlockTemplate` so the test's `into_model()` runs the
            // generated conversion. `Null`/`Text` (proposal-mode replies) are an error here.
            pub fn get_block_template(
                &self,
                request: &TemplateRequest,
            ) -> Result<vgen::GetBlockTemplateVariant2> {
                let req: vcli::mining::GetBlockTemplateTemplateRequest =
                    serde_json::from_value(into_json(request)?)?;
                match self.rt.block_on(self.inner.get_block_template(req)).map_err(Self::map_err)? {
                    vgen::GetBlockTemplate::Object(v) => Ok(v),
                    vgen::GetBlockTemplate::Null(()) =>
                        Err(Error::Returned("getblocktemplate returned null".to_owned())),
                    vgen::GetBlockTemplate::Text(s) =>
                        Err(Error::Returned(format!("getblocktemplate returned: {s}"))),
                }
            }

            pub fn get_mining_info(&self) -> Result<vgen::GetMiningInfo> {
                self.rt.block_on(self.inner.get_mining_info()).map_err(Self::map_err)
            }

            pub fn get_network_hash_ps(&self) -> Result<f64> {
                Ok(*self.rt.block_on(self.inner.get_network_hash_ps()).map_err(Self::map_err)?)
            }

            pub fn get_prioritised_transactions(&self) -> Result<vgen::GetPrioritisedTransactions> {
                self.rt.block_on(self.inner.get_prioritised_transactions()).map_err(Self::map_err)
            }

            pub fn prioritise_transaction(
                &self,
                txid: &Txid,
                fee_delta: bitcoin::SignedAmount,
            ) -> Result<bool> {
                let res = self
                    .rt
                    .block_on(self.inner.prioritise_transaction(txid.to_string(), fee_delta.to_sat()))
                    .map_err(Self::map_err)?;
                Ok(*res)
            }

            pub fn submit_block(&self, block: &Block) -> Result<()> {
                let hex = bitcoin::consensus::encode::serialize_hex(block);
                match self.rt.block_on(self.inner.submit_block(hex)).map_err(Self::map_err)? {
                    vgen::SubmitBlock::Null(()) => Ok(()),
                    vgen::SubmitBlock::Text(reason) => Err(Error::Returned(reason)),
                }
            }

            pub fn submit_header(&self, header: &bitcoin::block::Header) -> Result<()> {
                let hexdata = bitcoin::consensus::encode::serialize_hex(header);
                self.rt.block_on(self.inner.submit_header(hexdata)).map_err(Self::map_err)
            }
        }

        // == Network ==
        impl Client {
            pub fn add_node(&self, node: &str, command: AddNodeCommand) -> Result<()> {
                let command = serde_json::from_value::<String>(into_json(command)?)?;
                self.rt.block_on(self.inner.add_node(node.to_owned(), command)).map_err(Self::map_err)
            }

            pub fn clear_banned(&self) -> Result<()> {
                self.rt.block_on(self.inner.clear_banned()).map_err(Self::map_err)
            }

            pub fn disconnect_node(&self, address: &str) -> Result<()> {
                let opts = vcli::network::DisconnectNodeOptions {
                    address: Some(address.to_owned()),
                    nodeid: None,
                };
                self.rt.block_on(self.inner.disconnect_node_with(opts)).map_err(Self::map_err)
            }

            pub fn get_added_node_info(&self) -> Result<vgen::GetAddedNodeInfo> {
                self.rt.block_on(self.inner.get_added_node_info()).map_err(Self::map_err)
            }

            pub fn get_addr_man_info(&self) -> Result<vgen::GetAddrManInfo> {
                self.rt.block_on(self.inner.get_addr_man_info()).map_err(Self::map_err)
            }

            pub fn get_connection_count(&self) -> Result<vgen::GetConnectionCount> {
                self.rt.block_on(self.inner.get_connection_count()).map_err(Self::map_err)
            }

            pub fn get_net_totals(&self) -> Result<vgen::GetNetTotals> {
                self.rt.block_on(self.inner.get_net_totals()).map_err(Self::map_err)
            }

            /// Server version field of `getnetworkinfo`; `check_expected_server_version` calls this
            /// (the sync `get_network_info` macro that normally defines it is not reused here).
            pub fn server_version(&self) -> Result<usize> {
                let res = self.rt.block_on(self.inner.get_network_info()).map_err(Self::map_err)?;
                usize::try_from(res.version).map_err(|e| Error::Returned(e.to_string()))
            }

            pub fn get_network_info(&self) -> Result<vgen::GetNetworkInfo> {
                self.rt.block_on(self.inner.get_network_info()).map_err(Self::map_err)
            }

            pub fn get_node_addresses(&self) -> Result<vgen::GetNodeAddresses> {
                self.rt.block_on(self.inner.get_node_addresses()).map_err(Self::map_err)
            }

            pub fn add_peer_address(&self, address: &str, port: u16) -> Result<vgen::AddPeerAddress> {
                self.rt
                    .block_on(self.inner.add_peer_address(address.to_owned(), port as i64))
                    .map_err(Self::map_err)
            }

            // == Hidden ==
            //
            // Hidden RPCs are absent from Core's OpenRPC dump; the codegen supplement adds
            // them, so these route through the generated wrappers like everything else.

            /// # Panics
            ///
            /// * Panics if `conf_target` is outside the range [1, 1008].
            pub fn estimate_raw_fee(&self, conf_target: u32) -> Result<vgen::EstimateRawFee> {
                assert!(
                    (1..=1008).contains(&conf_target),
                    "invalid conf_target, must be between 1 and 1008 inclusive"
                );
                self.rt
                    .block_on(self.inner.estimate_raw_fee(conf_target as i64))
                    .map_err(Self::map_err)
            }

            pub fn get_orphan_txs(&self) -> Result<vgen::GetOrphanTxsVerbose0> {
                self.rt.block_on(self.inner.get_orphan_txs_verbose_0()).map_err(Self::map_err)
            }

            pub fn get_orphan_txs_verbosity_1(&self) -> Result<vgen::GetOrphanTxsVerbose1> {
                self.rt.block_on(self.inner.get_orphan_txs_verbose_1()).map_err(Self::map_err)
            }

            pub fn get_orphan_txs_verbosity_2(&self) -> Result<vgen::GetOrphanTxsVerbose2> {
                self.rt.block_on(self.inner.get_orphan_txs_verbose_2()).map_err(Self::map_err)
            }

            pub fn add_connection(
                &self,
                address: &str,
                connection_type: &str,
                v2transport: bool,
            ) -> Result<vgen::AddConnection> {
                self.rt
                    .block_on(self.inner.add_connection(
                        address.to_owned(),
                        connection_type.to_owned(),
                        v2transport,
                    ))
                    .map_err(Self::map_err)
            }

            pub fn sync_with_validation_interface_queue(&self) -> Result<()> {
                self.rt
                    .block_on(self.inner.sync_with_validation_interface_queue())
                    .map_err(Self::map_err)
            }

            pub fn reconsider_block(&self, block_hash: bitcoin::BlockHash) -> Result<()> {
                self.rt
                    .block_on(self.inner.reconsider_block(block_hash.to_string()))
                    .map_err(Self::map_err)
            }

            pub fn mock_scheduler(&self, delta_time: u64) -> Result<()> {
                self.rt
                    .block_on(self.inner.mock_scheduler(delta_time as i64))
                    .map_err(Self::map_err)
            }

            pub fn get_raw_addrman(&self) -> Result<vgen::GetRawAddrMan> {
                self.rt.block_on(self.inner.get_raw_addr_man()).map_err(Self::map_err)
            }

            pub fn get_peer_info(&self) -> Result<vgen::GetPeerInfo> {
                self.rt.block_on(self.inner.get_peer_info()).map_err(Self::map_err)
            }

            pub fn list_banned(&self) -> Result<vgen::ListBanned> {
                self.rt.block_on(self.inner.list_banned()).map_err(Self::map_err)
            }

            pub fn ping(&self) -> Result<()> {
                self.rt.block_on(self.inner.ping()).map_err(Self::map_err)
            }

            pub fn set_ban(&self, subnet: &str, command: SetBanCommand) -> Result<()> {
                let command = serde_json::from_value::<String>(into_json(command)?)?;
                self.rt
                    .block_on(self.inner.set_ban(subnet.to_owned(), command))
                    .map_err(Self::map_err)
            }

            pub fn set_network_active(&self, state: bool) -> Result<vgen::SetNetworkActive> {
                self.rt.block_on(self.inner.set_network_active(state)).map_err(Self::map_err)
            }
        }

        // == Blockchain ==
        impl Client {
            pub fn get_blockchain_info(&self) -> Result<vgen::GetBlockchainInfo> {
                self.rt.block_on(self.inner.get_blockchain_info()).map_err(Self::map_err)
            }

            /// Convenience: `getbestblockhash` parsed via the generated `into_model`.
            pub fn best_block_hash(&self) -> Result<bitcoin::BlockHash> {
                let json = self.get_best_block_hash()?;
                Ok(json.into_model().map_err(|e| Error::Returned(e.to_string()))?.0)
            }

            pub fn get_best_block_hash(&self) -> Result<vgen::GetBestBlockHash> {
                self.rt.block_on(self.inner.get_best_block_hash()).map_err(Self::map_err)
            }

            /// Convenience: `getblock` verbosity 0 decoded via the generated `into_model`.
            pub fn get_block(&self, hash: BlockHash) -> Result<Block> {
                let json = self.get_block_verbose_zero(hash)?;
                Ok(json.into_model().map_err(|e| Error::Returned(e.to_string()))?.0)
            }

            pub fn get_block_verbose_zero(&self, hash: BlockHash) -> Result<vgen::GetBlockVerbose0> {
                self.rt.block_on(self.inner.get_block_verbose_0(hash.to_string())).map_err(Self::map_err)
            }

            pub fn get_block_verbose_one(&self, hash: BlockHash) -> Result<vgen::GetBlockVerbose1> {
                self.rt.block_on(self.inner.get_block_verbose_1(hash.to_string())).map_err(Self::map_err)
            }

            pub fn get_block_verbose_two(&self, hash: BlockHash) -> Result<vgen::GetBlockVerbose2> {
                self.rt.block_on(self.inner.get_block_verbose_2(hash.to_string())).map_err(Self::map_err)
            }

            pub fn get_block_verbose_three(&self, hash: BlockHash) -> Result<vgen::GetBlockVerbose3> {
                self.rt.block_on(self.inner.get_block_verbose_3(hash.to_string())).map_err(Self::map_err)
            }

            pub fn get_block_count(&self) -> Result<vgen::GetBlockCount> {
                self.rt.block_on(self.inner.get_block_count()).map_err(Self::map_err)
            }

            pub fn get_block_filter(&self, hash: BlockHash) -> Result<vgen::GetBlockFilter> {
                self.rt.block_on(self.inner.get_block_filter(hash.to_string())).map_err(Self::map_err)
            }

            pub fn get_block_from_peer(&self, hash: BlockHash, peer_id: i64) -> Result<()> {
                self.rt
                    .block_on(self.inner.get_block_from_peer(hash.to_string(), peer_id))
                    .map_err(Self::map_err)?;
                Ok(())
            }

            pub fn get_block_hash(&self, height: u64) -> Result<vgen::GetBlockHash> {
                self.rt.block_on(self.inner.get_block_hash(height as i64)).map_err(Self::map_err)
            }

            pub fn get_block_header(&self, hash: &BlockHash) -> Result<vgen::GetBlockHeaderVerbose0> {
                self.rt
                    .block_on(self.inner.get_block_header_verbose_0(hash.to_string()))
                    .map_err(Self::map_err)
            }

            pub fn get_block_header_verbose(
                &self,
                hash: &BlockHash,
            ) -> Result<vgen::GetBlockHeaderVerbose1> {
                self.rt
                    .block_on(self.inner.get_block_header_verbose_1(hash.to_string()))
                    .map_err(Self::map_err)
            }

            pub fn get_block_stats_by_height(
                &self,
                height: u32,
                stats: Option<&[&str]>,
            ) -> Result<vgen::GetBlockStats> {
                let hh = vcli::blockchain::GetBlockStatsHashOrHeight::Number(height as i64);
                self.get_block_stats_inner(hh, stats)
            }

            pub fn get_block_stats_by_block_hash(
                &self,
                hash: &BlockHash,
                stats: Option<&[&str]>,
            ) -> Result<vgen::GetBlockStats> {
                let hh = vcli::blockchain::GetBlockStatsHashOrHeight::Text(hash.to_string());
                self.get_block_stats_inner(hh, stats)
            }

            fn get_block_stats_inner(
                &self,
                hash_or_height: vcli::blockchain::GetBlockStatsHashOrHeight,
                stats: Option<&[&str]>,
            ) -> Result<vgen::GetBlockStats> {
                match stats {
                    None => self
                        .rt
                        .block_on(self.inner.get_block_stats(hash_or_height))
                        .map_err(Self::map_err),
                    Some(stats) => {
                        let opts = vcli::blockchain::GetBlockStatsOptions {
                            stats: Some(stats.iter().map(|s| s.to_string()).collect()),
                        };
                        self.rt
                            .block_on(self.inner.get_block_stats_with(hash_or_height, opts))
                            .map_err(Self::map_err)
                    }
                }
            }

            pub fn get_chain_states(&self) -> Result<vgen::GetChainStates> {
                self.rt.block_on(self.inner.get_chain_states()).map_err(Self::map_err)
            }

            pub fn get_chain_tips(&self) -> Result<vgen::GetChainTips> {
                self.rt.block_on(self.inner.get_chain_tips()).map_err(Self::map_err)
            }

            pub fn get_chain_tx_stats(&self) -> Result<vgen::GetChainTxStats> {
                self.rt.block_on(self.inner.get_chain_tx_stats()).map_err(Self::map_err)
            }

            pub fn get_deployment_info(&self, hash: &BlockHash) -> Result<vgen::GetDeploymentInfo> {
                let opts = vcli::blockchain::GetDeploymentInfoOptions {
                    block_hash: Some(hash.to_string()),
                };
                self.rt.block_on(self.inner.get_deployment_info_with(opts)).map_err(Self::map_err)
            }

            pub fn get_deployment_info_tip(&self) -> Result<vgen::GetDeploymentInfo> {
                self.rt.block_on(self.inner.get_deployment_info()).map_err(Self::map_err)
            }

            pub fn get_descriptor_activity(
                &self,
                block_hashes: &[BlockHash],
                scan_objects: &[&str],
            ) -> Result<vgen::GetDescriptorActivity> {
                let hashes = block_hashes.iter().map(|h| h.to_string()).collect();
                let objects = scan_objects
                    .iter()
                    .map(|s| vcli::blockchain::GetDescriptorActivityScanObjects::Text(s.to_string()))
                    .collect();
                self.rt
                    .block_on(self.inner.get_descriptor_activity(hashes, objects))
                    .map_err(Self::map_err)
            }

            pub fn get_difficulty(&self) -> Result<vgen::GetDifficulty> {
                self.rt.block_on(self.inner.get_difficulty()).map_err(Self::map_err)
            }

            pub fn get_mempool_ancestors(&self, txid: Txid) -> Result<vgen::GetMempoolAncestorsVerbose0> {
                self.rt
                    .block_on(self.inner.get_mempool_ancestors_verbose_0(txid.to_string()))
                    .map_err(Self::map_err)
            }

            pub fn get_mempool_ancestors_verbose(
                &self,
                txid: Txid,
            ) -> Result<vgen::GetMempoolAncestorsVerbose1> {
                self.rt
                    .block_on(self.inner.get_mempool_ancestors_verbose_1(txid.to_string()))
                    .map_err(Self::map_err)
            }

            pub fn get_mempool_cluster(&self, txid: Txid) -> Result<vgen::GetMempoolCluster> {
                self.rt
                    .block_on(self.inner.get_mempool_cluster(txid.to_string()))
                    .map_err(Self::map_err)
            }

            pub fn get_mempool_descendants(
                &self,
                txid: Txid,
            ) -> Result<vgen::GetMempoolDescendantsVerbose0> {
                self.rt
                    .block_on(self.inner.get_mempool_descendants_verbose_0(txid.to_string()))
                    .map_err(Self::map_err)
            }

            pub fn get_mempool_descendants_verbose(
                &self,
                txid: Txid,
            ) -> Result<vgen::GetMempoolDescendantsVerbose1> {
                self.rt
                    .block_on(self.inner.get_mempool_descendants_verbose_1(txid.to_string()))
                    .map_err(Self::map_err)
            }

            pub fn get_mempool_entry(&self, txid: Txid) -> Result<vgen::GetMempoolEntry> {
                self.rt.block_on(self.inner.get_mempool_entry(txid.to_string())).map_err(Self::map_err)
            }

            pub fn get_mempool_feerate_diagram(&self) -> Result<vgen::GetMempoolFeeRateDiagram> {
                self.rt.block_on(self.inner.get_mempool_fee_rate_diagram()).map_err(Self::map_err)
            }

            pub fn get_mempool_info(&self) -> Result<vgen::GetMempoolInfo> {
                self.rt.block_on(self.inner.get_mempool_info()).map_err(Self::map_err)
            }

            pub fn get_raw_mempool(&self) -> Result<vgen::GetRawMempoolVerbose0> {
                self.rt.block_on(self.inner.get_raw_mempool_verbose_0()).map_err(Self::map_err)
            }

            pub fn get_raw_mempool_verbose(&self) -> Result<vgen::GetRawMempoolVerbose1> {
                self.rt.block_on(self.inner.get_raw_mempool_verbose_1()).map_err(Self::map_err)
            }

            pub fn get_raw_mempool_sequence(&self) -> Result<vgen::GetRawMempoolVerbose2> {
                self.rt.block_on(self.inner.get_raw_mempool_verbose_2()).map_err(Self::map_err)
            }

            // Returns the GENERATED `Object` variant; `vtype` aliases it to `GetTxOut`. A `null`
            // response (unknown/spent output) is an error here, matching what the tests exercise.
            pub fn get_tx_out(&self, txid: Txid, vout: u64) -> Result<vgen::GetTxOutVariant1> {
                match self
                    .rt
                    .block_on(self.inner.get_tx_out(txid.to_string(), vout as i64))
                    .map_err(Self::map_err)?
                {
                    vgen::GetTxOut::Object(v) => Ok(v),
                    vgen::GetTxOut::Null(()) =>
                        Err(Error::Returned("gettxout returned null".to_owned())),
                }
            }

            pub fn get_tx_out_proof(&self, txids: &[Txid]) -> Result<String> {
                let txids = txids.iter().map(|t| t.to_string()).collect();
                let proof =
                    self.rt.block_on(self.inner.get_tx_out_proof(txids)).map_err(Self::map_err)?;
                Ok(proof.0)
            }

            pub fn get_tx_out_set_info(&self) -> Result<vgen::GetTxOutSetInfo> {
                self.rt.block_on(self.inner.get_tx_out_set_info()).map_err(Self::map_err)
            }

            pub fn get_tx_spending_prevout(
                &self,
                outputs: &[bitcoin::OutPoint],
                mempool_only: bool,
                return_spending_tx: bool,
            ) -> Result<vgen::GetTxSpendingPrevout> {
                let outputs = outputs
                    .iter()
                    .map(|out| vcli::blockchain::GetTxSpendingPrevoutOutputs {
                        txid: out.txid.to_string(),
                        vout: out.vout as i64,
                    })
                    .collect();
                let opts = vcli::blockchain::GetTxSpendingPrevoutOptions {
                    mempool_only: Some(mempool_only),
                    return_spending_tx: Some(return_spending_tx),
                };
                self.rt
                    .block_on(self.inner.get_tx_spending_prevout_with(outputs, opts))
                    .map_err(Self::map_err)
            }

            pub fn import_mempool(&self, filepath: &str) -> Result<()> {
                self.rt
                    .block_on(self.inner.import_mempool(filepath.to_owned()))
                    .map_err(Self::map_err)?;
                Ok(())
            }

            pub fn load_tx_out_set(&self, path: &str) -> Result<vgen::LoadTxOutSet> {
                self.rt.block_on(self.inner.load_tx_out_set(path.to_owned())).map_err(Self::map_err)
            }

            pub fn dump_tx_out_set(&self, path: &str, snapshot_type: &str) -> Result<vgen::DumpTxOutSet> {
                let opts = vcli::blockchain::DumpTxOutSetOptions {
                    type_: Some(snapshot_type.to_owned()),
                    options: None,
                };
                self.rt
                    .block_on(self.inner.dump_tx_out_set_with(path.to_owned(), opts))
                    .map_err(Self::map_err)
            }

            pub fn precious_block(&self, hash: BlockHash) -> Result<()> {
                self.rt.block_on(self.inner.precious_block(hash.to_string())).map_err(Self::map_err)
            }

            pub fn prune_blockchain(&self, height: u64) -> Result<vgen::PruneBlockchain> {
                self.rt.block_on(self.inner.prune_blockchain(height as i64)).map_err(Self::map_err)
            }

            pub fn save_mempool(&self) -> Result<vgen::SaveMempool> {
                self.rt.block_on(self.inner.save_mempool()).map_err(Self::map_err)
            }

            // `scanblocks` start returns the GENERATED object variant (`vtype` aliases it to
            // `ScanBlocksStart`); status/abort are read raw by the tests, so they stay curated.
            pub fn scan_blocks_start(&self, scan_objects: &[&str]) -> Result<vgen::ScanBlocksVariant1> {
                let opts = vcli::blockchain::ScanBlocksOptions {
                    scan_objects: Some(
                        scan_objects
                            .iter()
                            .map(|s| vcli::blockchain::ScanBlocksScanObjects::Text(s.to_string()))
                            .collect(),
                    ),
                    start_height: None,
                    stop_height: None,
                    filter_type: None,
                    options: None,
                };
                match self
                    .rt
                    .block_on(self.inner.scan_blocks_with("start".to_owned(), opts))
                    .map_err(Self::map_err)?
                {
                    vgen::ScanBlocks::Object(v) => Ok(v),
                    other => Err(Error::Returned(format!("scanblocks start: unexpected {other:?}"))),
                }
            }

            pub fn scan_blocks_status(&self) -> Result<Option<ScanBlocksStatus>> {
                self.call("scanblocks", &["status".into()])
            }

            pub fn scan_blocks_abort(&self) -> Result<ScanBlocksAbort> {
                self.call("scanblocks", &["abort".into()])
            }

            pub fn scan_tx_out_set_start(
                &self,
                scan_objects: &[&str],
            ) -> Result<vgen::ScanTxOutSetVariant0> {
                let opts = vcli::blockchain::ScanTxOutSetOptions {
                    scan_objects: Some(
                        scan_objects
                            .iter()
                            .map(|s| vcli::blockchain::ScanTxOutSetScanObjects::Text(s.to_string()))
                            .collect(),
                    ),
                };
                match self
                    .rt
                    .block_on(self.inner.scan_tx_out_set_with("start".to_owned(), opts))
                    .map_err(Self::map_err)?
                {
                    vgen::ScanTxOutSet::Object(v) => Ok(v),
                    other =>
                        Err(Error::Returned(format!("scantxoutset start: unexpected {other:?}"))),
                }
            }

            pub fn scan_tx_out_set_status(&self) -> Result<Option<ScanTxOutSetStatus>> {
                self.call("scantxoutset", &["status".into()])
            }

            pub fn scan_tx_out_set_abort(&self) -> Result<ScanTxOutSetAbort> {
                self.call("scantxoutset", &["abort".into()])
            }

            pub fn verify_chain(&self) -> Result<vgen::VerifyChain> {
                self.rt.block_on(self.inner.verify_chain()).map_err(Self::map_err)
            }

            pub fn verify_tx_out_proof(&self, proof: &str) -> Result<vgen::VerifyTxOutProof> {
                self.rt
                    .block_on(self.inner.verify_tx_out_proof(proof.to_owned()))
                    .map_err(Self::map_err)
            }

            pub fn wait_for_block(&self, hash: &BlockHash) -> Result<vgen::WaitForBlock> {
                self.rt.block_on(self.inner.wait_for_block(hash.to_string())).map_err(Self::map_err)
            }

            pub fn wait_for_block_height(&self, height: u64) -> Result<vgen::WaitForBlockHeight> {
                self.rt
                    .block_on(self.inner.wait_for_block_height(height as i64))
                    .map_err(Self::map_err)
            }

            pub fn wait_for_new_block(&self) -> Result<vgen::WaitForNewBlock> {
                self.rt.block_on(self.inner.wait_for_new_block()).map_err(Self::map_err)
            }
        }

        // == Control ==
        // Raw responses the tests read directly (no `into_model`); `logging`/`getmemoryinfo` keep
        // the curated struct shape (the generated ones are plain maps).
        impl Client {
            pub fn get_memory_info(&self) -> Result<GetMemoryInfoStats> {
                self.call("getmemoryinfo", &[])
            }

            pub fn get_rpc_info(&self) -> Result<vgen::GetRpcInfo> {
                self.rt.block_on(self.inner.get_rpc_info()).map_err(Self::map_err)
            }

            pub fn help(&self) -> Result<String> { self.call("help", &[]) }

            pub fn logging(&self) -> Result<Logging> { self.call("logging", &[]) }

            pub fn stop(&self) -> Result<String> { self.call("stop", &[]) }

            pub fn uptime(&self) -> Result<u32> { self.call("uptime", &[]) }
        }

        // == Rawtransactions ==
        impl Client {
            pub fn create_raw_transaction(
                &self,
                inputs: &[Input],
                outputs: &[Output],
            ) -> Result<vgen::CreateRawTransaction> {
                let inputs: Vec<vcli::raw_transactions::CreateRawTransactionInputs> =
                    serde_json::from_value(into_json(inputs)?)?;
                let outputs: Vec<vcli::raw_transactions::CreateRawTransactionOutputs> =
                    serde_json::from_value(into_json(outputs)?)?;
                self.rt
                    .block_on(self.inner.create_raw_transaction(inputs, outputs))
                    .map_err(Self::map_err)
            }

            pub fn fund_raw_transaction(
                &self,
                tx: &bitcoin::Transaction,
            ) -> Result<vgen::FundRawTransaction> {
                let hex = bitcoin::consensus::encode::serialize_hex(tx);
                self.rt.block_on(self.inner.fund_raw_transaction(hex)).map_err(Self::map_err)
            }

            pub fn send_raw_transaction(
                &self,
                tx: &bitcoin::Transaction,
            ) -> Result<vgen::SendRawTransaction> {
                let hex = bitcoin::consensus::encode::serialize_hex(tx);
                self.rt.block_on(self.inner.send_raw_transaction(hex)).map_err(Self::map_err)
            }
        }

        // == Wallet (full surface) ==
        impl Client {
            pub fn abandon_transaction(&self, txid: Txid) -> Result<()> {
                self.rt
                    .block_on(self.inner.abandon_transaction(txid.to_string()))
                    .map_err(Self::map_err)
            }

            pub fn abort_rescan(&self) -> Result<vgen::AbortRescan> {
                self.rt.block_on(self.inner.abort_rescan()).map_err(Self::map_err)
            }

            pub fn backup_wallet(&self, destination: &Path) -> Result<()> {
                self.rt
                    .block_on(self.inner.backup_wallet(destination.to_string_lossy().into_owned()))
                    .map_err(Self::map_err)
            }

            pub fn bump_fee(&self, txid: Txid) -> Result<vgen::BumpFee> {
                self.rt.block_on(self.inner.bump_fee(txid.to_string())).map_err(Self::map_err)
            }

            pub fn create_wallet_descriptor(
                &self,
                address_type: &str,
                hdkey: &str,
            ) -> Result<vgen::CreateWalletDescriptor> {
                let opts = vcli::wallet::CreateWalletDescriptorOptions {
                    hd_key: Some(hdkey.to_owned()),
                    ..Default::default()
                };
                self.rt
                    .block_on(self.inner.create_wallet_descriptor_with(address_type.to_owned(), opts))
                    .map_err(Self::map_err)
            }

            pub fn encrypt_wallet(&self, passphrase: &str) -> Result<vgen::EncryptWallet> {
                self.rt
                    .block_on(self.inner.encrypt_wallet(passphrase.to_owned()))
                    .map_err(Self::map_err)
            }

            pub fn get_addresses_by_label(&self, label: &str) -> Result<vgen::GetAddressesByLabel> {
                self.rt
                    .block_on(self.inner.get_addresses_by_label(label.to_owned()))
                    .map_err(Self::map_err)
            }

            pub fn get_address_info(&self, address: &Address) -> Result<vgen::GetAddressInfo> {
                self.rt
                    .block_on(self.inner.get_address_info(address.to_string()))
                    .map_err(Self::map_err)
            }

            pub fn get_hd_keys(&self) -> Result<vgen::GetHdKeys> {
                self.rt.block_on(self.inner.get_hd_keys()).map_err(Self::map_err)
            }

            pub fn get_raw_change_address(&self) -> Result<vgen::GetRawChangeAddress> {
                self.rt.block_on(self.inner.get_raw_change_address()).map_err(Self::map_err)
            }

            pub fn get_received_by_address(
                &self,
                address: &Address<NetworkChecked>,
            ) -> Result<vgen::GetReceivedByAddress> {
                self.rt
                    .block_on(self.inner.get_received_by_address(address.to_string()))
                    .map_err(Self::map_err)
            }

            pub fn get_received_by_label(&self, label: &str) -> Result<vgen::GetReceivedByLabel> {
                self.rt
                    .block_on(self.inner.get_received_by_label(label.to_owned()))
                    .map_err(Self::map_err)
            }

            pub fn get_wallet_info(&self) -> Result<vgen::GetWalletInfo> {
                self.rt.block_on(self.inner.get_wallet_info()).map_err(Self::map_err)
            }

            pub fn import_descriptors(
                &self,
                requests: &[ImportDescriptorsRequest],
            ) -> Result<vgen::ImportDescriptors> {
                let requests: Vec<vcli::wallet::ImportDescriptorsRequests> =
                    serde_json::from_value(into_json(requests)?)?;
                self.rt.block_on(self.inner.import_descriptors(requests)).map_err(Self::map_err)
            }

            pub fn import_pruned_funds(
                &self,
                raw_transaction: &str,
                tx_out_proof: &str,
            ) -> Result<()> {
                self.rt
                    .block_on(
                        self.inner
                            .import_pruned_funds(raw_transaction.to_owned(), tx_out_proof.to_owned()),
                    )
                    .map_err(Self::map_err)
            }

            pub fn key_pool_refill(&self) -> Result<()> {
                self.rt.block_on(self.inner.keypool_refill()).map_err(Self::map_err)
            }

            pub fn list_address_groupings(&self) -> Result<vgen::ListAddressGroupings> {
                self.rt.block_on(self.inner.list_address_groupings()).map_err(Self::map_err)
            }

            pub fn list_descriptors(&self) -> Result<vgen::ListDescriptors> {
                self.rt.block_on(self.inner.list_descriptors()).map_err(Self::map_err)
            }

            pub fn list_labels(&self) -> Result<vgen::ListLabels> {
                self.rt.block_on(self.inner.list_labels()).map_err(Self::map_err)
            }

            pub fn list_lock_unspent(&self) -> Result<vgen::ListLockUnspent> {
                self.rt.block_on(self.inner.list_lock_unspent()).map_err(Self::map_err)
            }

            pub fn list_received_by_address(&self) -> Result<vgen::ListReceivedByAddress> {
                self.rt.block_on(self.inner.list_received_by_address()).map_err(Self::map_err)
            }

            pub fn list_received_by_label(&self) -> Result<vgen::ListReceivedByLabel> {
                self.rt.block_on(self.inner.list_received_by_label()).map_err(Self::map_err)
            }

            pub fn list_since_block(&self) -> Result<vgen::ListSinceBlock> {
                self.rt.block_on(self.inner.list_since_block()).map_err(Self::map_err)
            }

            pub fn list_transactions(&self) -> Result<vgen::ListTransactions> {
                self.rt.block_on(self.inner.list_transactions()).map_err(Self::map_err)
            }

            pub fn list_unspent(&self) -> Result<vgen::ListUnspent> {
                self.rt.block_on(self.inner.list_unspent()).map_err(Self::map_err)
            }

            pub fn list_wallet_dir(&self) -> Result<vgen::ListWalletDir> {
                self.rt.block_on(self.inner.list_wallet_dir()).map_err(Self::map_err)
            }

            pub fn list_wallets(&self) -> Result<vgen::ListWallets> {
                self.rt.block_on(self.inner.list_wallets()).map_err(Self::map_err)
            }

            fn lock_unspent_inner(
                &self,
                unlock: bool,
                outputs: &[(Txid, u32)],
            ) -> Result<vgen::LockUnspent> {
                let transactions = outputs
                    .iter()
                    .map(|(txid, vout)| vcli::wallet::LockUnspentTransactions {
                        txid: txid.to_string(),
                        vout: *vout as i64,
                    })
                    .collect();
                let opts = vcli::wallet::LockUnspentOptions {
                    transactions: Some(transactions),
                    ..Default::default()
                };
                self.rt.block_on(self.inner.lock_unspent_with(unlock, opts)).map_err(Self::map_err)
            }

            pub fn lock_unspent(&self, outputs: &[(Txid, u32)]) -> Result<vgen::LockUnspent> {
                self.lock_unspent_inner(false, outputs)
            }

            pub fn unlock_unspent(&self, outputs: &[(Txid, u32)]) -> Result<vgen::LockUnspent> {
                self.lock_unspent_inner(true, outputs)
            }

            pub fn migrate_wallet(&self, wallet_name: &str) -> Result<vgen::MigrateWallet> {
                let opts = vcli::wallet::MigrateWalletOptions {
                    wallet_name: Some(wallet_name.to_owned()),
                    ..Default::default()
                };
                self.rt.block_on(self.inner.migrate_wallet_with(opts)).map_err(Self::map_err)
            }

            pub fn psbt_bump_fee(&self, txid: &Txid) -> Result<vgen::PsbtBumpFee> {
                self.rt.block_on(self.inner.psbt_bump_fee(txid.to_string())).map_err(Self::map_err)
            }

            pub fn remove_pruned_funds(&self, txid: Txid) -> Result<()> {
                self.rt
                    .block_on(self.inner.remove_pruned_funds(txid.to_string()))
                    .map_err(Self::map_err)
            }

            pub fn rescan_blockchain(&self) -> Result<vgen::RescanBlockchain> {
                self.rt.block_on(self.inner.rescan_blockchain()).map_err(Self::map_err)
            }

            pub fn restore_wallet(
                &self,
                wallet_name: &str,
                backup_file: &Path,
            ) -> Result<vgen::RestoreWallet> {
                self.rt
                    .block_on(self.inner.restore_wallet(
                        wallet_name.to_owned(),
                        backup_file.to_string_lossy().into_owned(),
                    ))
                    .map_err(Self::map_err)
            }

            pub fn send(&self, outputs: &BTreeMap<String, f64>) -> Result<vgen::SendResult> {
                let outputs: Vec<vcli::wallet::SendResultOutputs> =
                    serde_json::from_value(serde_json::json!([outputs]))?;
                self.rt.block_on(self.inner.send(outputs)).map_err(Self::map_err)
            }

            pub fn send_all(&self, recipients: &[Address]) -> Result<vgen::SendAll> {
                let recipients: Vec<vcli::wallet::SendAllRecipients> = serde_json::from_value(
                    serde_json::json!(recipients.iter().map(|a| a.to_string()).collect::<Vec<_>>()),
                )?;
                self.rt.block_on(self.inner.send_all(recipients)).map_err(Self::map_err)
            }

            fn send_many_amounts(
                amounts: &BTreeMap<Address, Amount>,
            ) -> Result<std::collections::BTreeMap<String, vcli::wallet::SendManyAmounts>> {
                let json: std::collections::BTreeMap<String, f64> = amounts
                    .iter()
                    .map(|(addr, amount)| (addr.to_string(), amount.to_btc()))
                    .collect();
                Ok(serde_json::from_value(into_json(json)?)?)
            }

            pub fn send_many(
                &self,
                amounts: BTreeMap<Address, Amount>,
            ) -> Result<vgen::SendManyVerbose0> {
                let amounts = Self::send_many_amounts(&amounts)?;
                self.rt.block_on(self.inner.send_many_verbose_0(amounts)).map_err(Self::map_err)
            }

            pub fn send_many_verbose(
                &self,
                amounts: BTreeMap<Address, Amount>,
            ) -> Result<vgen::SendManyVerbose1> {
                let amounts = Self::send_many_amounts(&amounts)?;
                self.rt.block_on(self.inner.send_many_verbose_1(amounts)).map_err(Self::map_err)
            }

            pub fn send_to_address_rbf(
                &self,
                address: &Address<NetworkChecked>,
                amount: Amount,
            ) -> Result<vgen::SendToAddressVerbose0> {
                let amount: vcli::wallet::SendToAddressAmount =
                    serde_json::from_value(into_json(amount.to_btc())?)?;
                let opts = vcli::wallet::SendToAddressOptions {
                    replaceable: Some(true),
                    ..Default::default()
                };
                self.rt
                    .block_on(self.inner.send_to_address_verbose_0_with(
                        address.to_string(),
                        amount,
                        opts,
                    ))
                    .map_err(Self::map_err)
            }

            pub fn set_wallet_flag(&self, flag: &str) -> Result<vgen::SetWalletFlag> {
                self.rt.block_on(self.inner.set_wallet_flag(flag.to_owned())).map_err(Self::map_err)
            }

            pub fn sign_message(&self, address: &Address, message: &str) -> Result<vgen::SignMessage> {
                self.rt
                    .block_on(self.inner.sign_message(address.to_string(), message.to_owned()))
                    .map_err(Self::map_err)
            }

            pub fn simulate_raw_transaction(
                &self,
                rawtxs: &[String],
            ) -> Result<vgen::SimulateRawTransaction> {
                let opts = vcli::wallet::SimulateRawTransactionOptions {
                    raw_txs: Some(rawtxs.to_vec()),
                    ..Default::default()
                };
                self.rt
                    .block_on(self.inner.simulate_raw_transaction_with(opts))
                    .map_err(Self::map_err)
            }

            pub fn unload_wallet(&self, wallet: &str) -> Result<vgen::UnloadWallet> {
                let opts = vcli::wallet::UnloadWalletOptions {
                    wallet_name: Some(wallet.to_owned()),
                    ..Default::default()
                };
                self.rt.block_on(self.inner.unload_wallet_with(opts)).map_err(Self::map_err)
            }

            pub fn wallet_create_funded_psbt(
                &self,
                inputs: Vec<WalletCreateFundedPsbtInput>,
                outputs: Vec<BTreeMap<Address, Amount>>,
            ) -> Result<vgen::WalletCreateFundedPsbt> {
                let outputs_json: Vec<std::collections::BTreeMap<String, f64>> = outputs
                    .iter()
                    .map(|map| {
                        map.iter().map(|(addr, amount)| (addr.to_string(), amount.to_btc())).collect()
                    })
                    .collect();
                let outputs: Vec<vcli::wallet::WalletCreateFundedPsbtOutputs> =
                    serde_json::from_value(into_json(outputs_json)?)?;
                let inputs: Vec<vcli::wallet::WalletCreateFundedPsbtInputs> =
                    serde_json::from_value(into_json(inputs)?)?;
                let opts = vcli::wallet::WalletCreateFundedPsbtOptions {
                    inputs: Some(inputs),
                    ..Default::default()
                };
                self.rt
                    .block_on(self.inner.wallet_create_funded_psbt_with(outputs, opts))
                    .map_err(Self::map_err)
            }

            pub fn wallet_display_address(&self, address: &str) -> Result<vgen::WalletDisplayAddress> {
                self.rt
                    .block_on(self.inner.wallet_display_address(address.to_owned()))
                    .map_err(Self::map_err)
            }

            pub fn wallet_lock(&self) -> Result<()> {
                self.rt.block_on(self.inner.wallet_lock()).map_err(Self::map_err)
            }

            pub fn wallet_passphrase(&self, passphrase: &str, timeout: u64) -> Result<()> {
                self.rt
                    .block_on(self.inner.wallet_passphrase(passphrase.to_owned(), timeout as i64))
                    .map_err(Self::map_err)
            }

            pub fn wallet_passphrase_change(
                &self,
                old_passphrase: &str,
                new_passphrase: &str,
            ) -> Result<()> {
                self.rt
                    .block_on(
                        self.inner.wallet_passphrase_change(
                            old_passphrase.to_owned(),
                            new_passphrase.to_owned(),
                        ),
                    )
                    .map_err(Self::map_err)
            }

            pub fn wallet_process_psbt(&self, psbt: &bitcoin::Psbt) -> Result<vgen::WalletProcessPsbt> {
                self.rt
                    .block_on(self.inner.wallet_process_psbt(psbt.to_string()))
                    .map_err(Self::map_err)
            }
        }

        // == Rawtransactions (full surface) ==
        impl Client {
            pub fn abort_private_broadcast(&self, id: &str) -> Result<vgen::AbortPrivateBroadcast> {
                self.rt
                    .block_on(self.inner.abort_private_broadcast(id.to_owned()))
                    .map_err(Self::map_err)
            }

            pub fn analyze_psbt(&self, psbt: &bitcoin::Psbt) -> Result<vgen::AnalyzePsbt> {
                self.rt.block_on(self.inner.analyze_psbt(psbt.to_string())).map_err(Self::map_err)
            }

            pub fn combine_psbt(&self, txs: &[bitcoin::Psbt]) -> Result<vgen::CombinePsbt> {
                let txs = txs.iter().map(|p| p.to_string()).collect();
                self.rt.block_on(self.inner.combine_psbt(txs)).map_err(Self::map_err)
            }

            pub fn combine_raw_transaction(
                &self,
                txs: &[bitcoin::Transaction],
            ) -> Result<vgen::CombineRawTransaction> {
                let txs =
                    txs.iter().map(|tx| bitcoin::consensus::encode::serialize_hex(tx)).collect();
                self.rt.block_on(self.inner.combine_raw_transaction(txs)).map_err(Self::map_err)
            }

            pub fn convert_to_psbt(&self, tx: &bitcoin::Transaction) -> Result<vgen::ConvertToPsbt> {
                let hex = bitcoin::consensus::encode::serialize_hex(tx);
                self.rt.block_on(self.inner.convert_to_psbt(hex)).map_err(Self::map_err)
            }

            pub fn create_psbt(&self, inputs: &[Input], outputs: &[Output]) -> Result<vgen::CreatePsbt> {
                let inputs: Vec<vcli::raw_transactions::CreatePsbtInputs> =
                    serde_json::from_value(into_json(inputs)?)?;
                let outputs: Vec<vcli::raw_transactions::CreatePsbtOutputs> =
                    serde_json::from_value(into_json(outputs)?)?;
                self.rt.block_on(self.inner.create_psbt(inputs, outputs)).map_err(Self::map_err)
            }

            pub fn decode_psbt(&self, psbt: &str) -> Result<vgen::DecodePsbt> {
                self.rt.block_on(self.inner.decode_psbt(psbt.to_owned())).map_err(Self::map_err)
            }

            pub fn decode_raw_transaction(
                &self,
                tx: &bitcoin::Transaction,
            ) -> Result<vgen::DecodeRawTransaction> {
                let hex = bitcoin::consensus::encode::serialize_hex(tx);
                self.rt.block_on(self.inner.decode_raw_transaction(hex)).map_err(Self::map_err)
            }

            pub fn decode_script(&self, script: &str) -> Result<vgen::DecodeScript> {
                self.rt.block_on(self.inner.decode_script(script.to_owned())).map_err(Self::map_err)
            }

            pub fn finalize_psbt(&self, psbt: &bitcoin::Psbt) -> Result<vgen::FinalizePsbt> {
                self.rt.block_on(self.inner.finalize_psbt(psbt.to_string())).map_err(Self::map_err)
            }

            pub fn get_private_broadcast_info(&self) -> Result<vgen::GetPrivateBroadcastInfo> {
                self.rt.block_on(self.inner.get_private_broadcast_info()).map_err(Self::map_err)
            }

            pub fn get_raw_transaction(&self, txid: Txid) -> Result<vgen::GetRawTransactionVerbose0> {
                self.rt
                    .block_on(self.inner.get_raw_transaction_verbose_0(txid.to_string()))
                    .map_err(Self::map_err)
            }

            pub fn get_raw_transaction_verbose(
                &self,
                txid: Txid,
            ) -> Result<vgen::GetRawTransactionVerbose1> {
                self.rt
                    .block_on(self.inner.get_raw_transaction_verbose_1(txid.to_string()))
                    .map_err(Self::map_err)
            }

            pub fn join_psbts(&self, psbts: &[bitcoin::Psbt]) -> Result<vgen::JoinPsbts> {
                let txs = psbts.iter().map(|p| p.to_string()).collect();
                self.rt.block_on(self.inner.join_psbts(txs)).map_err(Self::map_err)
            }

            pub fn sign_raw_transaction_with_key(
                &self,
                tx: &bitcoin::Transaction,
                keys: &[bitcoin::PrivateKey],
            ) -> Result<vgen::SignRawTransactionWithKey> {
                let hex = bitcoin::consensus::encode::serialize_hex(tx);
                let keys = keys.iter().map(|k| k.to_wif()).collect();
                self.rt
                    .block_on(self.inner.sign_raw_transaction_with_key(hex, keys))
                    .map_err(Self::map_err)
            }

            pub fn submit_package(
                &self,
                package: &[bitcoin::Transaction],
                max_fee_rate: Option<bitcoin::FeeRate>,
                max_burn_amount: Option<Amount>,
            ) -> Result<vgen::SubmitPackage> {
                let package: Vec<String> =
                    package.iter().map(|tx| bitcoin::consensus::encode::serialize_hex(tx)).collect();
                if max_fee_rate.is_none() && max_burn_amount.is_none() {
                    return self
                        .rt
                        .block_on(self.inner.submit_package(package))
                        .map_err(Self::map_err);
                }
                let max_fee_rate = match max_fee_rate {
                    Some(rate) => Some(serde_json::from_value(into_json(
                        rate.to_sat_per_kwu() as f64 * 4.0 / 1000.0,
                    )?)?),
                    None => None,
                };
                let max_burn_amount = match max_burn_amount {
                    Some(amount) => Some(serde_json::from_value(into_json(amount.to_btc())?)?),
                    None => None,
                };
                let opts = vcli::raw_transactions::SubmitPackageOptions {
                    max_fee_rate,
                    max_burn_amount,
                };
                self.rt.block_on(self.inner.submit_package_with(package, opts)).map_err(Self::map_err)
            }

            pub fn test_mempool_accept(
                &self,
                txs: &[bitcoin::Transaction],
            ) -> Result<vgen::TestMempoolAccept> {
                let raw_txs =
                    txs.iter().map(|tx| bitcoin::consensus::encode::serialize_hex(tx)).collect();
                self.rt.block_on(self.inner.test_mempool_accept(raw_txs)).map_err(Self::map_err)
            }

            pub fn utxo_update_psbt(&self, psbt: &bitcoin::Psbt) -> Result<vgen::UtxoUpdatePsbt> {
                self.rt.block_on(self.inner.utxo_update_psbt(psbt.to_string())).map_err(Self::map_err)
            }
        }

        // == Util ==
        impl Client {
            pub fn create_multisig(
                &self,
                nrequired: u32,
                keys: Vec<PublicKey>,
            ) -> Result<vgen::CreateMultisig> {
                let keys = keys.iter().map(|k| k.to_string()).collect();
                self.rt
                    .block_on(self.inner.create_multisig(nrequired as i64, keys))
                    .map_err(Self::map_err)
            }

            pub fn derive_addresses(&self, descriptor: &str) -> Result<vgen::DeriveAddresses> {
                self.rt
                    .block_on(self.inner.derive_addresses(descriptor.to_owned()))
                    .map_err(Self::map_err)
            }

            pub fn estimate_smart_fee(&self, blocks: u32) -> Result<vgen::EstimateSmartFee> {
                self.rt
                    .block_on(self.inner.estimate_smart_fee(blocks as i64))
                    .map_err(Self::map_err)
            }

            pub fn estimate_smart_fee_with_mode(
                &self,
                blocks: u32,
                mode: FeeEstimateMode,
            ) -> Result<vgen::EstimateSmartFee> {
                let estimate_mode = serde_json::from_value::<String>(into_json(mode)?)?;
                let opts = vcli::util::EstimateSmartFeeOptions {
                    estimate_mode: Some(estimate_mode),
                    ..Default::default()
                };
                self.rt
                    .block_on(self.inner.estimate_smart_fee_with(blocks as i64, opts))
                    .map_err(Self::map_err)
            }

            pub fn get_descriptor_info(&self, descriptor: &str) -> Result<vgen::GetDescriptorInfo> {
                self.rt
                    .block_on(self.inner.get_descriptor_info(descriptor.to_owned()))
                    .map_err(Self::map_err)
            }

            pub fn get_index_info(&self) -> Result<vgen::GetIndexInfo> {
                self.rt.block_on(self.inner.get_index_info()).map_err(Self::map_err)
            }

            pub fn sign_message_with_privkey(
                &self,
                privkey: &bitcoin::PrivateKey,
                message: &str,
            ) -> Result<vgen::SignMessageWithPrivKey> {
                self.rt
                    .block_on(self.inner.sign_message_with_priv_key(privkey.to_wif(), message.to_owned()))
                    .map_err(Self::map_err)
            }

            pub fn validate_address(
                &self,
                address: &Address<NetworkChecked>,
            ) -> Result<vgen::ValidateAddress> {
                self.rt
                    .block_on(self.inner.validate_address(address.to_string()))
                    .map_err(Self::map_err)
            }

            pub fn verify_message(
                &self,
                address: &Address<NetworkChecked>,
                signature: &sign_message::MessageSignature,
                message: &str,
            ) -> Result<vgen::VerifyMessage> {
                self.rt
                    .block_on(self.inner.verify_message(
                        address.to_string(),
                        signature.to_string(),
                        message.to_owned(),
                    ))
                    .map_err(Self::map_err)
            }
        }

        // == Signer ==
        impl Client {
            pub fn enumerate_signers(&self) -> Result<vgen::EnumerateSigners> {
                self.rt.block_on(self.inner.enumerate_signers()).map_err(Self::map_err)
            }
        }

        // == Zmq ==
        // Not in Core's OpenRPC spec: raw call returning the curated type (read raw by the test).
        impl Client {
            pub fn get_zmq_notifications(&self) -> Result<Vec<GetZmqNotifications>> {
                self.call("getzmqnotifications", &[])
            }
        }
    };
}
