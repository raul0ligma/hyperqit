use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Errors {
    #[error("failed to generate agent signature: {0:?}")]
    AgentSignature(String),

    #[error("failed to call hyperliquid {0}:{1}")]
    HyperLiquidApiError(u16, String),
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
