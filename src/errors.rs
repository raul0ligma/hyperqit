use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Errors {
    #[error("failed to generate agent signature: {0}")]
    AgentSignature(String),

    #[error("failed to find value: {0} {1}")]
    DataError(String, String),

    #[error("failed to call hyperliquid {0}:{1}")]
    HyperLiquidApiError(u16, String),

    #[error("failed to place order {0}")]
    PlaceOrderError(String),

    #[error("funding rate is negative {0}")]
    FundRateNegative(f64),
}

#[derive(Error, Debug, Clone)]
#[error("expected {expected:?}, got {actual:?}")]
pub struct CmpError<T: std::fmt::Debug + Clone> {
    pub expected: T,
    pub actual: T,
}

pub type Result<T> = anyhow::Result<T>;
