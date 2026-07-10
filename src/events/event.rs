use crate::amount::Amount;
use crate::ids::{AccountId, AssetId, EpochId, LaneId, OrderId, TxId, VaultId};

#[derive(Clone, Debug)]
pub enum Event {
    AssetRegistered {
        asset: AssetId,
        symbol: String,
    },
    VaultCreated {
        vault: VaultId,
        asset: AssetId,
    },
    LaneCreated {
        lane: LaneId,
        source: AssetId,
        target: AssetId,
    },
    EpochOpened {
        epoch: EpochId,
    },
    OrderQueued {
        order: OrderId,
        lane: LaneId,
        amount_in: Amount,
    },
    OrderSettled {
        tx: TxId,
        order: OrderId,
        gross_out: Amount,
        recipient_out: Amount,
        fee: Amount,
        maker_rebate: Amount,
    },
    LiquidityCreditWithdrawn {
        account: AccountId,
        asset: AssetId,
        vault: VaultId,
        amount: Amount,
    },
    EpochClosed {
        epoch: EpochId,
    },
}
