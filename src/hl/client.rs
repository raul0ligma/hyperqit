use std::time::SystemTime;
use tracing::{debug, error, info};

use alloy::primitives::{Address, FixedBytes};

use crate::errors::{Errors, Result};
use crate::hl::exchange::{
    ExchangeRequest, ExchangeResponse, generate_action_params, generate_send_asset_params,
    generate_transfer_params,
};
use crate::hl::info::{GetInfoReq, PerpetualsInfo, SpotResponse};
use crate::hl::message::SignedMessage;
use crate::hl::nonce::NonceManager;
use crate::hl::user_info::{
    FundingHistory, GetUserFundingHistoryReq, GetUserInfoReq, UserPerpPosition, UserSpotPosition,
};
use crate::hl::utils::*;
use crate::hl::{Actions, TransferRequest};
use crate::{
    BulkCancel, BulkOrder, CancelOrder, ExchangeOrderResponse, HyperLiquidSigningHash, Order,
    OrderRequest, PerpDeployAction, SendAssetRequest, Signers,
};

pub trait HlAgentWallet {
    async fn sign_order(&self, to_sign: FixedBytes<32>) -> Result<SignedMessage>;
}

pub struct HyperliquidClient {
    client: reqwest::Client,
    signer: Signers,
    network: Network,
    user: Address,
    nonce_manager: NonceManager,
}

impl HyperliquidClient {
    pub fn new(network: Network, signer: Signers, user: Address) -> Self {
        debug!("creating hyperliquid client for {} on {:?}", user, network);
        HyperliquidClient {
            client: reqwest::Client::new(),
            signer,
            network,
            user,
            nonce_manager: NonceManager::new(),
        }
    }

    pub async fn get_user_funding_history(&self, since: u128) -> Result<FundingHistory> {
        debug!(
            "fetching funding history for user {} since {}",
            self.user, since
        );

        let end_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis();

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
            error!("failed to get funding history: {} - {}", status_code, body);
            return Err(Errors::HyperLiquidApiError(status_code, body).into());
        }

        let out: FundingHistory = serde_json::from_str(body.as_str())?;
        debug!("retrieved funding history with {} entries", out.len());

