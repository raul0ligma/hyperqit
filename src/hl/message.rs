use alloy::primitives::U256;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignedMessage {
    pub r: U256,
    pub s: U256,
    pub v: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignedMessageHex {
    pub r: String,
    pub s: String,
    pub v: u64,
}

impl From<SignedMessage> for SignedMessageHex {
    fn from(signed_msg: SignedMessage) -> Self {
        SignedMessageHex {
            r: format!("0x{:x}", signed_msg.r),
            s: format!("0x{:x}", signed_msg.s),
            v: signed_msg.v,
        }
    }
}
