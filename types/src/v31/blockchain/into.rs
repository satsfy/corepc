// SPDX-License-Identifier: CC0-1.0

use alloc::collections::BTreeMap;
use core::fmt;

use bitcoin::consensus::encode;
use bitcoin::{amount::ParseAmountError, hex, Amount, BlockHash, OutPoint, Transaction, Txid, Wtxid};

use super::{
    CoinbaseTransaction, GetBlockVerboseOne, GetBlockVerboseThree, GetBlockVerboseTwo,
    GetDeploymentInfo, GetMempoolAncestorsVerbose, GetMempoolCluster, GetMempoolDescendantsVerbose,
    GetMempoolEntry, GetMempoolFeerateDiagram, GetMempoolInfo, GetMempoolInfoError,
    GetRawMempoolVerbose, GetTxSpendingPrevout, GetTxSpendingPrevoutItem, MempoolEntry,
    MempoolEntryFees,
};
use crate::{model, NumericError};

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

/// Error when converting a `GetMempoolCluster` type into the model type.
#[derive(Debug)]
pub enum GetMempoolClusterError {
    /// Conversion of a transaction id to a `Txid` failed.
    Txid(hex::HexToArrayError),
    /// Conversion of a chunk fee to an `Amount` failed.
    ChunkFee(ParseAmountError),
}

impl fmt::Display for GetMempoolClusterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Txid(e) => write!(f, "conversion of `txid` field failed: {}", e),
            Self::ChunkFee(e) => write!(f, "conversion of `chunkfee` field failed: {}", e),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for GetMempoolClusterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Txid(e) => Some(e),
            Self::ChunkFee(e) => Some(e),
        }
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
            chunk_weight,
            time,
            height,
            descendant_count,
            descendant_size,
            ancestor_count,
            ancestor_size,
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

/// Error when converting a `MempoolEntryFees` type into the model type.
#[derive(Debug)]
pub enum MempoolEntryFeesError {
    /// Conversion of the `base` field failed.
    Base(ParseAmountError),
    /// Conversion of the `modified` field failed.
    Modified(ParseAmountError),
    /// Conversion of the `ancestor` field failed.
    Ancestor(ParseAmountError),
    /// Conversion of the `descendant` field failed.
    Descendant(ParseAmountError),
    /// Conversion of the `chunk` field failed.
    Chunk(ParseAmountError),
}

impl fmt::Display for MempoolEntryFeesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Base(e) => write!(f, "conversion of the `base` field failed: {}", e),
            Self::Modified(e) => write!(f, "conversion of the `modified` field failed: {}", e),
            Self::Ancestor(e) => write!(f, "conversion of the `ancestor` field failed: {}", e),
            Self::Descendant(e) => write!(f, "conversion of the `descendant` field failed: {}", e),
            Self::Chunk(e) => write!(f, "conversion of the `chunk` field failed: {}", e),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for MempoolEntryFeesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Base(e) => Some(e),
            Self::Modified(e) => Some(e),
            Self::Ancestor(e) => Some(e),
            Self::Descendant(e) => Some(e),
            Self::Chunk(e) => Some(e),
        }
    }
}

/// Error when converting a `MempoolEntry` type into the model type.
#[derive(Debug)]
pub enum MempoolEntryError {
    /// Conversion of a numeric type to an expected type failed.
    Numeric(NumericError),
    /// Conversion of the `wtxid` field failed.
    Wtxid(hex::HexToArrayError),
    /// Conversion of the `fees` field failed.
    Fees(MempoolEntryFeesError),
    /// Conversion of the `depends` field failed.
    Depends(hex::HexToArrayError),
    /// Conversion of the `spentby` field failed.
    SpentBy(hex::HexToArrayError),
}

impl From<NumericError> for MempoolEntryError {
    fn from(e: NumericError) -> Self { Self::Numeric(e) }
}