        Ok(out)
    }

    pub async fn update_leverage(&self, a: u32, is_cross: bool, leverage: u32) -> Result<()> {
        info!(
            "updating leverage for asset {} to {}x (cross: {})",
            a, leverage, is_cross
        );

        let nonce = self.nonce_manager.get_next_nonce();

        let action: Actions = Actions::UpdateLeverage(crate::UpdateLeverage {
            asset: a,
            is_cross,
            leverage,
        });

        let is_mainnet = self.network == Network::Mainnet;
        let (to_sign, domain) = generate_action_params(&action, is_mainnet, nonce)?;

        let hash = to_sign.hyperliquid_signing_hash(&domain);

        let signature = self.signer.sign_order(hash).await?;

        let payload = ExchangeRequest {
            action: serde_json::to_value(action)?,
            signature,
            nonce,
        };

        debug!(
            "update leverage payload: {}",
            serde_json::to_string(&payload).unwrap()
        );

        let resp = self
            .client
            .post(format!("{}/exchange", Into::<String>::into(self.network)))
            .json(&payload)
            .send()
            .await?;

        let status_code = resp.status().as_u16();
        let body = resp.text().await?;
        if status_code != 200 {
            error!("failed to update leverage: {} - {}", status_code, body);
            return Err(Errors::HyperLiquidApiError(status_code, body).into());
        }

        let out: ExchangeResponse = serde_json::from_str(body.as_str())?;
        debug!("leverage update response: {:?}", out);

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
    ) -> Result<ExchangeOrderResponse> {
        debug!(
            "creating {} position for asset {} with ${} USD (price: {}, slippage: {}%)",
            if is_buy { "buy" } else { "sell" },
            a,
            sz,
            current_px,
            slippage * 100.0
        );

        let (px, sz) = get_formatted_position_with_amount(
            current_px,
            sz,
            is_perp,
            is_buy,
            sz_decimals,
            slippage,
        );

        self.create_position(a, is_buy, px, sz, reduce_only).await
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
    ) -> Result<ExchangeOrderResponse> {
        debug!(
            "creating {} position for asset {} with size {} (price: {}, slippage: {}%)",
            if is_buy { "buy" } else { "sell" },
            a,
            sz,
            current_px,
            slippage * 100.0
        );

        let (px, sz) = get_formatted_position_with_amount_raw(
            current_px,
            sz,
            is_perp,
            is_buy,
            sz_decimals,
            slippage,
        );

        self.create_position(a, is_buy, px, sz, reduce_only).await
    }

    async fn create_position(
        &self,
        a: u32,
        is_buy: bool,
        px: String,
        sz: String,
        reduce_only: bool,
    ) -> Result<ExchangeOrderResponse> {
        self.create_position_raw(crate::BulkOrder {
            orders: vec![OrderRequest {
                asset: a,
                is_buy,
                limit_px: px.clone(),
                sz: sz.clone(),
                reduce_only,
                order_type: Order::Limit(crate::Limit { tif: "Ioc".into() }),
                cloid: None,
            }],
            grouping: "na".to_string(),
        })
        .await
    }

    pub async fn create_position_raw(&self, orders: BulkOrder) -> Result<ExchangeOrderResponse> {
        let nonce: u64 = self.nonce_manager.get_next_nonce();

        let action: Actions = Actions::Order(orders);

        let is_mainnet = self.network == Network::Mainnet;
        let (to_sign, domain) = generate_action_params(&action, is_mainnet, nonce)?;
        let hash = to_sign.hyperliquid_signing_hash(&domain);
        let signature: SignedMessage = self.signer.sign_order(hash).await?;

        let payload = ExchangeRequest {
            action: serde_json::to_value(action)?,
            signature,
            nonce,
        };

        debug!(
            "order payload: {}",
            serde_json::to_string(&payload).unwrap()
        );

        let resp = self
            .client
            .post(format!("{}/exchange", Into::<String>::into(self.network)))
            .json(&payload)
            .send()
            .await?;

        let status_code = resp.status().as_u16();
        let body = resp.text().await?;
        if status_code != 200 {
            debug!("failed to create position: {} - {}", status_code, body);
            return Err(Errors::HyperLiquidApiError(status_code, body).into());
        }

        let out: ExchangeResponse = serde_json::from_str(body.as_str())?;
        debug!("order response: {:?}", out);
        if out.status != "ok".to_string() {
            return Err(Errors::HyperLiquidApiError(100, out.response.to_string()).into());
        }

        Ok(serde_json::from_value(out.response)?)
    }

    pub async fn transfer_usd_to_spot(&self, amount: u64) -> Result<()> {
        debug!("transferring ${} USD to spot", amount);

        let nonce = self.nonce_manager.get_next_nonce();

        let transfer_req = TransferRequest {
            chain: self.network.name(),
            sig_chain_id: "0xa4b1".to_string(),
            amount: amount.to_string(),
            to_perp: false,
            nonce: nonce,
        };

        debug!("transfer request: {:?}", transfer_req);

        let (to_sign, domain) = generate_transfer_params(&transfer_req)?;
        let hash = to_sign.hyperliquid_signing_hash(&domain);
        let signature = self.signer.sign_order(hash).await?;
        let payload = ExchangeRequest {
            nonce,
            signature,
            action: serde_json::to_value(Actions::UsdClassTransfer(transfer_req))?,
        };

        debug!(
            "transfer payload: {}",
            serde_json::to_string(&payload).unwrap()
        );

        let resp = self
            .client
            .post(format!("{}/exchange", Into::<String>::into(self.network)))
            .json(&payload)
            .send()
            .await?;

        let status_code = resp.status().as_u16();
        let body = resp.text().await?;
        if status_code != 200 {
            error!("failed to transfer USD: {} - {}", status_code, body);
            return Err(Errors::HyperLiquidApiError(status_code, body).into());
        }

        let out: ExchangeResponse = serde_json::from_str(body.as_str())?;
        debug!("transfer response: {:?}", out);
        if out.status != "ok".to_string() {
            return Err(Errors::HyperLiquidApiError(100, out.response.to_string()).into());
        }

        Ok(())
    }

    pub async fn send_asset_to_dex(&self, req: SendAssetRequest) -> Result<()> {
        debug!("transferring to dex {}", req.dst_dex.clone());
        let mut transfer_req = req.clone();
        let nonce = self.nonce_manager.get_next_nonce();
        transfer_req.nonce = nonce;

        debug!("send asset request: {:?}", transfer_req);

        let (to_sign, domain) = generate_send_asset_params(&transfer_req)?;
        debug!("transfer domain: {:?}", domain);

        let hash = to_sign.hyperliquid_signing_hash(&domain);
        let signature = self.signer.sign_order(hash).await?;

        let payload = ExchangeRequest {
            nonce,
            signature,
            action: serde_json::to_value(Actions::SendAsset(transfer_req))?,
        };

        debug!(
            "send asset payload: {}",
            serde_json::to_string(&payload).unwrap()
        );

        let resp = self
            .client
            .post(format!("{}/exchange", Into::<String>::into(self.network)))
            .json(&payload)
            .send()
            .await?;

        let status_code = resp.status().as_u16();
        let body = resp.text().await?;
        if status_code != 200 {
            error!("failed to send asset: {} - {}", status_code, body);
            return Err(Errors::HyperLiquidApiError(status_code, body).into());
        }

        let out: ExchangeResponse = serde_json::from_str(body.as_str())?;
        debug!("send asset response: {:?}", out);
        if out.status != "ok".to_string() {
            return Err(Errors::HyperLiquidApiError(100, out.response.to_string()).into());
        }

        Ok(())
    }

    pub async fn get_perp_info(&self, dex: Option<String>) -> Result<PerpetualsInfo> {
        debug!("fetching perpetuals info");

        let payload = GetInfoReq {
            asset_type: "metaAndAssetCtxs".into(),
            dex,
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
            error!("failed to get perp info: {} - {}", status_code, body);
            return Err(Errors::HyperLiquidApiError(status_code, body).into());
        }

        let out: PerpetualsInfo = serde_json::from_str(body.as_str())?;
        Ok(out)
    }

    pub async fn get_spot_info(&self, dex: Option<String>) -> Result<SpotResponse> {
        debug!("fetching spot info");

        let payload = GetInfoReq {
            asset_type: "spotMetaAndAssetCtxs".into(),
            dex,
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
            error!("failed to get spot info: {} - {}", status_code, body);
            return Err(Errors::HyperLiquidApiError(status_code, body).into());
        }

        let out: SpotResponse = serde_json::from_str(body.as_str())?;
        Ok(out)
    }

    pub async fn get_user_spot_info(&self, dex: Option<String>) -> Result<UserSpotPosition> {
        debug!("fetching user spot positions for {}", self.user);

        let payload = GetUserInfoReq {
            request_type: "spotClearinghouseState".into(),
            user: self.user.to_string(),
            dex,
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
            error!("failed to get user spot info: {} - {}", status_code, body);
            return Err(Errors::HyperLiquidApiError(status_code, body).into());
        }

        debug!("user spot response: {}", body);
        let out: UserSpotPosition = serde_json::from_str(body.as_str())?;

        Ok(out)
    }

    pub async fn get_user_perp_info(&self, dex: Option<String>) -> Result<UserPerpPosition> {
        debug!("fetching user perp positions for {}", self.user);

        let payload = GetUserInfoReq {
            request_type: "clearinghouseState".into(),
            user: self.user.to_string(),
            dex,
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
            error!("failed to get user perp info: {} - {}", status_code, body);
            return Err(Errors::HyperLiquidApiError(status_code, body).into());
        }

        Ok(serde_json::from_str(body.as_str())?)
    }

    pub async fn cancel_order(&self, oid: i64, a: u32) -> Result<ExchangeOrderResponse> {
        debug!("cancelling order {} for asset {}", oid, a);

        self.cancel_order_raw(BulkCancel {
            cancels: vec![CancelOrder { asset: a, oid }],
        })
        .await
    }

    pub async fn cancel_order_raw(&self, orders: BulkCancel) -> Result<ExchangeOrderResponse> {
        debug!("cancelling order raw {:?}", orders);

        let nonce = self.nonce_manager.get_next_nonce();
        let action: Actions = Actions::Cancel(orders);

        let is_mainnet = self.network == Network::Mainnet;
        let (to_sign, domain) = generate_action_params(&action, is_mainnet, nonce)?;
        let hash = to_sign.hyperliquid_signing_hash(&domain);
        let signature = self.signer.sign_order(hash).await?;

        let payload = ExchangeRequest {
            action: serde_json::to_value(action)?,
            signature,
            nonce,
        };

        debug!(
            "cancel order payload: {}",
            serde_json::to_string(&payload).unwrap()
        );

        let resp = self
            .client
            .post(format!("{}/exchange", Into::<String>::into(self.network)))
            .json(&payload)
            .send()
            .await?;

        let status_code = resp.status().as_u16();
        let body = resp.text().await?;
        if status_code != 200 {
            error!("failed to cancel order: {} - {}", status_code, body);
            return Err(Errors::HyperLiquidApiError(status_code, body).into());
        }

        let out: ExchangeResponse = serde_json::from_str(body.as_str())?;
        debug!("cancel order response: {:?}", out);

        Ok(serde_json::from_value(out.response)?)
    }

    pub async fn perp_deploy_action(&self, deploy_params: PerpDeployAction) -> Result<()> {
        debug!("creating perp deploy action {:?}", deploy_params.clone());

        let nonce = self.nonce_manager.get_next_nonce();

        let action: Actions = Actions::PerpDeploy(deploy_params.clone());

        let is_mainnet = self.network == Network::Mainnet;
        let (to_sign, domain) = generate_action_params(&action, is_mainnet, nonce)?;
        let hash = to_sign.hyperliquid_signing_hash(&domain);
        let signature = self.signer.sign_order(hash).await?;

        let payload = ExchangeRequest {
            action: serde_json::to_value(action)?,
            signature,
            nonce,
        };

        debug!(
            "perp deploy action: {}",
            serde_json::to_string(&payload).unwrap()
        );

        let resp = self
            .client
            .post(format!("{}/exchange", Into::<String>::into(self.network)))
            .json(&payload)
            .send()
            .await?;

        let status_code = resp.status().as_u16();
        let body = resp.text().await?;
        if status_code != 200 {
            error!(
                "failed to call perp deploy action: {} - {}",
                status_code, body
            );
            return Err(Errors::HyperLiquidApiError(status_code, body).into());
        }

        let out: ExchangeResponse = serde_json::from_str(body.as_str())?;
        debug!("perp deploy action response: {:?}", out);

        Ok(())
    }
}
