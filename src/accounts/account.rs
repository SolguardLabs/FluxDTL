use crate::amount::Amount;
use crate::error::FluxResult;
use crate::ids::{AccountId, AssetId};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Account {
    pub id: AccountId,
    pub label: String,
    balances: BTreeMap<AssetId, Amount>,
}

impl Account {
    pub fn new(id: AccountId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            balances: BTreeMap::new(),
        }
    }

    pub fn credit(&mut self, asset: AssetId, amount: Amount) -> FluxResult<()> {
        let current = self.balance(asset);
        self.balances.insert(asset, current.checked_add(amount)?);
        Ok(())
    }

    pub fn debit(&mut self, asset: AssetId, amount: Amount) -> FluxResult<()> {
        let current = self.balance(asset);
        self.balances.insert(asset, current.checked_sub(amount)?);
        Ok(())
    }

    pub fn balance(&self, asset: AssetId) -> Amount {
        self.balances.get(&asset).copied().unwrap_or_default()
    }

    pub fn tracked_assets(&self) -> usize {
        self.balances.len()
    }
}