impl fmt::Display for MempoolEntryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Numeric(e) => write!(f, "numeric: {}", e),
            Self::Wtxid(e) => write!(f, "conversion of the `wtxid` field failed: {}", e),
            Self::Fees(e) => write!(f, "conversion of the `fees` field failed: {}", e),
            Self::Depends(e) => write!(f, "conversion of the `depends` field failed: {}", e),
            Self::SpentBy(e) => write!(f, "conversion of the `spentby` field failed: {}", e),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for MempoolEntryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Numeric(e) => Some(e),
            Self::Wtxid(e) => Some(e),
            Self::Fees(e) => Some(e),
            Self::Depends(e) => Some(e),
            Self::SpentBy(e) => Some(e),
        }
    }
}

/// Error when converting a map of `MempoolEntry`s into the model type.
#[derive(Debug)]
pub enum MapMempoolEntryError {
    /// Conversion of a `txid` failed.
    Txid(hex::HexToArrayError),
    /// Conversion of a `MempoolEntry` failed.
    MempoolEntry(MempoolEntryError),
}

impl fmt::Display for MapMempoolEntryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Txid(e) => write!(f, "conversion of a `txid` failed: {}", e),
            Self::MempoolEntry(e) => write!(f, "conversion of a `MempoolEntry` failed: {}", e),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for MapMempoolEntryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Txid(e) => Some(e),
            Self::MempoolEntry(e) => Some(e),
        }
    }
}

/// Error when converting a `GetMempoolFeerateDiagram` type into the model type.
#[derive(Debug)]
pub enum GetMempoolFeerateDiagramError {
    /// Conversion of a numeric type to an expected type failed.
    Numeric(NumericError),
    /// Conversion of a `fee` field failed.
    Fee(ParseAmountError),
}

impl From<NumericError> for GetMempoolFeerateDiagramError {
    fn from(e: NumericError) -> Self { Self::Numeric(e) }
}

impl fmt::Display for GetMempoolFeerateDiagramError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Numeric(e) => write!(f, "numeric: {}", e),
            Self::Fee(e) => write!(f, "conversion of the `fee` field failed: {}", e),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for GetMempoolFeerateDiagramError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Numeric(e) => Some(e),
            Self::Fee(e) => Some(e),
        }
    }
}

impl GetDeploymentInfo {
    /// Converts version specific type to a version nonspecific, more strongly typed type.
    pub fn into_model(
        self,
    ) -> Result<model::GetDeploymentInfo, crate::v23::GetDeploymentInfoError> {
        let inner = crate::v23::GetDeploymentInfo {
            hash: self.hash,
            height: self.height,
            deployments: self.deployments,
        };
        let mut model = inner.into_model()?;
        model.script_flags = Some(self.script_flags);
        Ok(model)
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

/// Error when converting a `GetTxSpendingPrevout` type into the model type.
#[derive(Debug)]
pub enum GetTxSpendingPrevoutError {
    /// Conversion of the `txid` field failed.
    Txid(hex::HexToArrayError),
    /// Conversion of the `spendingtxid` field failed.
    SpendingTxid(hex::HexToArrayError),
    /// Conversion of the `spendingtx` field failed.
    SpendingTx(encode::FromHexError),
    /// Conversion of the `blockhash` field failed.
    BlockHash(hex::HexToArrayError),
}

impl fmt::Display for GetTxSpendingPrevoutError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Txid(e) => write!(f, "conversion of the `txid` field failed: {}", e),
            Self::SpendingTxid(e) =>
                write!(f, "conversion of the `spendingtxid` field failed: {}", e),
            Self::SpendingTx(e) => write!(f, "conversion of the `spendingtx` field failed: {}", e),
            Self::BlockHash(e) => write!(f, "conversion of the `blockhash` field failed: {}", e),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for GetTxSpendingPrevoutError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Txid(e) => Some(e),
            Self::SpendingTxid(e) => Some(e),
            Self::SpendingTx(e) => Some(e),
            Self::BlockHash(e) => Some(e),
        }
    }
}
