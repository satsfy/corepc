// SPDX-License-Identifier: CC0-1.0

//! Isolation bridges for the async blocking facade.
//!
//! These are the methods the facade implements *itself* instead of reusing the sync client's
//! `impl_client_*` macros, so a bug in the sync argument-encoding cannot reach the async path. The
//! generated `client_async/v{N}/blocking.rs` invokes [`impl_async_bridges!`] once and skips the
//! matching sync macros.
//!
//! Unlike codegen string templates, this is real Rust: the compiler, `rustfmt` and `rust-analyzer`
//! all check it. The version is passed as a token (`impl_async_bridges!(v31)`) so the
//! version-specific paths (`$crate::types::v31::...`) resolve; the curated response types are
//! resolved unqualified at the call site via the facade's `use crate::types::v{N}::*`.
//!
//! Adding a bridge: add the method here AND add its sync-macro suffix to `BRIDGED_METHODS` in
//! `codegen/src/codegen/blocking.rs` so the reused sync macro is skipped (no duplicate definition).

/// Emit the blocking facade's bridged methods for version `$v`, e.g. `impl_async_bridges!(v31)`.
///
/// Expanded inside `client_async/v{N}/blocking.rs`, where `Client`, `Result`, `Error`, `into_json`,
/// `AddressType` and the curated response types are all in scope.
#[macro_export]
macro_rules! impl_async_bridges {
    ($v:ident) => {
        // == Wallet ==
        // `getnewaddress` routes through the GENERATED async wrapper (its own argument encoding),
        // NOT the reused sync macro, so the sync client's arg-encoding (and any bug in it) cannot
        // reach the async path.
        impl Client {
            fn get_new_address_generated(
                &self,
                label: Option<&str>,
                ty: Option<AddressType>,
            ) -> Result<$crate::types::$v::generated::GetNewAddress> {
                let address_type = match ty {
                    Some(ty) => Some(serde_json::from_value::<String>(into_json(ty)?)?),
                    None => None,
                };
                let opts = $crate::client_async::$v::wallet::GetNewAddressOptions {
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
            ) -> Result<$crate::types::$v::generated::GetNewAddress> {
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
        }

        // == Generating ==
        // These RPCs are not in Core's OpenRPC, so there is no generated async wrapper to route
        // through; the facade owns a raw `self.call`, which still isolates it from the sync macro.
        impl Client {
            pub fn generate_block(
                &self,
                output: &str,
                transactions: &[String],
                submit: bool,
            ) -> Result<GenerateBlock> {
                self.call(
                    "generateblock",
                    &[into_json(output)?, into_json(transactions)?, into_json(submit)?],
                )
            }

            pub fn generate_to_address(
                &self,
                nblocks: usize,
                address: &bitcoin::Address,
            ) -> Result<GenerateToAddress> {
                self.call("generatetoaddress", &[nblocks.into(), into_json(address)?])
            }

            pub fn generate_to_descriptor(
                &self,
                nblocks: usize,
                descriptor: &str,
            ) -> Result<GenerateToDescriptor> {
                self.call("generatetodescriptor", &[nblocks.into(), descriptor.into()])
            }

            pub fn invalidate_block(&self, hash: BlockHash) -> Result<()> {
                match self.call("invalidateblock", &[into_json(hash)?]) {
                    Ok(serde_json::Value::Null) => Ok(()),
                    Ok(res) => Err(Error::Returned(res.to_string())),
                    Err(err) => Err(err.into()),
                }
            }
        }

        // == Mining ==
        // Each method routes through the GENERATED async wrapper on `self.inner` (its own
        // arg-encoding plus generated response type), NOT a raw `self.call`, so the generated call
        // surface is exercised and stays isolated from the sync macro. Where the generated response
        // type differs from the curated type the explicit tests consume, the response is converted
        // back through JSON (both sides serialize to Core's shape).
        impl Client {
            // Returns the GENERATED `Object` variant (`GetBlockTemplateVariant2`), which now has its
            // own `into_model`; the facade's `vtype` aliases it to `GetBlockTemplate` so the test's
            // `into_model()` runs the generated conversion. `Null`/`Text` (proposal-mode replies) are
            // not templates, so they are an error here.
            pub fn get_block_template(
                &self,
                request: &TemplateRequest,
            ) -> Result<$crate::types::$v::generated::GetBlockTemplateVariant2> {
                // Curated `TemplateRequest` -> the wrapper's own request struct (`rules` is
                // `Vec<TemplateRules>` here, `Vec<String>` there; both serialize to Core's shape).
                let req: $crate::client_async::$v::mining::GetBlockTemplateTemplateRequest =
                    serde_json::from_value(into_json(request)?)?;
                match self.rt.block_on(self.inner.get_block_template(req)).map_err(Self::map_err)? {
                    $crate::types::$v::generated::GetBlockTemplate::Object(v) => Ok(v),
                    $crate::types::$v::generated::GetBlockTemplate::Null(()) =>
                        Err(Error::Returned("getblocktemplate returned null".to_owned())),
                    $crate::types::$v::generated::GetBlockTemplate::Text(s) =>
                        Err(Error::Returned(format!("getblocktemplate returned: {s}"))),
                }
            }

            // `get_mining_info` returns the GENERATED response type directly (no JSON round-trip):
            // the facade's `vtype` shim re-exports the generated `GetMiningInfo`/`GetMiningInfoError`
            // for these names, so the unchanged test's `json.into_model()` runs the GENERATED
            // `into_model`, pnot the curated one. This is the explicit path; the other struct methods
            // still round-trip to the curated type (their generated `into_model` is missing or their
            // test pins the curated error type).
            pub fn get_mining_info(&self) -> Result<$crate::types::$v::generated::GetMiningInfo> {
                self.rt.block_on(self.inner.get_mining_info()).map_err(Self::map_err)
            }

            pub fn get_network_hash_ps(&self) -> Result<f64> {
                Ok(*self.rt.block_on(self.inner.get_network_hash_ps()).map_err(Self::map_err)?)
            }

            pub fn get_prioritised_transactions(
                &self,
            ) -> Result<$crate::types::$v::generated::GetPrioritisedTransactions> {
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
                    $crate::types::$v::generated::SubmitBlock::Null(()) => Ok(()),
                    $crate::types::$v::generated::SubmitBlock::Text(reason) =>
                        Err(Error::Returned(reason)),
                }
            }

            pub fn submit_header(&self, header: &bitcoin::block::Header) -> Result<()> {
                let hexdata = bitcoin::consensus::encode::serialize_hex(header);
                self.rt.block_on(self.inner.submit_header(hexdata)).map_err(Self::map_err)
            }
        }

        // == Network ==
        // Every method routes through the generated async wrapper on `self.inner`, isolated from the
        // sync macros. `get_network_info` returns the GENERATED response type (the facade's `vtype`
        // aliases it) so the test's `into_model()` runs the GENERATED conversion. The raw-accessed
        // responses round-trip generated -> curated through JSON so the tests' field access compiles
        // unchanged; unit/primitive methods extract from the wrapper. `add_peer_address` is a hidden
        // RPC with no generated wrapper, so it owns a raw `self.call` (still off the sync macro).
        impl Client {
            pub fn add_node(&self, node: &str, command: AddNodeCommand) -> Result<()> {
                let command = serde_json::from_value::<String>(into_json(command)?)?;
                self.rt.block_on(self.inner.add_node(node.to_owned(), command)).map_err(Self::map_err)
            }

            pub fn clear_banned(&self) -> Result<()> {
                self.rt.block_on(self.inner.clear_banned()).map_err(Self::map_err)
            }

            pub fn disconnect_node(&self, address: &str) -> Result<()> {
                let opts = $crate::client_async::$v::network::DisconnectNodeOptions {
                    address: Some(address.to_owned()),
                    nodeid: None,
                };
                self.rt.block_on(self.inner.disconnect_node_with(opts)).map_err(Self::map_err)
            }

            pub fn get_added_node_info(&self) -> Result<GetAddedNodeInfo> {
                let res = self.rt.block_on(self.inner.get_added_node_info()).map_err(Self::map_err)?;
                Ok(serde_json::from_value(into_json(res)?)?)
            }

            pub fn get_addr_man_info(&self) -> Result<GetAddrManInfo> {
                let res = self.rt.block_on(self.inner.get_addr_man_info()).map_err(Self::map_err)?;
                Ok(serde_json::from_value(into_json(res)?)?)
            }

            pub fn get_connection_count(&self) -> Result<GetConnectionCount> {
                let res =
                    self.rt.block_on(self.inner.get_connection_count()).map_err(Self::map_err)?;
                Ok(serde_json::from_value(into_json(res)?)?)
            }

            pub fn get_net_totals(&self) -> Result<GetNetTotals> {
                let res = self.rt.block_on(self.inner.get_net_totals()).map_err(Self::map_err)?;
                Ok(serde_json::from_value(into_json(res)?)?)
            }

            /// Server version field of `getnetworkinfo`; `check_expected_server_version` calls this
            /// (the sync `get_network_info` macro that normally defines it is skipped here).
            pub fn server_version(&self) -> Result<usize> {
                let res = self.rt.block_on(self.inner.get_network_info()).map_err(Self::map_err)?;
                usize::try_from(res.version).map_err(|e| Error::Returned(e.to_string()))
            }

            pub fn get_network_info(&self) -> Result<$crate::types::$v::generated::GetNetworkInfo> {
                self.rt.block_on(self.inner.get_network_info()).map_err(Self::map_err)
            }

            pub fn get_node_addresses(&self) -> Result<GetNodeAddresses> {
                let res = self.rt.block_on(self.inner.get_node_addresses()).map_err(Self::map_err)?;
                Ok(serde_json::from_value(into_json(res)?)?)
            }

            pub fn add_peer_address(&self, address: &str, port: u16) -> Result<AddPeerAddress> {
                self.call("addpeeraddress", &[address.into(), port.into()])
            }

            pub fn get_peer_info(&self) -> Result<GetPeerInfo> {
                let res = self.rt.block_on(self.inner.get_peer_info()).map_err(Self::map_err)?;
                Ok(serde_json::from_value(into_json(res)?)?)
            }

            pub fn list_banned(&self) -> Result<ListBanned> {
                let res = self.rt.block_on(self.inner.list_banned()).map_err(Self::map_err)?;
                Ok(serde_json::from_value(into_json(res)?)?)
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

            pub fn set_network_active(&self, state: bool) -> Result<SetNetworkActive> {
                let res =
                    self.rt.block_on(self.inner.set_network_active(state)).map_err(Self::map_err)?;
                Ok(serde_json::from_value(into_json(res)?)?)
            }
        }
    };
}
