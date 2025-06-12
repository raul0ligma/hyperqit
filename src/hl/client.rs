use std::time::SystemTime;

use alloy::dyn_abi::Eip712Domain;

use crate::errors::{Errors, Result};
use crate::hl::exchange::{
    ExchangeRequest, ExchangeResponse, generate_action_params, generate_transfer_params,
};
use crate::hl::info::{GetInfoReq, PerpetualsInfo, SpotInfo, SpotResponse};
use crate::hl::message::SignedMessage;
use crate::hl::{Actions, TransferRequest};
use crate::{HyperLiquidSigningHash, Order, OrderRequest};

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
}

impl<S> HyperliquidClient<S>
where
    S: HlAgentWallet,
{
    pub fn new(network: Network, signer: S) -> Self {
        HyperliquidClient {
            client: reqwest::Client::new(),
            signer,
            network,
        }
    }

    pub async fn open_position(&self) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis() as u64;

        let action: Actions = Actions::Order(crate::BulkOrder {
            orders: vec![OrderRequest {
                asset: 2,
                is_buy: true,
                limit_px: "112549".into(),
                sz: "0.0003".to_string(),
                reduce_only: false,
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
}
