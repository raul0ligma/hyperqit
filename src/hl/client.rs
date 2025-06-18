use std::time::SystemTime;

use alloy::dyn_abi::Eip712Domain;
use alloy::primitives::Address;

use crate::errors::{Errors, Result};
use crate::hl::exchange::{
    ExchangeRequest, ExchangeResponse, generate_action_params, generate_transfer_params,
};
use crate::hl::info::{GetInfoReq, PerpetualsInfo, SpotResponse};
use crate::hl::message::SignedMessage;
use crate::hl::user_info::{
    FundingHistory, GetUserFundingHistoryReq, GetUserInfoReq, UserPerpPosition, UserSpotPosition,
};
use crate::hl::{Actions, TransferRequest};
use crate::{CancelOrder, HyperLiquidSigningHash, Order, OrderRequest};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Testnet,
}

impl Network {
    pub fn name(self) -> String {
        match self {
            Network::Mainnet => "Mainnet".to_string(),
            Network::Testnet => "Testnet".to_string(),
        }
    }
}

impl From<Network> for String {
    fn from(val: Network) -> Self {
        match val {
            Network::Mainnet => "https://api.hyperliquid.xyz".to_string(),
            Network::Testnet => "https://api.hyperliquid-testnet.xyz".to_string(),
        }
    }
}

const MAX_SIGNIFICANT_DIGITS: i32 = 5i32;
const MAX_DECIMALS_SPOT: i32 = 8i32;
const MAX_DECIMALS_PERP: i32 = 6i32;

fn format_decimals(v: f64, decimals: i32) -> f64 {
    let decimal_shift = 10f64.powi(decimals);

    (v * decimal_shift).round() / decimal_shift
}

fn format_significant_digits_and_decimals(v: f64, decimals: i32) -> f64 {
    // m is magnitude,
    let m = v.abs().log10().floor() as i32;
    let scale = 10f64.powi(MAX_SIGNIFICANT_DIGITS - m - 1);
    let shifted = (v * scale).round() / scale;
    format_decimals(shifted, decimals)
}

fn get_formatted_position_with_amount(
    current_px: f64,
    size_in_usd: f64,
    is_perp: bool,
    is_buy: bool,
    sz_decimals: i32,
    slippage: f64,
) -> (String, String) {
    let sz_raw = size_in_usd / current_px;

    get_formatted_position_with_amount_raw(
        current_px,
        sz_raw,
        is_perp,
        is_buy,
        sz_decimals,
        slippage,
    )
}

fn get_formatted_position_with_amount_raw(
    current_px: f64,
    sz_raw: f64,
    is_perp: bool,
    is_buy: bool,
    sz_decimals: i32,
    slippage: f64,
) -> (String, String) {
    let out_px = if is_buy {
        current_px * (1.0 + slippage)
    } else {
        current_px * (1.0 - slippage)
    };

    let sz = format_decimals(sz_raw, sz_decimals);
    let decimals = if is_perp {
        MAX_DECIMALS_PERP - sz_decimals
    } else {
        MAX_DECIMALS_SPOT - sz_decimals
    };
    let px = format_significant_digits_and_decimals(out_px, decimals);
    (px.to_string(), sz.to_string())
}
pub trait HlAgentWallet {
    async fn sign_order(
        &self,
        domain: Eip712Domain,
        to_sign: impl HyperLiquidSigningHash,
    ) -> Result<SignedMessage>;
}
pub struct HyperliquidClient<S>
where
    S: HlAgentWallet,
{
    client: reqwest::Client,
    signer: S,
    network: Network,
    user: Address,
}

