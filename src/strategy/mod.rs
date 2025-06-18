mod strategy;
use serde::{Deserialize, Serialize};

pub enum Asset {
    CommonAsset(String),
    WithPerpAndSpot(String, String),
}

pub enum Amount {
    Usd(String),
    Raw(String),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum StrategyStatus {
    Active,
    InActive,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    perp_amount: f64,
    perp_mid_px: f64,
    perp_pos_usd: f64,
    liq_px: f64,
    spot_amount: f64,
    spot_mid_px: f64,
    spot_pos_usd: f64,
    perp_funding_rate: f64,
    dn_diff: f64,
    funding_earning_nh: f64,
    at: u64,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyState {
    pub status: StrategyStatus,
    pub position: Option<Position>,
}
pub use strategy::*;
