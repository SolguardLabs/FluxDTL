use crate::error::{FluxError, FluxResult};
use std::fmt;

pub const BPS_DENOMINATOR: u128 = 10_000;
pub type BasisPoints = u16;

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Amount(u128);

impl Amount {
    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn new(value: u128) -> Self {
        Self(value)
    }

    pub const fn raw(self) -> u128 {
        self.0
    }

    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub fn checked_add(self, other: Self) -> FluxResult<Self> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .ok_or(FluxError::AmountOverflow)
    }

    pub fn checked_sub(self, other: Self) -> FluxResult<Self> {
        self.0
            .checked_sub(other.0)
            .map(Self)
            .ok_or(FluxError::InsufficientBalance)
    }

    pub fn checked_mul(self, factor: u128) -> FluxResult<Self> {
        self.0
            .checked_mul(factor)
            .map(Self)
            .ok_or(FluxError::AmountOverflow)
    }

    pub fn checked_div(self, divisor: u128) -> FluxResult<Self> {
        if divisor == 0 {
            return Err(FluxError::AmountOverflow);
        }

        Ok(Self(self.0 / divisor))
    }

    pub fn saturating_sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }

    pub fn min(self, other: Self) -> Self {
        if self <= other {
            self
        } else {
            other
        }
    }

    pub fn mul_bps_floor(self, bps: BasisPoints) -> FluxResult<Self> {
        self.0
            .checked_mul(u128::from(bps))
            .and_then(|value| value.checked_div(BPS_DENOMINATOR))
            .map(Self)
            .ok_or(FluxError::AmountOverflow)
    }

    pub fn mul_ratio_floor(self, numerator: u128, denominator: u128) -> FluxResult<Self> {
        if denominator == 0 {
            return Err(FluxError::AmountOverflow);
        }

        self.0
            .checked_mul(numerator)
            .and_then(|value| value.checked_div(denominator))
            .map(Self)
            .ok_or(FluxError::AmountOverflow)
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
