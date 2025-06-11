use alloy::primitives::U256;
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignedMessage {
    pub r: U256,
    pub s: U256,
    pub v: u64,
}
