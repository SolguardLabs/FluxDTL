use crate::amount::BasisPoints;
use crate::error::{FluxError, FluxResult};
use crate::ids::AssetId;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Asset {
    pub id: AssetId,
    pub symbol: String,
    pub decimals: u8,
    pub risk_weight_bps: BasisPoints,
    pub enabled: bool,
}

#[derive(Clone, Debug, Default)]
pub struct AssetRegistry {
    assets: BTreeMap<AssetId, Asset>,
}

impl AssetRegistry {
    pub fn insert(&mut self, asset: Asset) -> FluxResult<()> {
        if asset.decimals > 18 {
            return Err(FluxError::AmountOverflow);
        }

        self.assets.insert(asset.id, asset);
        Ok(())
    }

    pub fn get(&self, id: AssetId) -> FluxResult<&Asset> {
        let asset = self.assets.get(&id).ok_or(FluxError::AssetNotFound)?;
        if !asset.enabled {
            return Err(FluxError::AssetDisabled);
        }
        Ok(asset)
    }

    pub fn len(&self) -> usize {
        self.assets.len()
    }
}
