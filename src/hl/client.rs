use alloy::sol_types::SolStruct;
use anyhow::Ok;
use async_trait::async_trait;

use std::time::SystemTime;
use tracing::{debug, error, info};

use alloy::primitives::{Address, FixedBytes};

use crate::errors::{Errors, Result};
use crate::hl::exchange::{
    CONVERT_TO_MULTI_SIG_USER_MULTISIG_TYPE, CONVERT_TO_MULTI_SIG_USER_TYPE, ConvertToMultiSigUser,
    ExchangeRequest, ExchangeResponse, MultiSigConvertToMultiSigUser, MultiSigSendAsset,
    MultiSigUsdClassTransfer, MultiSigUsdSend, SEND_ASSET_MULTISIG_TYPE, SEND_ASSET_TYPE,
    SendAsset, USD_CLASS_TRANSFER_MULTISIG_TYPE, USD_CLASS_TRANSFER_TYPE, USD_SEND_MULTISIG_TYPE,
    UsdClassTransfer, generate_action_params, generate_multi_sig_hash, generate_multi_sig_l1_hash,
    hyperliquid_signing_hash_with_default_domain,
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
    BulkCancel, BulkOrder, CancelOrder, ConvertToMultiSigUserRequest, ExchangeOrderResponse,
    GetHistoricalOrders, GetUserFills, GetUserMultiSigConfig, GetUserOpenOrders,
    HyperLiquidSigningHash, LocalWallet, MultiSigConfig, MultiSigPayload, MultiSigRequest, Order,
    OrderRequest, PerpDeployAction, SendAssetRequest, SignedMessageHex, Signers, UsdSendRequest,
    UserFillsResponse, UserMultiSigConfig, UserOpenOrdersResponse, UserOrderHistoryResponse,
};

