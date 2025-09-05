// Module declarations
mod client;
mod errors;
mod internal;
mod market_info;
mod order_responses;
mod requests;
mod signing;
mod user_data;
mod utils;
mod wallet;

pub use client::HyperliquidClient;
pub use errors::{CmpError, Errors, Result};
pub use market_info::{
    PerpMarketInfo, SpotMarketInfo, create_unified_market_info, find_market_by_name,
};
pub use order_responses::*;
pub use requests::*;
pub use signing::{SignedMessage, SignedMessageHex, Signer};
pub use user_data::*;
pub use utils::*;
pub use wallet::{HyperLiquidSigningHash, LocalWallet};
