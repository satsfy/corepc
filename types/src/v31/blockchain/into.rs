// SPDX-License-Identifier: CC0-1.0

use core::fmt;

use bitcoin::{amount::ParseAmountError, hex, Amount, Txid};

use super::GetMempoolCluster;
use crate::model;

impl GetMempoolCluster {
    /// Converts version specific type to a version nonspecific, more strongly typed type.
    pub fn into_model(self) -> Result<model::GetMempoolCluster, GetMempoolClusterError> {
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
