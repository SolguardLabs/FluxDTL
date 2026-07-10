use crate::accounts::Account;
use crate::amount::Amount;
use crate::asset::{Asset, AssetRegistry};
use crate::epochs::{EpochBook, EpochWindow};
use crate::error::{FluxError, FluxResult};
use crate::events::Event;
use crate::ids::{AccountId, AssetId, EpochId, LaneId, OrderId, TxId, VaultId};
use crate::lanes::{LaneConfig, LanePolicy};
use crate::oracle::{Oracle, PricePoint};
use crate::orders::{OrderStatus, TransferOrder};
use crate::risk::RiskGuard;
use crate::settlement::SettlementEngine;
use crate::vault::{Vault, VaultMode};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct DemoReport {
    pub accounts: usize,
    pub assets: usize,
    pub vaults: usize,
    pub lanes: usize,
    pub epochs: usize,
    pub events: usize,
    pub settled_volume: Amount,
    pub liquidity_credits: Amount,
}

impl DemoReport {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"accounts\":{},\"assets\":{},\"vaults\":{},\"lanes\":{},\"epochs\":{},\"events\":{},\"settled_volume\":{},\"liquidity_credits\":{}}}",
            self.accounts,
            self.assets,
            self.vaults,
            self.lanes,
            self.epochs,
            self.events,
            self.settled_volume.raw(),
            self.liquidity_credits.raw()
        )
    }
}

#[derive(Clone, Debug)]
pub struct Ledger {
    assets: AssetRegistry,
    accounts: BTreeMap<AccountId, Account>,
    vaults: BTreeMap<VaultId, Vault>,
    oracle: Oracle,
    lanes: BTreeMap<LaneId, LaneConfig>,
    orders: BTreeMap<OrderId, TransferOrder>,
    epochs: EpochBook,
    risk: RiskGuard,
    liquidity_credits: BTreeMap<AccountId, Amount>,
    events: Vec<Event>,
    slot: u64,
    next_account: u64,
    next_asset: u64,
    next_vault: u64,
    next_lane: u64,
    next_order: u64,
    next_epoch: u64,
    next_tx: u64,
}

impl Ledger {
    pub fn new() -> Self {
        Self {
            assets: AssetRegistry::default(),
            accounts: BTreeMap::new(),
            vaults: BTreeMap::new(),
            oracle: Oracle::default(),
            lanes: BTreeMap::new(),
            orders: BTreeMap::new(),
            epochs: EpochBook::default(),
            risk: RiskGuard::default(),
            liquidity_credits: BTreeMap::new(),
            events: Vec::new(),
            slot: 1,
            next_account: 1,
            next_asset: 1,
            next_vault: 1,
            next_lane: 1,
            next_order: 1,
            next_epoch: 1,
            next_tx: 1,
        }
    }

    pub fn create_account(&mut self, label: impl Into<String>) -> AccountId {
        let id = AccountId::new(self.next_account);
        self.next_account += 1;
        self.accounts.insert(id, Account::new(id, label));
        id
    }

    pub fn register_asset(
        &mut self,
        symbol: impl Into<String>,
        decimals: u8,
        risk_weight_bps: u16,
    ) -> FluxResult<AssetId> {
        let id = AssetId::new(self.next_asset);
        self.next_asset += 1;
        let symbol = symbol.into();
        self.assets.insert(Asset {
            id,
            symbol: symbol.clone(),
            decimals,
            risk_weight_bps,
            enabled: true,
        })?;
        self.events
            .push(Event::AssetRegistered { asset: id, symbol });
        Ok(id)
    }

    pub fn set_price(
        &mut self,
        asset: AssetId,
        price_e8: u128,
        confidence_bps: u16,
    ) -> FluxResult<()> {
        self.assets.get(asset)?;
        self.oracle.set_price(PricePoint {
            asset,
            price_e8,
            confidence_bps,
            slot: self.slot,
        });
        Ok(())
    }

    pub fn create_vault(
        &mut self,
        asset: AssetId,
        controller: AccountId,
        mode: VaultMode,
    ) -> FluxResult<VaultId> {
        self.assets.get(asset)?;
        self.account(controller)?;
        let id = VaultId::new(self.next_vault);
        self.next_vault += 1;
        self.vaults
            .insert(id, Vault::new(id, asset, controller, mode));
        self.events.push(Event::VaultCreated { vault: id, asset });
        Ok(id)
    }

    pub fn deposit_vault(&mut self, vault: VaultId, amount: Amount) -> FluxResult<()> {
        self.vault_mut(vault)?.deposit(amount)
    }

    pub fn create_lane(&mut self, mut lane: LaneConfig) -> FluxResult<LaneId> {
        self.assets.get(lane.source_asset)?;
        self.assets.get(lane.target_asset)?;
        let source_vault = self.vault(lane.source_vault)?;
        let target_vault = self.vault(lane.target_vault)?;
        if source_vault.asset != lane.source_asset || target_vault.asset != lane.target_asset {
            return Err(FluxError::InvalidVaultAsset);
        }
        self.account(lane.operator)?;

        let id = LaneId::new(self.next_lane);
        self.next_lane += 1;
        lane.id = id;
        self.events.push(Event::LaneCreated {
            lane: id,
            source: lane.source_asset,
            target: lane.target_asset,
        });
        self.lanes.insert(id, lane);
        Ok(id)
    }

