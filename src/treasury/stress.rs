use crate::amount::{Amount, BasisPoints, BPS_DENOMINATOR};
use crate::ids::AssetId;
use std::collections::BTreeSet;
use std::fmt;

const PRICE_SCALE: u128 = 100_000_000;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AssetExposure {
    pub asset: AssetId,
    pub reserve: Amount,
    pub committed: Amount,
    pub expected_inflow: Amount,
    pub price_e8: u128,
    pub confidence_bps: BasisPoints,
    pub haircut_bps: BasisPoints,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct StressPolicy {
    pub min_confidence_bps: BasisPoints,
    pub inflow_recovery_bps: BasisPoints,
    pub watch_coverage_bps: BasisPoints,
    pub halt_coverage_bps: BasisPoints,
    pub max_concentration_bps: BasisPoints,
}

impl Default for StressPolicy {
    fn default() -> Self {
        Self {
            min_confidence_bps: 9_700,
            inflow_recovery_bps: 7_500,
            watch_coverage_bps: 11_500,
            halt_coverage_bps: 9_500,
            max_concentration_bps: 6_500,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StressBand {
    Healthy,
    Watch,
    Constrained,
    Halted,
}

impl fmt::Display for StressBand {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Healthy => "healthy",
            Self::Watch => "watch",
            Self::Constrained => "constrained",
            Self::Halted => "halted",
        };
        formatter.write_str(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StressError {
    EmptyPortfolio,
    DuplicateAsset(AssetId),
    InvalidBasisPoints(&'static str),
    MissingPrice(AssetId),
    LowConfidence(AssetId),
    ArithmeticOverflow,
}

impl fmt::Display for StressError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPortfolio => formatter.write_str("portfolio has no asset exposures"),
            Self::DuplicateAsset(asset) => write!(formatter, "duplicate asset exposure: {asset}"),
            Self::InvalidBasisPoints(field) => write!(formatter, "invalid basis points: {field}"),
            Self::MissingPrice(asset) => write!(formatter, "missing price for asset: {asset}"),
            Self::LowConfidence(asset) => write!(formatter, "price confidence too low: {asset}"),
            Self::ArithmeticOverflow => formatter.write_str("treasury stress arithmetic overflow"),
        }
    }
}

impl std::error::Error for StressError {}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AssetStress {
    pub asset: AssetId,
    pub reserve_value_e8: u128,
    pub stressed_reserve_value_e8: u128,
    pub committed_value_e8: u128,
    pub recoverable_inflow_value_e8: u128,
    pub net_liquidity_value_e8: i128,
    pub utilization_bps: u16,
    pub concentration_bps: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortfolioStress {
    pub band: StressBand,
    pub assets: Vec<AssetStress>,
    pub gross_reserve_value_e8: u128,
    pub stressed_resources_value_e8: u128,
    pub committed_value_e8: u128,
    pub liquidity_gap_value_e8: u128,
    pub coverage_bps: u16,
    pub largest_concentration_bps: u16,
    pub low_confidence_assets: usize,
}

impl PortfolioStress {
    pub fn accepts_new_orders(&self) -> bool {
        matches!(self.band, StressBand::Healthy | StressBand::Watch)
    }

    pub fn requires_operator_review(&self) -> bool {
        !matches!(self.band, StressBand::Healthy)
    }
}

pub struct TreasuryStressEngine;

impl TreasuryStressEngine {
    pub fn assess(
        policy: StressPolicy,
        exposures: &[AssetExposure],
    ) -> Result<PortfolioStress, StressError> {
        Self::validate_policy(policy)?;
        if exposures.is_empty() {
            return Err(StressError::EmptyPortfolio);
        }

        let mut seen = BTreeSet::new();
        let mut low_confidence_assets = 0usize;
        let mut gross_reserve_value_e8 = 0u128;
        let mut stressed_resources_value_e8 = 0u128;
        let mut committed_value_e8 = 0u128;
        let mut provisional = Vec::with_capacity(exposures.len());

        for exposure in exposures {
            if !seen.insert(exposure.asset) {
                return Err(StressError::DuplicateAsset(exposure.asset));
            }
            if exposure.price_e8 == 0 {
                return Err(StressError::MissingPrice(exposure.asset));
            }
            if exposure.haircut_bps > 10_000 || exposure.confidence_bps > 10_000 {
                return Err(StressError::InvalidBasisPoints("asset exposure"));
            }
            if exposure.confidence_bps < policy.min_confidence_bps {
                low_confidence_assets += 1;
            }

            let reserve_value = Self::value(exposure.reserve, exposure.price_e8)?;
            let committed_value = Self::value(exposure.committed, exposure.price_e8)?;
            let inflow_value = Self::value(exposure.expected_inflow, exposure.price_e8)?;
            let stressed_reserve = Self::mul_bps(
                reserve_value,
                10_000u16.saturating_sub(exposure.haircut_bps),
            )?;
            let recoverable_inflow = Self::mul_bps(inflow_value, policy.inflow_recovery_bps)?;
            let resources = stressed_reserve
                .checked_add(recoverable_inflow)
                .ok_or(StressError::ArithmeticOverflow)?;

            gross_reserve_value_e8 = gross_reserve_value_e8
                .checked_add(reserve_value)
                .ok_or(StressError::ArithmeticOverflow)?;
            stressed_resources_value_e8 = stressed_resources_value_e8
                .checked_add(resources)
                .ok_or(StressError::ArithmeticOverflow)?;
            committed_value_e8 = committed_value_e8
                .checked_add(committed_value)
                .ok_or(StressError::ArithmeticOverflow)?;

            provisional.push((
                *exposure,
                reserve_value,
                stressed_reserve,
                committed_value,
                recoverable_inflow,
                resources,
            ));
        }

        let mut largest_concentration_bps = 0u16;
        let mut assets = Vec::with_capacity(provisional.len());
        for (exposure, reserve_value, stressed_reserve, committed_value, inflow, resources) in
            provisional
        {
            let concentration_bps = Self::ratio_bps(stressed_reserve, stressed_resources_value_e8);
            largest_concentration_bps = largest_concentration_bps.max(concentration_bps);
            assets.push(AssetStress {
                asset: exposure.asset,
                reserve_value_e8: reserve_value,
                stressed_reserve_value_e8: stressed_reserve,
                committed_value_e8: committed_value,
                recoverable_inflow_value_e8: inflow,
                net_liquidity_value_e8: Self::signed_difference(resources, committed_value),
                utilization_bps: Self::ratio_bps(committed_value, resources),
                concentration_bps,
            });
        }

        let coverage_bps = if committed_value_e8 == 0 {
            10_000
        } else {
            Self::ratio_bps(stressed_resources_value_e8, committed_value_e8)
        };
        let liquidity_gap_value_e8 = committed_value_e8.saturating_sub(stressed_resources_value_e8);
        let concentration_breach = largest_concentration_bps > policy.max_concentration_bps;
        let band = Self::classify(
            policy,
            coverage_bps,
            low_confidence_assets,
            concentration_breach,
        );

        Ok(PortfolioStress {
            band,
            assets,
            gross_reserve_value_e8,
            stressed_resources_value_e8,
            committed_value_e8,
            liquidity_gap_value_e8,
            coverage_bps,
            largest_concentration_bps,
            low_confidence_assets,
        })
    }

    fn validate_policy(policy: StressPolicy) -> Result<(), StressError> {
        let values = [
            ("min_confidence_bps", policy.min_confidence_bps),
            ("inflow_recovery_bps", policy.inflow_recovery_bps),
            ("watch_coverage_bps", policy.watch_coverage_bps),
            ("halt_coverage_bps", policy.halt_coverage_bps),
            ("max_concentration_bps", policy.max_concentration_bps),
        ];
        for (field, value) in values {
            if value > 10_000 && field != "watch_coverage_bps" {
                return Err(StressError::InvalidBasisPoints(field));
            }
        }
        if policy.halt_coverage_bps > policy.watch_coverage_bps {
            return Err(StressError::InvalidBasisPoints("coverage thresholds"));
        }
        Ok(())
    }

    fn value(amount: Amount, price_e8: u128) -> Result<u128, StressError> {
        amount
            .raw()
            .checked_mul(price_e8)
            .and_then(|value| value.checked_div(PRICE_SCALE))
            .ok_or(StressError::ArithmeticOverflow)
    }

    fn mul_bps(value: u128, bps: BasisPoints) -> Result<u128, StressError> {
        value
            .checked_mul(u128::from(bps))
            .and_then(|scaled| scaled.checked_div(BPS_DENOMINATOR))
            .ok_or(StressError::ArithmeticOverflow)
    }

    fn ratio_bps(numerator: u128, denominator: u128) -> u16 {
        if denominator == 0 {
            return if numerator == 0 { 0 } else { u16::MAX };
        }
        numerator
            .saturating_mul(BPS_DENOMINATOR)
            .checked_div(denominator)
            .unwrap_or(u128::from(u16::MAX))
            .min(u128::from(u16::MAX)) as u16
    }

    fn signed_difference(left: u128, right: u128) -> i128 {
        if left >= right {
            left.saturating_sub(right).min(i128::MAX as u128) as i128
        } else {
            -(right.saturating_sub(left).min(i128::MAX as u128) as i128)
        }
    }

    fn classify(
        policy: StressPolicy,
        coverage_bps: u16,
        low_confidence_assets: usize,
        concentration_breach: bool,
    ) -> StressBand {
        if coverage_bps < policy.halt_coverage_bps {
            StressBand::Halted
        } else if coverage_bps < 10_000 || low_confidence_assets > 0 {
            StressBand::Constrained
        } else if coverage_bps < policy.watch_coverage_bps || concentration_breach {
            StressBand::Watch
        } else {
            StressBand::Healthy
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exposure(asset: u64, reserve: u128, committed: u128) -> AssetExposure {
        AssetExposure {
            asset: AssetId::new(asset),
            reserve: Amount::new(reserve),
            committed: Amount::new(committed),
            expected_inflow: Amount::new(0),
            price_e8: PRICE_SCALE,
            confidence_bps: 9_950,
            haircut_bps: 500,
        }
    }

    #[test]
    fn healthy_portfolio_keeps_capacity_open() {
        let report = TreasuryStressEngine::assess(
            StressPolicy::default(),
            &[exposure(1, 150_000, 75_000), exposure(2, 150_000, 75_000)],
        )
        .unwrap();
        assert_eq!(report.band, StressBand::Healthy);
        assert!(report.accepts_new_orders());
        assert!(!report.requires_operator_review());
    }

    #[test]
    fn haircut_can_move_portfolio_to_watch() {
        let report = TreasuryStressEngine::assess(
            StressPolicy::default(),
            &[exposure(1, 120_000, 100_000), exposure(2, 60_000, 50_000)],
        )
        .unwrap();
        assert_eq!(report.band, StressBand::Watch);
        assert!(report.accepts_new_orders());
    }

    #[test]
    fn insufficient_coverage_halts_new_flow() {
        let report =
            TreasuryStressEngine::assess(StressPolicy::default(), &[exposure(1, 80_000, 100_000)])
                .unwrap();
        assert_eq!(report.band, StressBand::Halted);
        assert_eq!(report.liquidity_gap_value_e8, 24_000);
        assert!(!report.accepts_new_orders());
    }

    #[test]
    fn low_confidence_constrains_even_with_surplus() {
        let mut item = exposure(1, 300_000, 100_000);
        item.confidence_bps = 9_500;
        let report = TreasuryStressEngine::assess(StressPolicy::default(), &[item]).unwrap();
        assert_eq!(report.band, StressBand::Constrained);
        assert_eq!(report.low_confidence_assets, 1);
    }

    #[test]
    fn recoverable_inflow_contributes_conservatively() {
        let mut item = exposure(1, 100_000, 120_000);
        item.expected_inflow = Amount::new(40_000);
        item.haircut_bps = 1_000;
        let report = TreasuryStressEngine::assess(StressPolicy::default(), &[item]).unwrap();
        assert_eq!(report.stressed_resources_value_e8, 120_000);
        assert_eq!(report.coverage_bps, 10_000);
        assert_eq!(report.band, StressBand::Watch);
    }

    #[test]
    fn concentration_is_reported_per_asset() {
        let report = TreasuryStressEngine::assess(
            StressPolicy::default(),
            &[exposure(1, 180_000, 50_000), exposure(2, 20_000, 5_000)],
        )
        .unwrap();
        assert_eq!(report.largest_concentration_bps, 9_000);
        assert_eq!(report.assets[0].concentration_bps, 9_000);
        assert_eq!(report.band, StressBand::Watch);
    }

    #[test]
    fn duplicate_assets_are_rejected() {
        let result = TreasuryStressEngine::assess(
            StressPolicy::default(),
            &[exposure(7, 100, 10), exposure(7, 100, 10)],
        );
        assert_eq!(result, Err(StressError::DuplicateAsset(AssetId::new(7))));
    }

    #[test]
    fn zero_price_is_rejected() {
        let mut item = exposure(8, 100, 10);
        item.price_e8 = 0;
        assert_eq!(
            TreasuryStressEngine::assess(StressPolicy::default(), &[item]),
            Err(StressError::MissingPrice(AssetId::new(8)))
        );
    }

    #[test]
    fn empty_portfolio_is_rejected() {
        assert_eq!(
            TreasuryStressEngine::assess(StressPolicy::default(), &[]),
            Err(StressError::EmptyPortfolio)
        );
    }

    #[test]
    fn malformed_policy_is_rejected() {
        let policy = StressPolicy {
            inflow_recovery_bps: 10_001,
            ..StressPolicy::default()
        };
        assert_eq!(
            TreasuryStressEngine::assess(policy, &[exposure(1, 100, 10)]),
            Err(StressError::InvalidBasisPoints("inflow_recovery_bps"))
        );
    }

    #[test]
    fn arithmetic_overflow_fails_closed() {
        let mut item = exposure(1, u128::MAX, 1);
        item.price_e8 = u128::MAX;
        assert_eq!(
            TreasuryStressEngine::assess(StressPolicy::default(), &[item]),
            Err(StressError::ArithmeticOverflow)
        );
    }
}
