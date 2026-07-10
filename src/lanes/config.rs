use crate::amount::{Amount, BasisPoints};
use crate::ids::{AccountId, AssetId, LaneId, VaultId};

#[derive(Copy, Clone, Debug)]
pub struct LanePolicy {
    pub fee_bps: BasisPoints,
    pub maker_rebate_bps: BasisPoints,
    pub min_confidence_bps: BasisPoints,
    pub max_epoch_notional: Amount,
    pub max_residual: Amount,
    pub utilization_cap_bps: BasisPoints,
}

#[derive(Clone, Debug)]
pub struct LaneConfig {
    pub id: LaneId,
    pub name: String,
    pub source_asset: AssetId,
    pub target_asset: AssetId,
    pub source_vault: VaultId,
    pub target_vault: VaultId,
    pub operator: AccountId,
    pub policy: LanePolicy,
    pub enabled: bool,
}