    pub fn open_epoch(&mut self) -> EpochId {
        let id = EpochId::new(self.next_epoch);
        self.next_epoch += 1;
        self.epochs.insert(EpochWindow::new(id, self.slot));
        self.events.push(Event::EpochOpened { epoch: id });
        id
    }

    #[allow(clippy::too_many_arguments)]
    pub fn submit_order(
        &mut self,
        owner: AccountId,
        recipient: AccountId,
        epoch: EpochId,
        lane_id: LaneId,
        amount_in: Amount,
        min_out: Amount,
        nonce: u64,
    ) -> FluxResult<OrderId> {
        self.account(owner)?;
        self.account(recipient)?;
        let lane = self.lane(lane_id)?.clone();
        if !lane.enabled {
            return Err(FluxError::LaneDisabled);
        }

        let quote = SettlementEngine::quote(&self.assets, &self.oracle, &lane, amount_in)?;
        if quote.recipient_out < min_out {
            return Err(FluxError::MinimumOutput);
        }

        self.vault_mut(lane.source_vault)?.lock(amount_in)?;

        let id = OrderId::new(self.next_order);
        self.next_order += 1;
        self.epochs
            .get_mut(epoch)?
            .queue_order(lane_id, id, amount_in)?;
        self.orders.insert(
            id,
            TransferOrder {
                id,
                owner,
                recipient,
                lane: lane_id,
                epoch,
                amount_in,
                min_out,
                created_slot: self.slot,
                nonce,
                status: OrderStatus::Pending,
            },
        );
        self.events.push(Event::OrderQueued {
            order: id,
            lane: lane_id,
            amount_in,
        });
        Ok(id)
    }

    pub fn settle_order(&mut self, order_id: OrderId) -> FluxResult<TxId> {
        let order = self.order(order_id)?.clone();
        if order.status != OrderStatus::Pending {
            return Err(FluxError::OrderNotPending);
        }

        let lane = self.lane(order.lane)?.clone();
        let quote = SettlementEngine::quote(&self.assets, &self.oracle, &lane, order.amount_in)?;
        if quote.recipient_out < order.min_out {
            return Err(FluxError::MinimumOutput);
        }

        let target_snapshot = self.vault(lane.target_vault)?.clone();
        let residual = self.risk.authorize(
            order.epoch,
            &lane,
            &target_snapshot,
            quote.gross_out,
            quote.netted_input_value,
        )?;

        self.vault_mut(lane.source_vault)?
            .consume_locked(order.amount_in)?;
        self.vault_mut(lane.target_vault)?.pay(quote.gross_out)?;
        self.account_mut(order.recipient)?
            .credit(lane.target_asset, quote.recipient_out)?;
        self.account_mut(lane.operator)?
            .credit(lane.target_asset, quote.protocol_fee)?;

        let current_credit = self
            .liquidity_credits
            .get(&order.owner)
            .copied()
            .unwrap_or_default();
        self.liquidity_credits
            .insert(order.owner, current_credit.checked_add(quote.maker_rebate)?);

        self.epochs.get_mut(order.epoch)?.record_settlement(
            lane.id,
            quote.gross_out,
            quote.protocol_fee,
            quote.maker_rebate,
            residual,
        )?;

        let tx = TxId::new(self.next_tx);
        self.next_tx += 1;
        if let Some(order) = self.orders.get_mut(&order_id) {
            order.status = OrderStatus::Settled;
        }
        self.events.push(Event::OrderSettled {
            tx,
            order: order_id,
            gross_out: quote.gross_out,
            recipient_out: quote.recipient_out,
            fee: quote.protocol_fee,
            maker_rebate: quote.maker_rebate,
        });
        self.slot += 1;
        Ok(tx)
    }

    pub fn withdraw_liquidity_credit(
        &mut self,
        account: AccountId,
        vault: VaultId,
        asset: AssetId,
        amount: Amount,
    ) -> FluxResult<TxId> {
        self.account(account)?;
        let vault_asset = self.vault(vault)?.asset;
        if vault_asset != asset {
            return Err(FluxError::InvalidVaultAsset);
        }

        let current = self
            .liquidity_credits
            .get(&account)
            .copied()
            .unwrap_or_default();
        if current < amount {
            return Err(FluxError::LiquidityCreditExceeded);
        }

        self.liquidity_credits
            .insert(account, current.checked_sub(amount)?);
        self.vault_mut(vault)?.pay(amount)?;
        self.account_mut(account)?.credit(asset, amount)?;

        let tx = TxId::new(self.next_tx);
        self.next_tx += 1;
        self.events.push(Event::LiquidityCreditWithdrawn {
            account,
            asset,
            vault,
            amount,
        });
        Ok(tx)
    }

