use crate::amount::Amount;
use crate::error::{FluxError, FluxResult};
use crate::ids::{AccountId, AssetId, VaultId};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VaultMode {
    Funding,
    Settlement,
    Incentive,
}

#[derive(Clone, Debug)]
pub struct Vault {
    pub id: VaultId,
    pub asset: AssetId,
    pub controller: AccountId,
    pub mode: VaultMode,
    pub reserve: Amount,
    pub locked: Amount,
    pub paid: Amount,
    pub received: Amount,
}

impl Vault {
    pub fn new(id: VaultId, asset: AssetId, controller: AccountId, mode: VaultMode) -> Self {
        Self {
            id,
            asset,
            controller,
            mode,
            reserve: Amount::zero(),
            locked: Amount::zero(),
            paid: Amount::zero(),
            received: Amount::zero(),
        }
    }

    pub fn available(&self) -> Amount {
        self.reserve.saturating_sub(self.locked)
    }

    pub fn deposit(&mut self, amount: Amount) -> FluxResult<()> {
        self.reserve = self.reserve.checked_add(amount)?;
        self.received = self.received.checked_add(amount)?;
        Ok(())
    }

    pub fn lock(&mut self, amount: Amount) -> FluxResult<()> {
        if self.available() < amount {
            return Err(FluxError::InsufficientBalance);
        }
        self.locked = self.locked.checked_add(amount)?;
        Ok(())
    }

    pub fn consume_locked(&mut self, amount: Amount) -> FluxResult<()> {
        self.locked = self.locked.checked_sub(amount)?;
        self.reserve = self.reserve.checked_sub(amount)?;
        self.paid = self.paid.checked_add(amount)?;
        Ok(())
    }

    pub fn release_locked(&mut self, amount: Amount) -> FluxResult<()> {
        self.locked = self.locked.checked_sub(amount)?;
        Ok(())
    }

    pub fn pay(&mut self, amount: Amount) -> FluxResult<()> {
        if self.available() < amount {
            return Err(FluxError::InsufficientBalance);
        }

        self.reserve = self.reserve.checked_sub(amount)?;
        self.paid = self.paid.checked_add(amount)?;
        Ok(())
    }
}
