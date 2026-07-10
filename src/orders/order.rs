use crate::amount::Amount;
use crate::ids::{AccountId, EpochId, LaneId, OrderId};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Settled,
    Cancelled,
}

#[derive(Clone, Debug)]
pub struct TransferOrder {
    pub id: OrderId,
    pub owner: AccountId,
    pub recipient: AccountId,
    pub lane: LaneId,
    pub epoch: EpochId,
    pub amount_in: Amount,
    pub min_out: Amount,
    pub created_slot: u64,
    pub nonce: u64,
    pub status: OrderStatus,
}
