use crate::errors::Result;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPerpPosition {
    pub asset_positions: Vec<AssetPosition>,
    pub cross_maintenance_margin_used: String,
    pub cross_margin_summary: CrossMarginSummary,
    pub margin_summary: MarginSummary,
    pub time: i64,
    pub withdrawable: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetPosition {
    pub position: Position,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub coin: String,
    pub cum_funding: CumFunding,
    pub entry_px: String,
    pub leverage: Leverage,
    pub liquidation_px: Option<String>,
    pub margin_used: String,
    pub max_leverage: i64,
    pub position_value: String,
    pub return_on_equity: String,
    pub szi: String,
    pub unrealized_pnl: String,
}

impl Position {
    pub fn get_close_order_info(&self) -> Result<(bool, f64)> {
        let parsed_sz: f64 = self.szi.parse()?;
        let is_long = parsed_sz > 0.0;
        Ok((!is_long, parsed_sz.abs()))
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CumFunding {
    pub all_time: String,
    pub since_change: String,
    pub since_open: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Leverage {
    #[serde(rename = "type")]
    pub type_field: String,
    pub value: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossMarginSummary {
    pub account_value: String,
    pub total_margin_used: String,
    pub total_ntl_pos: String,
    pub total_raw_usd: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarginSummary {
    pub account_value: String,
    pub total_margin_used: String,
    pub total_ntl_pos: String,
    pub total_raw_usd: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSpotPosition {
    pub balances: Vec<Balance>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub coin: String,
    pub token: i64,
    pub hold: String,
    pub total: String,
    pub entry_ntl: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetUserInfoReq {
    #[serde(rename = "type")]
    pub request_type: String,
    pub user: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dex: Option<String>,
}

pub type FundingHistory = Vec<UserTransaction>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserTransaction {
    pub delta: Delta,
    pub hash: String,
    pub time: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Delta {
    pub coin: String,
    pub funding_rate: String,
    pub szi: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub usdc: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetUserFundingHistoryReq {
    #[serde(rename = "type")]
    pub request_type: String,
    pub user: String,
    pub start_time: u128,
    pub end_time: u128,
}
