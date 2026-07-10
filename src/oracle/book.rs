use crate::amount::Amount;
use crate::asset::AssetRegistry;
use crate::error::{FluxError, FluxResult};
use crate::ids::AssetId;
use std::collections::BTreeMap;

#[derive(Copy, Clone, Debug)]
pub struct PricePoint {
    pub asset: AssetId,
    pub price_e8: u128,
    pub confidence_bps: u16,
    pub slot: u64,
}

#[derive(Clone, Debug, Default)]
pub struct Oracle {
    prices: BTreeMap<AssetId, PricePoint>,
}

impl Oracle {
    pub fn set_price(&mut self, point: PricePoint) {
        self.prices.insert(point.asset, point);
    }

    pub fn price(&self, asset: AssetId) -> FluxResult<PricePoint> {
        self.prices
            .get(&asset)
            .copied()
            .ok_or(FluxError::PriceNotAvailable)
    }

    pub fn quote(
        &self,
        assets: &AssetRegistry,
        source: AssetId,
        target: AssetId,
        amount: Amount,
    ) -> FluxResult<Amount> {
        let source_asset = assets.get(source)?;
        let target_asset = assets.get(target)?;
        let source_price = self.price(source)?;
        let target_price = self.price(target)?;
        let source_unit = pow10(source_asset.decimals)?;
        let target_unit = pow10(target_asset.decimals)?;

        amount
            .checked_mul(source_price.price_e8)?
            .checked_mul(target_unit)?
            .checked_div(source_unit)?
            .checked_div(target_price.price_e8)
    }
}

fn pow10(decimals: u8) -> FluxResult<u128> {
    let mut value = 1u128;
    for _ in 0..decimals {
        value = value.checked_mul(10).ok_or(FluxError::AmountOverflow)?;
    }
    Ok(value)
}
