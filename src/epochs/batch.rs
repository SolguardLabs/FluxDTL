use crate::amount::Amount;
use crate::error::{FluxError, FluxResult};
use crate::ids::{EpochId, LaneId, OrderId};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct LaneAccumulator {
    pub orders: Vec<OrderId>,
    pub input: Amount,
    pub output: Amount,
    pub fees: Amount,
    pub rebates: Amount,
    pub residual: Amount,
}

#[derive(Clone, Debug)]
pub struct EpochWindow {
    pub id: EpochId,
    pub opened_slot: u64,
    pub closed: bool,
    pub lanes: BTreeMap<LaneId, LaneAccumulator>,
}

impl EpochWindow {
    pub fn new(id: EpochId, opened_slot: u64) -> Self {
        Self {
            id,
            opened_slot,
            closed: false,
            lanes: BTreeMap::new(),
        }
    }

    pub fn queue_order(&mut self, lane: LaneId, order: OrderId, amount: Amount) -> FluxResult<()> {
        if self.closed {
            return Err(FluxError::EpochClosed);
        }

        let accumulator = self.lanes.entry(lane).or_default();
        accumulator.orders.push(order);
        accumulator.input = accumulator.input.checked_add(amount)?;
        Ok(())
    }

    pub fn record_settlement(
        &mut self,
        lane: LaneId,
        output: Amount,
        fee: Amount,
        rebate: Amount,
        residual: Amount,
    ) -> FluxResult<()> {
        if self.closed {
            return Err(FluxError::EpochClosed);
        }

        let accumulator = self.lanes.entry(lane).or_default();
        accumulator.output = accumulator.output.checked_add(output)?;
        accumulator.fees = accumulator.fees.checked_add(fee)?;
        accumulator.rebates = accumulator.rebates.checked_add(rebate)?;
        accumulator.residual = accumulator.residual.checked_add(residual)?;
        Ok(())
    }

    pub fn close(&mut self) {
        self.closed = true;
    }
}

#[derive(Clone, Debug, Default)]
pub struct EpochBook {
    epochs: BTreeMap<EpochId, EpochWindow>,
}

impl EpochBook {
    pub fn insert(&mut self, epoch: EpochWindow) {
        self.epochs.insert(epoch.id, epoch);
    }

    pub fn get(&self, id: EpochId) -> FluxResult<&EpochWindow> {
        self.epochs.get(&id).ok_or(FluxError::EpochNotFound)
    }

    pub fn get_mut(&mut self, id: EpochId) -> FluxResult<&mut EpochWindow> {
        self.epochs.get_mut(&id).ok_or(FluxError::EpochNotFound)
    }

    pub fn len(&self) -> usize {
        self.epochs.len()
    }
}