impl<S> HyperliquidClient<S>
where
    S: HlAgentWallet,
{
    pub fn new(network: Network, signer: S, user: Address) -> Self {
        HyperliquidClient {
            client: reqwest::Client::new(),
            signer,
            network,
            user,
        }
    }

    pub async fn get_user_funding_history(&self, since: u128) -> Result<FundingHistory> {
        let end_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis() as u128;

        let req = GetUserFundingHistoryReq {
            request_type: "userFunding".into(),
            user: self.user.to_string(),
            end_time,
            start_time: end_time - since,
        };

        let resp = self
            .client
            .post(format!("{}/info", Into::<String>::into(self.network)))
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        let status_code = resp.status().as_u16();
        let body = resp.text().await?;
        if status_code != 200 {
            return Err(Box::new(Errors::HyperLiquidApiError(status_code, body)));
        }
        let out: FundingHistory = serde_json::from_str(body.as_str())?;
        Ok(out)
    }

    pub async fn update_leverage(&self, a: u32, is_cross: bool, leverage: u32) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis() as u64;

        let action: Actions = Actions::UpdateLeverage(crate::UpdateLeverage {
            asset: a,
            is_cross: is_cross,
            leverage: leverage,
        });

        let is_mainnet = self.network == Network::Mainnet;
        let (to_sign, domain) = generate_action_params(&action, is_mainnet, timestamp)?;

        let signature = self.signer.sign_order(domain, to_sign).await?;

        let payload = ExchangeRequest {
            action: serde_json::to_value(action)?,
            signature,
            nonce: timestamp,
        };
        let out = serde_json::to_string(&payload).unwrap();
        println!("{} body", out);
        let resp = self
            .client
            .post(format!("{}/exchange", Into::<String>::into(self.network)))
            .json(&payload)
            .send()
            .await?;
        let status_code = resp.status().as_u16();
        let body = resp.text().await?;
        if status_code != 200 {
            return Err(Box::new(Errors::HyperLiquidApiError(status_code, body)));
        }

        let out: ExchangeResponse = serde_json::from_str(body.as_str())?;
        println!("{:?}", out);
        Ok(())
    }

    pub async fn create_position_with_size_in_usd(
        &self,
        a: u32,
        is_perp: bool,
        is_buy: bool,
        current_px: f64,
        sz: f64,
        reduce_only: bool,
        slippage: f64,
        sz_decimals: i32,
    ) -> Result<()> {
        let (px, sz) = get_formatted_position_with_amount(
            current_px,
            sz,
            is_perp,
            is_buy,
            sz_decimals,
            slippage,
        );

        self.create_position(a, is_buy, px, sz, reduce_only).await?;
        Ok(())
    }

    pub async fn create_position_with_size(
        &self,
        a: u32,
        is_perp: bool,
        is_buy: bool,
        current_px: f64,
        sz: f64,
        reduce_only: bool,
        slippage: f64,
        sz_decimals: i32,
    ) -> Result<()> {
        let (px, sz) = get_formatted_position_with_amount_raw(
            current_px,
            sz,
            is_perp,
            is_buy,
            sz_decimals,
            slippage,
        );

        self.create_position(a, is_buy, px, sz, reduce_only).await?;
        Ok(())
    }

    async fn create_position(
        &self,
        a: u32,
        is_buy: bool,
        px: String,
        sz: String,
        reduce_only: bool,
    ) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis() as u64;

        let action: Actions = Actions::Order(crate::BulkOrder {
            orders: vec![OrderRequest {
                asset: a,
                is_buy: is_buy,
                limit_px: px,
                sz: sz,
                reduce_only,
                order_type: Order::Limit(crate::Limit { tif: "Ioc".into() }),
                cloid: None,
            }],
            grouping: "na".to_string(),
        });

        let is_mainnet = self.network == Network::Mainnet;
        let (to_sign, domain) = generate_action_params(&action, is_mainnet, timestamp)?;

        let signature = self.signer.sign_order(domain, to_sign).await?;

        let payload = ExchangeRequest {
            action: serde_json::to_value(action)?,
            signature,
            nonce: timestamp,
        };
        let out = serde_json::to_string(&payload).unwrap();
        println!("{} body", out);
        let resp = self
            .client
            .post(format!("{}/exchange", Into::<String>::into(self.network)))
            .json(&payload)
            .send()
            .await?;
        let status_code = resp.status().as_u16();
        let body = resp.text().await?;
        if status_code != 200 {
            return Err(Box::new(Errors::HyperLiquidApiError(status_code, body)));
        }

        let out: ExchangeResponse = serde_json::from_str(body.as_str())?;
        println!("{:?}", out);
        Ok(())
    }

    pub async fn transfer_usd_to_spot(&self, amount: u64) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis() as u64;

        let transfer_req = TransferRequest {
            chain: self.network.name(),
            sig_chain_id: "0xa4b1".to_string(),
            amount: amount.to_string(),
            to_perp: false,
            nonce: timestamp,
        };

        println!("{:?}", transfer_req);

        let (to_sign, domain) = generate_transfer_params(&transfer_req)?;

        println!("{:?}", domain);

        let signature = self.signer.sign_order(domain, to_sign).await?;

        let payload = ExchangeRequest {
            nonce: timestamp,
            signature,
            action: serde_json::to_value(Actions::UsdClassTransfer(transfer_req))?,
        };
        let out = serde_json::to_string(&payload).unwrap();
        println!("{} body", out);
        let resp = self
            .client
            .post(format!("{}/exchange", Into::<String>::into(self.network)))
            .json(&payload)
            .send()
            .await?;

        let status_code = resp.status().as_u16();
        let body = resp.text().await?;
        if status_code != 200 {
            return Err(Box::new(Errors::HyperLiquidApiError(status_code, body)));
        }

        let out: ExchangeResponse = serde_json::from_str(body.as_str())?;
        println!("{:?}", out);
        Ok(())
    }
    pub async fn get_perp_info(&self) -> Result<PerpetualsInfo> {
        let payload = GetInfoReq {
            asset_type: "metaAndAssetCtxs".into(),
        };

        let resp = self
            .client
            .post(format!("{}/info", Into::<String>::into(self.network)))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let status_code = resp.status().as_u16();
        let body = resp.text().await?;
        if status_code != 200 {
            return Err(Box::new(Errors::HyperLiquidApiError(status_code, body)));
        }

        let out: PerpetualsInfo = serde_json::from_str(body.as_str())?;
        Ok(out)
    }

    pub async fn get_spot_info(&self) -> Result<(SpotResponse)> {
        let payload = GetInfoReq {
            asset_type: "spotMetaAndAssetCtxs".into(),
        };

        let resp = self
            .client
            .post(format!("{}/info", Into::<String>::into(self.network)))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let status_code = resp.status().as_u16();
        let body = resp.text().await?;
        if status_code != 200 {
            return Err(Box::new(Errors::HyperLiquidApiError(status_code, body)));
        }

        let out: SpotResponse = serde_json::from_str(body.as_str())?;
        Ok(out)
    }

    pub async fn get_user_spot_info(&self) -> Result<(UserSpotPosition)> {
        let payload = GetUserInfoReq {
            request_type: "spotClearinghouseState".into(),
            user: self.user.to_string(),
        };

        let resp = self
            .client
            .post(format!("{}/info", Into::<String>::into(self.network)))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let status_code = resp.status().as_u16();
        let body = resp.text().await?;
        if status_code != 200 {
            return Err(Box::new(Errors::HyperLiquidApiError(status_code, body)));
        }

        println!("{}", body);
        let out: UserSpotPosition = serde_json::from_str(body.as_str())?;
        Ok(out)
    }

    pub async fn get_user_perp_info(&self) -> Result<UserPerpPosition> {
        let payload = GetUserInfoReq {
            request_type: "clearinghouseState".into(),
            user: self.user.to_string(),
        };

        let resp = self
            .client
            .post(format!("{}/info", Into::<String>::into(self.network)))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let status_code = resp.status().as_u16();
        let body = resp.text().await?;
        if status_code != 200 {
            return Err(Box::new(Errors::HyperLiquidApiError(status_code, body)));
        }

        let out: UserPerpPosition = serde_json::from_str(body.as_str())?;
        Ok(out)
    }

    pub async fn cancel_order(&self, oid: i64, a: u32) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis() as u64;

        let action: Actions = Actions::Cancel(crate::BulkCancel {
            cancels: vec![CancelOrder { asset: a, oid: oid }],
        });

        let is_mainnet = self.network == Network::Mainnet;
        let (to_sign, domain) = generate_action_params(&action, is_mainnet, timestamp)?;

        let signature = self.signer.sign_order(domain, to_sign).await?;

        let payload = ExchangeRequest {
            action: serde_json::to_value(action)?,
            signature,
            nonce: timestamp,
        };
        let out = serde_json::to_string(&payload).unwrap();
        println!("{} body", out);
        let resp = self
            .client
            .post(format!("{}/exchange", Into::<String>::into(self.network)))
            .json(&payload)
            .send()
            .await?;
        let status_code = resp.status().as_u16();
        let body = resp.text().await?;
        if status_code != 200 {
            return Err(Box::new(Errors::HyperLiquidApiError(status_code, body)));
        }

        let out: ExchangeResponse = serde_json::from_str(body.as_str())?;
        println!("{:?}", out);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt() -> Result<()> {
        println!(
            "{}",
            format_significant_digits_and_decimals(1.5655555555, 3)
        );
        Ok(())
    }
}
