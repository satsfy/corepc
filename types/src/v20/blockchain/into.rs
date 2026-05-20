// SPDX-License-Identifier: CC0-1.0

use bitcoin::{Amount, BlockHash, FeeRate, Weight};

use super::{GetBlockStats, GetBlockStatsError};
use crate::model;

impl GetBlockStats {
    /// Converts version specific type to a version nonspecific, more strongly typed type.
    pub fn into_model(self) -> Result<model::GetBlockStats, GetBlockStatsError> {
        use GetBlockStatsError as E;

        // `FeeRate::sat_per_vb` returns an option if value overflows.
        let average_fee_rate = self.average_fee_rate.and_then(FeeRate::from_sat_per_vb);
        let block_hash =
            self.block_hash.map(|h| h.parse::<BlockHash>()).transpose().map_err(E::BlockHash)?;
        let fee_rate_percentiles = self
            .fee_rate_percentiles
            .map(|arr| arr.iter().map(|vb| FeeRate::from_sat_per_vb(*vb)).collect());
        let max_fee_rate = self.max_fee_rate.and_then(FeeRate::from_sat_per_vb);
        let minimum_fee_rate = self.minimum_fee_rate.and_then(FeeRate::from_sat_per_vb);

        // FIXME: Double check that these values are virtual bytes and not weight units.
        let segwit_total_weight = self.segwit_total_weight.and_then(Weight::from_vb);
        let total_weight = self.total_weight.and_then(Weight::from_vb);

        Ok(model::GetBlockStats {
            average_fee: self.average_fee.map(Amount::from_sat),
            average_fee_rate,
            average_tx_size: self
                .average_tx_size
                .map(|v| crate::to_u32(v, "average_tx_size"))
                .transpose()?,
            block_hash,
            fee_rate_percentiles,
            height: self.height.map(|v| crate::to_u32(v, "height")).transpose()?,
            inputs: self.inputs.map(|v| crate::to_u32(v, "inputs")).transpose()?,
            max_fee: self.max_fee.map(Amount::from_sat),
            max_fee_rate,
            max_tx_size: self.max_tx_size.map(|v| crate::to_u32(v, "max_tx_size")).transpose()?,
            median_fee: self.median_fee.map(Amount::from_sat),
            median_time: self.median_time.map(|v| crate::to_u32(v, "median_time")).transpose()?,
            median_tx_size: self
                .median_tx_size
                .map(|v| crate::to_u32(v, "median_tx_size"))
                .transpose()?,
            minimum_fee: self.minimum_fee.map(Amount::from_sat),
            minimum_fee_rate,
            minimum_tx_size: self
                .minimum_tx_size
                .map(|v| crate::to_u32(v, "minimum_tx_size"))
                .transpose()?,
            outputs: self.outputs.map(|v| crate::to_u32(v, "outputs")).transpose()?,
            subsidy: self.subsidy.map(Amount::from_sat),
            segwit_total_size: self
                .segwit_total_size
                .map(|v| crate::to_u32(v, "segwit_total_size"))
                .transpose()?,
            segwit_total_weight,
            segwit_txs: self.segwit_txs.map(|v| crate::to_u32(v, "segwit_txs")).transpose()?,
            time: self.time.map(|v| crate::to_u32(v, "time")).transpose()?,
            total_out: self.total_out.map(Amount::from_sat),
            total_size: self.total_size.map(|v| crate::to_u32(v, "total_size")).transpose()?,
            total_weight,
            total_fee: self.total_fee.map(Amount::from_sat),
            txs: self.txs.map(|v| crate::to_u32(v, "txs")).transpose()?,
            utxo_increase: self.utxo_increase,
            utxo_size_increase: self.utxo_size_increase,
            utxo_increase_actual: None,      // v25 and later only.
            utxo_size_increase_actual: None, // v25 and later only.
        })
    }
}
