use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderRequest {
    #[serde(rename = "a", alias = "asset")]
    pub asset: u32,
    #[serde(rename = "b", alias = "isBuy")]
    pub is_buy: bool,
    #[serde(rename = "p", alias = "limitPx")]
    pub limit_px: String,
    #[serde(rename = "s", alias = "sz")]
    pub sz: String,
    #[serde(rename = "r", alias = "reduceOnly", default)]
    pub reduce_only: bool,
    #[serde(rename = "t", alias = "orderType")]
    pub order_type: Order,
    #[serde(rename = "c", alias = "cloid", skip_serializing_if = "Option::is_none")]
    pub cloid: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Limit {
    pub tif: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Order {
    Limit(Limit),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TransferRequest {
    #[serde(rename = "hyperliquidChain")]
    pub chain: String,
    #[serde(rename = "signatureChainId")]
    pub sig_chain_id: String,
    #[serde(rename = "amount")]
    pub amount: String,
    #[serde(rename = "toPerp")]
    pub to_perp: bool,
    #[serde(rename = "nonce")]
    pub nonce: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Actions {
    Order(BulkOrder),
    UsdClassTransfer(TransferRequest),
    Cancel(BulkCancel),
    UpdateLeverage(UpdateLeverage),
    PerpDeploy(PerpDeployAction),
    SendAsset(SendAssetRequest),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BulkOrder {
    pub orders: Vec<OrderRequest>,
    pub grouping: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BulkCancel {
    pub cancels: Vec<CancelOrder>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrder {
    #[serde(rename = "a", alias = "asset")]
    pub asset: u32,
    #[serde(rename = "o", alias = "oid")]
    pub oid: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLeverage {
    pub asset: u32,
    pub is_cross: bool,
    pub leverage: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PerpDexSchemaInput {
    pub full_name: String,
    pub collateral_token: u64,
    pub oracle_updater: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegisterAssetRequest {
    pub coin: String,
    pub sz_decimals: u64,
    pub oracle_px: String,
    pub margin_table_id: u64,
    pub only_isolated: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegisterAsset {
    pub max_gas: Option<u64>,
    pub asset_request: RegisterAssetRequest,
    pub dex: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<PerpDexSchemaInput>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]

pub enum PerpDeployAction {
    RegisterAsset(RegisterAsset),
    SetFundingMultiplier(SetFundingMultipliers),
    SetOracle(SetOracle),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetOracle {
    pub dex: String,
    pub oracle_pxs: Vec<[String; 2]>,
    pub mark_pxs: Vec<Vec<[String; 2]>>,
}

pub type SetFundingMultipliers = Vec<[String; 2]>;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SendAssetRequest {
    #[serde(rename = "hyperliquidChain")]
    pub chain: String,
    #[serde(rename = "signatureChainId")]
    pub sig_chain_id: String,
    #[serde(rename = "destination")]
    pub destination: String,
    #[serde(rename = "sourceDex")]
    pub source_dex: String,
    #[serde(rename = "destinationDex")]
    pub dst_dex: String,
    #[serde(rename = "amount")]
    pub amount: String,
    #[serde(rename = "token")]
    pub token: String,
    #[serde(rename = "fromSubAccount")]
    pub from_sub_account: String,
    #[serde(rename = "nonce")]
    pub nonce: u64,
}