#[async_trait]
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

        Ok(serde_json::from_str(body.as_str())?)
    }

    pub async fn get_user_open_orders(
        &self,
        dex: Option<String>,
    ) -> Result<UserOpenOrdersResponse> {
        debug!("fetching open orders for user {}", self.user);

        let req = GetUserOpenOrders {
            request_type: "openOrders".into(),
            user: self.user.to_string(),
            dex,
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

        Ok(serde_json::from_str(body.as_str())?)
    }

    pub async fn get_user_history(&self) -> Result<UserOrderHistoryResponse> {
        debug!("fetching historicalOrders orders for user {}", self.user);

        let req = GetHistoricalOrders {
            request_type: "historicalOrders".into(),
            user: self.user.to_string(),
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

        Ok(serde_json::from_str(body.as_str())?)
    }

    pub async fn get_user_fills(&self, aggregate_by_time: bool) -> Result<UserFillsResponse> {
        debug!("fetching fills for user {}", self.user);

        let req = GetUserFills {
            request_type: "userFills".into(),
            user: self.user.to_string(),
            aggregate_by_time,
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

        Ok(serde_json::from_str(body.as_str())?)
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

    pub async fn get_user_multi_sig_config(
        &self,
        user: Address,
    ) -> Result<Option<UserMultiSigConfig>> {
        debug!("fetching multi sig config for user {}", self.user);

        let req = GetUserMultiSigConfig {
            request_type: "userToMultiSigSigners".into(),
            user: user.to_string(),
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
            error!(
                "failed to get user multi sig config: {} - {}",
                status_code, body
            );
            return Err(Errors::HyperLiquidApiError(status_code, body).into());
        }

        Ok(serde_json::from_str(body.as_str())?)
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
        if out.status != *"ok" {
            return Err(Errors::HyperLiquidApiError(100, out.response.to_string()).into());
        }

        Ok(serde_json::from_value(out.response)?)
    }

    pub async fn transfer_usd(
        &self,
        amount: u64,
        to_perp: bool,
        sig_chain_id: String,
    ) -> Result<()> {
        debug!("transferring ${} USD to spot", amount);

        let nonce = self.nonce_manager.get_next_nonce();

        let transfer_req = TransferRequest {
            chain: self.network.name(),
            sig_chain_id: sig_chain_id,
            amount: amount.to_string(),
            to_perp: to_perp,
            nonce,
        };

        let sig_chain_id_u64 = parse_chain_id(&transfer_req.sig_chain_id)?;

        debug!("transfer request: {:?}", transfer_req);

        let hash: FixedBytes<32> = hyperliquid_signing_hash_with_default_domain(
            USD_CLASS_TRANSFER_TYPE.to_owned(),
            UsdClassTransfer {
                hyperliquidChain: transfer_req.chain.clone(),
                amount: transfer_req.amount.clone(),
                toPerp: transfer_req.to_perp,
                nonce: nonce,
            },
            sig_chain_id_u64,
        );

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
        if out.status != *"ok" {
            return Err(Errors::HyperLiquidApiError(100, out.response.to_string()).into());
        }

        Ok(())
    }

    pub async fn send_asset_to_dex(&self, req: SendAssetRequest) -> Result<()> {
        debug!("transferring to dex {}", req.dst_dex.clone());
        let mut transfer_req = req.clone();
        let nonce = self.nonce_manager.get_next_nonce();
        transfer_req.nonce = nonce;

        let sig_chain_id_u64 = parse_chain_id(&transfer_req.sig_chain_id)?;

        debug!("send asset request: {:?}", transfer_req);

        let hash: FixedBytes<32> = hyperliquid_signing_hash_with_default_domain(
            SEND_ASSET_TYPE.to_owned(),
            SendAsset {
                hyperliquidChain: transfer_req.chain.clone(),
                destination: transfer_req.destination.clone(),
                sourceDex: transfer_req.source_dex.clone(),
                destinationDex: transfer_req.dst_dex.clone(),
                token: transfer_req.token.clone(),
                amount: transfer_req.amount.clone(),
                fromSubAccount: transfer_req.from_sub_account.clone(),
                nonce: nonce,
            },
            sig_chain_id_u64,
        );

        debug!("transfer hash: {:?}", hash);
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
        if out.status != *"ok" {
            return Err(Errors::HyperLiquidApiError(100, out.response.to_string()).into());
        }

        Ok(())
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

    pub async fn convert_to_multi_sig(
        &self,
        sig_chain_id: String,
        mut signers: Vec<Address>,
        threshold: u64,
    ) -> Result<()> {
        let nonce = self.nonce_manager.get_next_nonce();
        signers.sort();

        let sig_chain_id_u64 = parse_chain_id(&sig_chain_id)?;
        let config_str = serde_json::to_string(&MultiSigConfig {
            authorized_users: signers.iter().map(|s| s.to_string()).collect(),
            threshold: threshold,
        })?;

        let convert_action: ConvertToMultiSigUserRequest = ConvertToMultiSigUserRequest {
            sig_chain_id,
            chain: self.network.name(),
            signers: config_str,
            nonce,
        };

        let hash = hyperliquid_signing_hash_with_default_domain(
            CONVERT_TO_MULTI_SIG_USER_TYPE.to_owned(),
            ConvertToMultiSigUser {
                hyperliquidChain: convert_action.chain.clone(),
                signers: convert_action.signers.clone(),
                nonce,
            },
            sig_chain_id_u64,
        );

        let signature = self.signer.sign_order(hash).await?;

        let payload = ExchangeRequest {
            nonce,
            signature,
            action: serde_json::to_value(Actions::ConvertToMultiSigUser(convert_action))?,
        };

        let resp = self
            .client
            .post(format!("{}/exchange", Into::<String>::into(self.network)))
            .json(&payload)
            .send()
            .await?;

        let status_code = resp.status().as_u16();
        let body = resp.text().await?;
        if status_code != 200 {
            error!("failed to convert to multisig: {} - {}", status_code, body);
            return Err(Errors::HyperLiquidApiError(status_code, body).into());
        }

        let out: ExchangeResponse = serde_json::from_str(body.as_str())?;
        debug!("convert to multisig response: {:?}", out);
        if out.status != *"ok" {
            return Err(Errors::HyperLiquidApiError(100, out.response.to_string()).into());
        }

        Ok(())
    }

    async fn execute_chain_multi_sig_action<T: SolStruct>(
        &self,
        nonce: u64,
        action: Actions,
        multisig_payload: T,
        sig_type: String,
        sig_chain_id: String,
        other_signers: Vec<Signers>,
        multi_sig_user: Address,
    ) -> Result<()> {
        let sig_chain_id_u64 = parse_chain_id(&sig_chain_id)?;

        // Generate inner hash for the multi-sig payload
        let inner_hash = hyperliquid_signing_hash_with_default_domain(
            sig_type,
            multisig_payload,
            sig_chain_id_u64,
        );

        // Collect signatures from leader and other signers
        let leader_signature = self.signer.sign_order(inner_hash).await?;
        let mut signatures: Vec<SignedMessageHex> = vec![leader_signature.into()];

        for other_signer in other_signers {
            let other_sig = other_signer.sign_order(inner_hash).await?;
            signatures.push(other_sig.into());
        }

        // Create multi-sig request
        let multi_sig_payload = MultiSigRequest {
            sig_chain_id,
            signatures,
            payload: MultiSigPayload {
                multi_sig_user: multi_sig_user.to_string().to_lowercase(),
                outer_signer: self.user.to_string().to_lowercase(),
                action: Box::new(action),
            },
        };

        // Generate outer signature hash and sign
        let outer_hash = generate_multi_sig_hash(multi_sig_payload.clone(), self.network, nonce)?;
        let leader_outer_signature = self.signer.sign_order(outer_hash).await?;

        // Send the request
        let payload = ExchangeRequest {
            nonce,
            signature: leader_outer_signature,
            action: serde_json::to_value(Actions::MultiSig(multi_sig_payload))?,
        };

        self.send_exchange_request(payload).await
    }

    /// Multi-sig USD class transfer (spot <-> perp)
    pub async fn multi_sig_usd_class_transfer(
        &self,
        amount: u64,
        to_perp: bool,
        sig_chain_id: String,
        other_signers: Vec<Signers>,
        multi_sig_user: Address,
    ) -> Result<()> {
        debug!(
            "multi-sig USD class transfer: ${} {} for user {}",
            amount,
            if to_perp { "to perp" } else { "to spot" },
            multi_sig_user
        );

        let nonce = self.nonce_manager.get_next_nonce();

        let transfer_req = TransferRequest {
            chain: self.network.name(),
            sig_chain_id: sig_chain_id.clone(),
            amount: amount.to_string(),
            to_perp,
            nonce,
        };

        let multisig_transfer_data = MultiSigUsdClassTransfer {
            hyperliquidChain: transfer_req.chain.clone(),
            payloadMultiSigUser: multi_sig_user,
            outerSigner: self.user,
            amount: transfer_req.amount.clone(),
            toPerp: to_perp,
            nonce,
        };

        self.execute_chain_multi_sig_action(
            nonce,
            Actions::UsdClassTransfer(transfer_req),
            multisig_transfer_data,
            USD_CLASS_TRANSFER_MULTISIG_TYPE.to_owned(),
            sig_chain_id,
            other_signers,
            multi_sig_user,
        )
        .await
    }

    /// Multi-sig send asset between DEXs
    pub async fn multi_sig_send_asset(
        &self,
        destination: Address,
        source_dex: String,
        destination_dex: String,
        token: String,
        amount: String,
        from_sub_account: Option<String>,
        sig_chain_id: String,
        other_signers: Vec<Signers>,
        multi_sig_user: Address,
    ) -> Result<()> {
        debug!(
            "multi-sig send asset: {} {} from {} to {} (destination: {})",
            amount, token, source_dex, destination_dex, destination
        );

        let nonce = self.nonce_manager.get_next_nonce();

        let send_asset_req = SendAssetRequest {
            chain: self.network.name(),
            sig_chain_id: sig_chain_id.clone(),
            destination: destination.to_string(),
            source_dex,
            dst_dex: destination_dex.clone(),
            token: token.clone(),
            amount: amount.clone(),
            from_sub_account: from_sub_account.unwrap_or_default(),
            nonce,
        };

        let multisig_send_data = MultiSigSendAsset {
            hyperliquidChain: send_asset_req.chain.clone(),
            payloadMultiSigUser: multi_sig_user,
            outerSigner: self.user,
            destination: send_asset_req.destination.clone(),
            sourceDex: send_asset_req.source_dex.clone(),
            destinationDex: send_asset_req.dst_dex.clone(),
            token: send_asset_req.token.clone(),
            amount: send_asset_req.amount.clone(),
            fromSubAccount: send_asset_req.from_sub_account.clone(),
            nonce,
        };

        self.execute_chain_multi_sig_action(
            nonce,
            Actions::SendAsset(send_asset_req),
            multisig_send_data,
            SEND_ASSET_MULTISIG_TYPE.to_owned(),
            sig_chain_id,
            other_signers,
            multi_sig_user,
        )
        .await
    }

    /// Multi-sig USD send (L1 withdrawal)
    pub async fn multi_sig_usd_send(
        &self,
        destination: Address,
        amount: String,
        sig_chain_id: String,
        other_signers: Vec<Signers>,
        multi_sig_user: Address,
    ) -> Result<()> {
        debug!(
            "multi-sig USD send: ${} to {} for user {}",
            amount, destination, multi_sig_user
        );

        let nonce = self.nonce_manager.get_next_nonce();

        let usd_send_req = UsdSendRequest {
            chain: self.network.name(),
            sig_chain_id: sig_chain_id.clone(),
            destination: destination.to_string(),
            amount: amount.clone(),
            time: nonce,
        };

        let multisig_usd_send_data = MultiSigUsdSend {
            hyperliquidChain: usd_send_req.chain.clone(),
            payloadMultiSigUser: multi_sig_user,
            outerSigner: self.user,
            destination: usd_send_req.destination.clone(),
            amount: usd_send_req.amount.clone(),
            time: nonce,
        };

        self.execute_chain_multi_sig_action(
            nonce,
            Actions::UsdSend(usd_send_req),
            multisig_usd_send_data,
            USD_SEND_MULTISIG_TYPE.to_owned(),
            sig_chain_id,
            other_signers,
            multi_sig_user,
        )
        .await
    }

    /// Multi-sig convert to multi-sig user
    pub async fn multi_sig_convert_to_multisig_user(
        &self,
        signers: Option<Vec<Address>>,
        threshold: u64,
        sig_chain_id: String,
        other_signers: Vec<Signers>,
        multi_sig_user: Address,
    ) -> Result<()> {
        debug!(
            "multi-sig convert to multisig user: {:?} signers, threshold {:?}",
            signers, threshold
        );

        let nonce = self.nonce_manager.get_next_nonce();
        let config_str = match signers {
            Some(signers) => {
                let mut sorted_signers = signers;
                sorted_signers.sort();
                serde_json::to_string(&MultiSigConfig {
                    authorized_users: sorted_signers.iter().map(|s| s.to_string()).collect(),
                    threshold,
                })?
            }
            None => "null".to_owned(),
        };

        let convert_req = ConvertToMultiSigUserRequest {
            sig_chain_id: sig_chain_id.clone(),
            chain: self.network.name(),
            signers: config_str.clone(),
            nonce,
        };

        let multisig_convert_data = MultiSigConvertToMultiSigUser {
            hyperliquidChain: convert_req.chain.clone(),
            payloadMultiSigUser: multi_sig_user,
            outerSigner: self.user,
            signers: config_str,
            nonce,
        };

        self.execute_chain_multi_sig_action(
            nonce,
            Actions::ConvertToMultiSigUser(convert_req),
            multisig_convert_data,
            CONVERT_TO_MULTI_SIG_USER_MULTISIG_TYPE.to_owned(),
            sig_chain_id,
            other_signers,
            multi_sig_user,
        )
        .await
    }

    /// Helper method to send exchange requests and handle responses
    async fn send_exchange_request(&self, payload: ExchangeRequest) -> Result<()> {
        debug!(
            "sending exchange request: {}",
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
            error!("exchange request failed: {} - {}", status_code, body);
            return Err(Errors::HyperLiquidApiError(status_code, body).into());
        }

        let out: ExchangeResponse = serde_json::from_str(body.as_str())?;
        debug!("exchange response: {:?}", out);

        if out.status != "ok" {
            return Err(Errors::HyperLiquidApiError(100, out.response.to_string()).into());
        }

        Ok(())
    }

    pub async fn multi_sig_l1_action(
        &self,
        action: Actions,
        sig_chain_id: String,
        other_signers: Vec<Signers>,
        multi_sig_user: Address,
    ) -> Result<()> {
        debug!("sending multi sig l1 action {:?}", action.clone());

        let nonce = self.nonce_manager.get_next_nonce();

        let is_mainnet = self.network == Network::Mainnet;
        let hash = generate_multi_sig_l1_hash(
            &action,
            multi_sig_user.to_string(),
            self.user.to_string(),
            is_mainnet,
            nonce,
        )?;

        let leader_signature = self.signer.sign_order(hash).await?;
        let mut signatures: Vec<SignedMessageHex> = vec![leader_signature.into()];

        for other in other_signers {
            let other_sig = other.sign_order(hash).await?;
            signatures.push(other_sig.into());
        }

        let multi_sig_payload: MultiSigRequest = MultiSigRequest {
            sig_chain_id,
            signatures,
            payload: MultiSigPayload {
                multi_sig_user: multi_sig_user.to_string().to_lowercase(),
                outer_signer: self.user.to_string().to_lowercase(),
                action: Box::new(action),
            },
        };

        let sig_hash = generate_multi_sig_hash(multi_sig_payload.clone(), self.network, nonce)?;
        let leader_outer_signature = self.signer.sign_order(sig_hash).await?;

        let payload = ExchangeRequest {
            nonce,
            signature: leader_outer_signature,
            action: serde_json::to_value(Actions::MultiSig(multi_sig_payload))?,
        };

        debug!(
            "sending multi sig l1: {}",
            serde_json::to_string(&payload).unwrap()
        );

        self.send_exchange_request(payload).await
    }
}
