// SPDX-License-Identifier: CC0-1.0

use alloc::collections::BTreeMap;

use bitcoin::consensus::encode;
use bitcoin::{
    block, Amount, BlockHash, CompactTarget, OutPoint, Target, Transaction, TxMerkleNode, Txid,
    Weight, Work, Wtxid,
};

use super::{
    CoinbaseTransaction, GetBlockVerboseOne, GetBlockVerboseThree, GetBlockVerboseTwo,
    GetDeploymentInfo, GetMempoolAncestorsVerbose, GetMempoolCluster, GetMempoolClusterError,
    GetMempoolDescendantsVerbose, GetMempoolEntry, GetMempoolFeerateDiagram,
    GetMempoolFeerateDiagramError, GetMempoolInfo, GetMempoolInfoError, GetRawMempoolVerbose,
    GetTxSpendingPrevout, GetTxSpendingPrevoutError, GetTxSpendingPrevoutItem,
    MapMempoolEntryError, MempoolEntry, MempoolEntryError, MempoolEntryFees, MempoolEntryFeesError,
};
use crate::model;

impl GetMempoolCluster {
    /// Converts version specific type to a version nonspecific, more strongly typed type.
    pub fn into_model(self) -> Result<model::GetMempoolCluster, GetMempoolClusterError> {
        // TODO: Use combinators.
        use GetMempoolClusterError as E;

        let mut chunks = vec![];
        for chunk in self.chunks {
            let txs = chunk
                .txs
                .iter()
                .map(|txid| txid.parse::<Txid>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(E::Txid)?;
            chunks.push(model::Chunk {
                chunk_fee: Amount::from_btc(chunk.chunk_fee).map_err(E::ChunkFee)?,
                chunk_weight: chunk.chunk_weight,
                txs,
            })
        }

        Ok(model::GetMempoolCluster {
            cluster_weight: self.cluster_weight,
            tx_count: self.tx_count,
            chunks,
        })
    }
}

impl GetMempoolEntry {
    /// Converts version specific type to a version nonspecific, more strongly typed type.
    pub fn into_model(self) -> Result<model::GetMempoolEntry, MempoolEntryError> {
        Ok(model::GetMempoolEntry(self.0.into_model()?))
    }
}

impl MempoolEntry {
    /// Converts version specific type to a version nonspecific, more strongly typed type.
    pub fn into_model(self) -> Result<model::MempoolEntry, MempoolEntryError> {
        use MempoolEntryError as E;

        let vsize = Some(crate::to_u32(self.vsize, "vsize")?);
        let size = None;
        let weight = Some(crate::to_u32(self.weight, "weight")?);
        let chunk_weight = Some(crate::to_u32(self.chunk_weight, "chunk_weight")?);
        let time = crate::to_u32(self.time, "time")?;
        let height = crate::to_u32(self.height, "height")?;
        let descendant_count = crate::to_u32(self.descendant_count, "descendant_count")?;
        let descendant_size = crate::to_u32(self.descendant_size, "descendant_size")?;
        let ancestor_count = crate::to_u32(self.ancestor_count, "ancestor_count")?;
        let ancestor_size = crate::to_u32(self.ancestor_size, "ancestor_size")?;
        let wtxid = self.wtxid.parse::<Wtxid>().map_err(E::Wtxid)?;
        let fees = self.fees.into_model().map_err(E::Fees)?;
        let depends = self
            .depends
            .iter()
            .map(|txid| txid.parse::<Txid>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(E::Depends)?;
        let spent_by = self
            .spent_by
            .iter()
            .map(|txid| txid.parse::<Txid>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(E::SpentBy)?;

        Ok(model::MempoolEntry {
            vsize,
            size,
            weight,
            time,
            height,
            descendant_count,
            descendant_size,
            ancestor_count,
            ancestor_size,
            chunk_weight,
            wtxid,
            fees,
            depends,
            spent_by,
            bip125_replaceable: Some(self.bip125_replaceable),
            unbroadcast: Some(self.unbroadcast),
        })
    }
}

impl MempoolEntryFees {
    /// Converts version specific type to a version nonspecific, more strongly typed type.
    pub fn into_model(self) -> Result<model::MempoolEntryFees, MempoolEntryFeesError> {
        use MempoolEntryFeesError as E;

        Ok(model::MempoolEntryFees {
            base: Amount::from_btc(self.base).map_err(E::Base)?,
            modified: Amount::from_btc(self.modified).map_err(E::Modified)?,
            ancestor: Amount::from_btc(self.ancestor).map_err(E::Ancestor)?,
            descendant: Amount::from_btc(self.descendant).map_err(E::Descendant)?,
            chunk: Some(Amount::from_btc(self.chunk).map_err(E::Chunk)?),
        })
    }
}

impl GetRawMempoolVerbose {
    /// Converts version specific type to a version nonspecific, more strongly typed type.
    pub fn into_model(self) -> Result<model::GetRawMempoolVerbose, MapMempoolEntryError> {
        use MapMempoolEntryError as E;

        let mut map = BTreeMap::new();
        for (k, v) in self.0.into_iter() {
            let txid = k.parse::<Txid>().map_err(E::Txid)?;
            let relative = v.into_model().map_err(E::MempoolEntry)?;
            map.insert(txid, relative);
        }
        Ok(model::GetRawMempoolVerbose(map))
    }
}

impl GetMempoolAncestorsVerbose {
    /// Converts version specific type to a version nonspecific, more strongly typed type.
    pub fn into_model(self) -> Result<model::GetMempoolAncestorsVerbose, MapMempoolEntryError> {
        use MapMempoolEntryError as E;

        let mut map = BTreeMap::new();
        for (k, v) in self.0.into_iter() {
            let txid = k.parse::<Txid>().map_err(E::Txid)?;
            let relative = v.into_model().map_err(E::MempoolEntry)?;
            map.insert(txid, relative);
        }
        Ok(model::GetMempoolAncestorsVerbose(map))
    }
}

impl GetMempoolDescendantsVerbose {
    /// Converts version specific type to a version nonspecific, more strongly typed type.
    pub fn into_model(self) -> Result<model::GetMempoolDescendantsVerbose, MapMempoolEntryError> {
        use MapMempoolEntryError as E;

        let mut map = BTreeMap::new();
        for (k, v) in self.0.into_iter() {
            let txid = k.parse::<Txid>().map_err(E::Txid)?;
            let relative = v.into_model().map_err(E::MempoolEntry)?;
            map.insert(txid, relative);
        }
        Ok(model::GetMempoolDescendantsVerbose(map))
    }
}

impl GetMempoolInfo {
    /// Converts version specific type to a version nonspecific, more strongly typed type.
    pub fn into_model(self) -> Result<model::GetMempoolInfo, GetMempoolInfoError> {
        let size = crate::to_u32(self.size, "size")?;
        let bytes = crate::to_u32(self.bytes, "bytes")?;
        let usage = crate::to_u32(self.usage, "usage")?;
        let max_mempool = crate::to_u32(self.max_mempool, "max_mempool")?;
        let mempool_min_fee = crate::btc_per_kb(self.mempool_min_fee)?;
        let min_relay_tx_fee = crate::btc_per_kb(self.min_relay_tx_fee)?;
        let incremental_relay_fee = crate::btc_per_kb(self.incremental_relay_fee)?;
        let unbroadcast_count = Some(crate::to_u32(self.unbroadcast_count, "unbroadcast_count")?);
        let limit_cluster_count =
            Some(crate::to_u32(self.limit_cluster_count, "limit_cluster_count")?);
        let limit_cluster_size =
            Some(crate::to_u32(self.limit_cluster_size, "limit_cluster_size")?);

        Ok(model::GetMempoolInfo {
            loaded: Some(self.loaded),
            size,
            bytes,
            usage,
            total_fee: Some(self.total_fee),
            max_mempool,
            mempool_min_fee,
            min_relay_tx_fee,
            incremental_relay_fee,
            unbroadcast_count,
            full_rbf: Some(self.full_rbf),
            permit_bare_multisig: Some(self.permit_bare_multisig),
            max_data_carrier_size: Some(self.max_data_carrier_size),
            limit_cluster_count,
            limit_cluster_size,
            optimal: Some(self.optimal),
        })
    }
}

impl GetMempoolFeerateDiagram {
    /// Converts version specific type to a version nonspecific, more strongly typed type.
    pub fn into_model(
        self,
    ) -> Result<model::GetMempoolFeerateDiagram, GetMempoolFeerateDiagramError> {
        use GetMempoolFeerateDiagramError as E;

        let mut entries = vec![];
        for entry in self.0 {
            entries.push(model::FeerateDiagramEntry {
                weight: crate::to_u64(entry.weight, "weight")?,
                fee: Amount::from_btc(entry.fee).map_err(E::Fee)?,
            });
        }
        Ok(model::GetMempoolFeerateDiagram(entries))
    }
}

impl GetDeploymentInfo {
    /// Converts version specific type to a version nonspecific, more strongly typed type.
    pub fn into_model(
        self,
    ) -> Result<model::GetDeploymentInfo, crate::v23::GetDeploymentInfoError> {
        use crate::v23::GetDeploymentInfoError as E;

        let hash = self.hash.parse::<BlockHash>().map_err(E::BlockHash)?;
        let deployments = self
            .deployments
            .into_iter()
            .map(|(name, dep)| dep.into_model().map(|d| (name, d)).map_err(E::Deployment))
            .collect::<Result<_, _>>()?;

        Ok(model::GetDeploymentInfo {
            hash,
            height: self.height,
            script_flags: Some(self.script_flags),
            deployments,
        })
    }
}

impl GetTxSpendingPrevout {
    /// Converts version specific type to a version nonspecific, more strongly typed type.
    pub fn into_model(self) -> Result<model::GetTxSpendingPrevout, GetTxSpendingPrevoutError> {
        let items =
            self.0.into_iter().map(|item| item.into_model()).collect::<Result<Vec<_>, _>>()?;
        Ok(model::GetTxSpendingPrevout(items))
    }
}

impl GetTxSpendingPrevoutItem {
    /// Converts version specific type to a version nonspecific, more strongly typed type.
    pub fn into_model(self) -> Result<model::GetTxSpendingPrevoutItem, GetTxSpendingPrevoutError> {
        use GetTxSpendingPrevoutError as E;

        let txid = self.txid.parse::<Txid>().map_err(E::Txid)?;
        let outpoint = OutPoint { txid, vout: self.vout };
        let spending_txid =
            self.spending_txid.map(|id| id.parse::<Txid>().map_err(E::SpendingTxid)).transpose()?;
        let spending_tx = self
            .spending_tx
            .map(|hex| encode::deserialize_hex::<Transaction>(&hex).map_err(E::SpendingTx))
            .transpose()?;
        let block_hash =
            self.block_hash.map(|h| h.parse::<BlockHash>().map_err(E::BlockHash)).transpose()?;

        Ok(model::GetTxSpendingPrevoutItem { outpoint, spending_txid, spending_tx, block_hash })
    }
}

impl CoinbaseTransaction {
    /// Converts version specific type to a version nonspecific, more strongly typed type.
    pub fn into_model(self) -> model::CoinbaseTransaction {
        model::CoinbaseTransaction {
            version: self.version,
            locktime: self.locktime,
            sequence: self.sequence,
            coinbase: self.coinbase,
            witness: self.witness,
        }
    }
}

impl GetBlockVerboseOne {
    /// Converts version specific type to a version nonspecific, more strongly typed type.
    pub fn into_model(
        self,
    ) -> Result<model::GetBlockVerboseOne, crate::v29::GetBlockVerboseOneError> {
        use crate::v29::GetBlockVerboseOneError as E;

        let hash = self.hash.parse::<BlockHash>().map_err(E::Hash)?;
        let stripped_size =
            self.stripped_size.map(|size| crate::to_u32(size, "stripped_size")).transpose()?;
        let weight = Weight::from_wu(self.weight);
        let version = block::Version::from_consensus(self.version);
        let merkle_root = self.merkle_root.parse::<TxMerkleNode>().map_err(E::MerkleRoot)?;
        let tx = self
            .tx
            .iter()
            .map(|t| t.parse::<Txid>().map_err(E::Hash))
            .collect::<Result<Vec<_>, _>>()?;
        let median_time = self.median_time.map(|t| crate::to_u32(t, "median_time")).transpose()?;
        let bits = CompactTarget::from_unprefixed_hex(&self.bits).map_err(E::Bits)?;
        let target = Some(Target::from_unprefixed_hex(self.target.as_ref()).map_err(E::Target)?);
        let chain_work = Work::from_unprefixed_hex(&self.chain_work).map_err(E::ChainWork)?;
        let previous_block_hash = self
            .previous_block_hash
            .map(|s| s.parse::<BlockHash>())
            .transpose()
            .map_err(E::PreviousBlockHash)?;
        let next_block_hash = self
            .next_block_hash
            .map(|s| s.parse::<BlockHash>())
            .transpose()
            .map_err(E::NextBlockHash)?;

        Ok(model::GetBlockVerboseOne {
            hash,
            confirmations: self.confirmations,
            size: crate::to_u32(self.size, "size")?,
            stripped_size,
            weight,
            coinbase_tx: Some(self.coinbase_tx.into_model()),
            height: crate::to_u32(self.height, "height")?,
            version,
            merkle_root,
            tx,
            time: crate::to_u32(self.time, "time")?,
            median_time,
            nonce: crate::to_u32(self.nonce, "nonce")?,
            bits,
            target,
            difficulty: self.difficulty,
            chain_work,
            n_tx: crate::to_u32(self.n_tx, "n_tx")?,
            previous_block_hash,
            next_block_hash,
        })
    }
}

impl GetBlockVerboseTwo {
    /// Converts version specific type to a version nonspecific, more strongly typed type.
    pub fn into_model(
        self,
    ) -> Result<model::GetBlockVerboseTwo, crate::v29::GetBlockVerboseTwoError> {
        use crate::v29::GetBlockVerboseTwoError as E;

        let hash = self.hash.parse::<BlockHash>().map_err(E::Hash)?;
        let stripped_size =
            self.stripped_size.map(|size| crate::to_u32(size, "stripped_size")).transpose()?;
        let weight = Weight::from_wu(self.weight);
        let version = block::Version::from_consensus(self.version);
        let merkle_root = self.merkle_root.parse::<TxMerkleNode>().map_err(E::MerkleRoot)?;
        let tx = self
            .tx
            .into_iter()
            .map(|entry| {
                let transaction = entry.transaction.into_model().map_err(E::Transaction)?;
                let fee = entry.fee.map(Amount::from_btc).transpose().map_err(E::Fee)?;
                Ok(model::GetBlockVerboseTwoTransaction { transaction, fee })
            })
            .collect::<Result<Vec<_>, E>>()?;
        let median_time = self.median_time.map(|t| crate::to_u32(t, "median_time")).transpose()?;
        let bits = CompactTarget::from_unprefixed_hex(&self.bits).map_err(E::Bits)?;
        let target = Some(Target::from_unprefixed_hex(self.target.as_ref()).map_err(E::Target)?);
        let chain_work = Work::from_unprefixed_hex(&self.chain_work).map_err(E::ChainWork)?;
        let previous_block_hash = self
            .previous_block_hash
            .map(|s| s.parse::<BlockHash>())
            .transpose()
            .map_err(E::PreviousBlockHash)?;
        let next_block_hash = self
            .next_block_hash
            .map(|s| s.parse::<BlockHash>())
            .transpose()
            .map_err(E::NextBlockHash)?;

        Ok(model::GetBlockVerboseTwo {
            hash,
            confirmations: self.confirmations,
            size: crate::to_u32(self.size, "size")?,
            stripped_size,
            weight,
            coinbase_tx: Some(self.coinbase_tx.into_model()),
            height: crate::to_u32(self.height, "height")?,
            version,
            merkle_root,
            tx,
            time: crate::to_u32(self.time, "time")?,
            median_time,
            nonce: crate::to_u32(self.nonce, "nonce")?,
            bits,
            target,
            difficulty: self.difficulty,
            chain_work,
            n_tx: crate::to_u32(self.n_tx, "n_tx")?,
            previous_block_hash,
            next_block_hash,
        })
    }
}

impl GetBlockVerboseThree {
    /// Converts version specific type to a version nonspecific, more strongly typed type.
    pub fn into_model(
        self,
    ) -> Result<model::GetBlockVerboseThree, crate::v29::GetBlockVerboseThreeError> {
        let coinbase_tx = Some(self.coinbase_tx.into_model());
        let inner = crate::v29::GetBlockVerboseThree {
            hash: self.hash,
            confirmations: self.confirmations,
            size: self.size,
            stripped_size: self.stripped_size,
            weight: self.weight,
            height: self.height,
            version: self.version,
            version_hex: self.version_hex,
            merkle_root: self.merkle_root,
            tx: self.tx,
            time: self.time,
            median_time: self.median_time,
            nonce: self.nonce,
            bits: self.bits,
            target: self.target,
            difficulty: self.difficulty,
            chain_work: self.chain_work,
            n_tx: self.n_tx,
            previous_block_hash: self.previous_block_hash,
            next_block_hash: self.next_block_hash,
        };
        let mut model = inner.into_model()?;
        model.coinbase_tx = coinbase_tx;
        Ok(model)
    }
}
