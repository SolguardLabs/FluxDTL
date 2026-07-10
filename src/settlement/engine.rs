use crate::amount::Amount;
use crate::asset::AssetRegistry;
use crate::error::{FluxError, FluxResult};
use crate::lanes::LaneConfig;
use crate::oracle::Oracle;

#[derive(Copy, Clone, Debug)]
pub struct SettlementQuote {
    pub gross_out: Amount,
    pub protocol_fee: Amount,
    pub maker_rebate: Amount,
    pub recipient_out: Amount,
    pub netted_input_value: Amount,
}

pub struct SettlementEngine;

impl SettlementEngine {
    pub fn quote(
        assets: &AssetRegistry,
        oracle: &Oracle,
        lane: &LaneConfig,
        amount_in: Amount,
    ) -> FluxResult<SettlementQuote> {
        let source_price = oracle.price(lane.source_asset)?;
        let target_price = oracle.price(lane.target_asset)?;
        if source_price.confidence_bps < lane.policy.min_confidence_bps
            || target_price.confidence_bps < lane.policy.min_confidence_bps
        {
            return Err(FluxError::PriceConfidence);
        }

        let gross_out = oracle.quote(assets, lane.source_asset, lane.target_asset, amount_in)?;
        let protocol_fee = gross_out.mul_bps_floor(lane.policy.fee_bps)?;
        let maker_rebate = gross_out.mul_bps_floor(lane.policy.maker_rebate_bps)?;
        let recipient_out = gross_out.checked_sub(protocol_fee)?;
        let netted_input_value =
            oracle.quote(assets, lane.source_asset, lane.target_asset, amount_in)?;

        Ok(SettlementQuote {
            gross_out,
            protocol_fee,
            maker_rebate,
            recipient_out,
            netted_input_value,
        })
    }
}