    pub fn close_epoch(&mut self, epoch: EpochId) -> FluxResult<()> {
        self.epochs.get_mut(epoch)?.close();
        self.events.push(Event::EpochClosed { epoch });
        Ok(())
    }

    pub fn demo() -> FluxResult<DemoReport> {
        let mut ledger = Self::new();
        let operator = ledger.create_account("flux-operator");
        let maker = ledger.create_account("regional-maker");
        let recipient = ledger.create_account("settlement-recipient");

        let usdc = ledger.register_asset("fUSDC", 6, 1_000)?;
        let eur = ledger.register_asset("fEUR", 6, 1_050)?;
        let mxn = ledger.register_asset("fMXN", 6, 1_450)?;

        ledger.set_price(usdc, 100_000_000, 9_980)?;
        ledger.set_price(eur, 108_000_000, 9_970)?;
        ledger.set_price(mxn, 5_800_000, 9_940)?;

        let usdc_funding = ledger.create_vault(usdc, operator, VaultMode::Funding)?;
        let eur_settlement = ledger.create_vault(eur, operator, VaultMode::Settlement)?;
        let mxn_settlement = ledger.create_vault(mxn, operator, VaultMode::Settlement)?;

        ledger.deposit_vault(usdc_funding, Amount::new(900_000_000_000))?;
        ledger.deposit_vault(eur_settlement, Amount::new(620_000_000_000))?;
        ledger.deposit_vault(mxn_settlement, Amount::new(9_000_000_000_000))?;

        let usdc_eur = ledger.create_lane(LaneConfig {
            id: LaneId::new(0),
            name: "usdc-eur-prime".to_string(),
            source_asset: usdc,
            target_asset: eur,
            source_vault: usdc_funding,
            target_vault: eur_settlement,
            operator,
            policy: LanePolicy {
                fee_bps: 16,
                maker_rebate_bps: 4,
                min_confidence_bps: 9_850,
                max_epoch_notional: Amount::new(80_000_000_000),
                max_residual: Amount::new(8_000_000_000),
                utilization_cap_bps: 7_500,
            },
            enabled: true,
        })?;

        let usdc_mxn = ledger.create_lane(LaneConfig {
            id: LaneId::new(0),
            name: "usdc-mxn-latam".to_string(),
            source_asset: usdc,
            target_asset: mxn,
            source_vault: usdc_funding,
            target_vault: mxn_settlement,
            operator,
            policy: LanePolicy {
                fee_bps: 24,
                maker_rebate_bps: 6,
                min_confidence_bps: 9_800,
                max_epoch_notional: Amount::new(900_000_000_000),
                max_residual: Amount::new(20_000_000_000),
                utilization_cap_bps: 8_000,
            },
            enabled: true,
        })?;

        let epoch = ledger.open_epoch();
        let order_a = ledger.submit_order(
            maker,
            recipient,
            epoch,
            usdc_eur,
            Amount::new(14_000_000_000),
            Amount::new(12_900_000_000),
            1,
        )?;
        let order_b = ledger.submit_order(
            maker,
            recipient,
            epoch,
            usdc_mxn,
            Amount::new(18_000_000_000),
            Amount::new(300_000_000_000),
            2,
        )?;
        ledger.settle_order(order_a)?;
        ledger.settle_order(order_b)?;
        ledger.withdraw_liquidity_credit(maker, eur_settlement, eur, Amount::new(45_000_000))?;
        ledger.close_epoch(epoch)?;

        let settled_volume = ledger
            .vaults
            .values()
            .try_fold(Amount::zero(), |total, vault| total.checked_add(vault.paid))?;
        let liquidity_credits = ledger
            .liquidity_credits
            .values()
            .try_fold(Amount::zero(), |total, credit| total.checked_add(*credit))?;

        Ok(DemoReport {
            accounts: ledger.accounts.len(),
            assets: ledger.assets.len(),
            vaults: ledger.vaults.len(),
            lanes: ledger.lanes.len(),
            epochs: ledger.epochs.len(),
            events: ledger.events.len(),
            settled_volume,
            liquidity_credits,
        })
    }

    fn account(&self, id: AccountId) -> FluxResult<&Account> {
        self.accounts.get(&id).ok_or(FluxError::AccountNotFound)
    }

    fn account_mut(&mut self, id: AccountId) -> FluxResult<&mut Account> {
        self.accounts.get_mut(&id).ok_or(FluxError::AccountNotFound)
    }

    fn vault(&self, id: VaultId) -> FluxResult<&Vault> {
        self.vaults.get(&id).ok_or(FluxError::VaultNotFound)
    }

    fn vault_mut(&mut self, id: VaultId) -> FluxResult<&mut Vault> {
        self.vaults.get_mut(&id).ok_or(FluxError::VaultNotFound)
    }

    fn lane(&self, id: LaneId) -> FluxResult<&LaneConfig> {
        self.lanes.get(&id).ok_or(FluxError::LaneNotFound)
    }

    fn order(&self, id: OrderId) -> FluxResult<&TransferOrder> {
        self.orders.get(&id).ok_or(FluxError::OrderNotFound)
    }
}
