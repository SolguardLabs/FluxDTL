use std::fmt;

pub type FluxResult<T> = Result<T, FluxError>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FluxError {
    AmountOverflow,
    InsufficientBalance,
    AssetNotFound,
    AssetDisabled,
    AccountNotFound,
    VaultNotFound,
    LaneNotFound,
    LaneDisabled,
    OrderNotFound,
    OrderNotPending,
    EpochNotFound,
    EpochClosed,
    PriceNotAvailable,
    PriceConfidence,
    InvalidVaultAsset,
    MinimumOutput,
    RiskLimitExceeded,
    LiquidityCreditExceeded,
}

impl fmt::Display for FluxError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::AmountOverflow => "amount overflow",
            Self::InsufficientBalance => "insufficient balance",
            Self::AssetNotFound => "asset not found",
            Self::AssetDisabled => "asset disabled",
            Self::AccountNotFound => "account not found",
            Self::VaultNotFound => "vault not found",
            Self::LaneNotFound => "lane not found",
            Self::LaneDisabled => "lane disabled",
            Self::OrderNotFound => "order not found",
            Self::OrderNotPending => "order is not pending",
            Self::EpochNotFound => "epoch not found",
            Self::EpochClosed => "epoch is closed",
            Self::PriceNotAvailable => "price not available",
            Self::PriceConfidence => "price confidence is below lane policy",
            Self::InvalidVaultAsset => "invalid vault asset",
            Self::MinimumOutput => "minimum output was not satisfied",
            Self::RiskLimitExceeded => "risk limit exceeded",
            Self::LiquidityCreditExceeded => "liquidity credit exceeded",
        };
        write!(formatter, "{message}")
    }
}

impl std::error::Error for FluxError {}
