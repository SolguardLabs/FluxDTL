use crate::amount::{Amount, BPS_DENOMINATOR};
use crate::error::{FluxError, FluxResult};
use crate::ids::{EpochId, LaneId};
use crate::lanes::LaneConfig;
use crate::vault::Vault;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct RiskSnapshot {
    pub lane_notional: BTreeMap<(EpochId, LaneId), Amount>,
    pub lane_residual: BTreeMap<(EpochId, LaneId), Amount>,
}

#[derive(Clone, Debug, Default)]
pub struct RiskGuard {
    pub snapshot: RiskSnapshot,
}

impl RiskGuard {
    pub fn authorize(
        &mut self,
        epoch: EpochId,
        lane: &LaneConfig,
        target_vault: &Vault,
        gross_out: Amount,
        netted_input_value: Amount,
    ) -> FluxResult<Amount> {
        let key = (epoch, lane.id);
        let residual = gross_out.saturating_sub(netted_input_value);

        let current_notional = self
            .snapshot
            .lane_notional
            .get(&key)
            .copied()
            .unwrap_or_default();
        let next_notional = current_notional.checked_add(gross_out)?;
        if next_notional > lane.policy.max_epoch_notional {
            return Err(FluxError::RiskLimitExceeded);
        }

        let current_residual = self
            .snapshot
            .lane_residual
            .get(&key)
            .copied()
            .unwrap_or_default();
        let next_residual = current_residual.checked_add(residual)?;
        if next_residual > lane.policy.max_residual {
            return Err(FluxError::RiskLimitExceeded);
        }

        let utilization = target_vault
            .paid
            .checked_add(gross_out)?
            .mul_ratio_floor(BPS_DENOMINATOR, target_vault.received.raw().max(1))?;
        if utilization.raw() > u128::from(lane.policy.utilization_cap_bps) {
            return Err(FluxError::RiskLimitExceeded);
        }

        self.snapshot.lane_notional.insert(key, next_notional);
        self.snapshot.lane_residual.insert(key, next_residual);
        Ok(residual)
    }
}
