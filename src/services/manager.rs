use std::sync::Arc;

use anyhow::Ok;
use serde::{Deserialize, Serialize};

use crate::{Amount, Asset, HlAgentWallet, Strategy, StrategyState, errors::Result};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenPositionRequest {
    value: Amount,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentUserPositionResponse {
    asset: Asset,
    signer: String,
    state: StrategyState,
}

pub struct StrategyManagerService {
    strategy: Arc<Strategy>,
    asset: Asset,
    signer: String,
}

impl StrategyManagerService {
    pub fn new(strategy: Arc<Strategy>, asset: Asset, signer: String) -> Self {
        Self {
            strategy,
            asset,
            signer,
        }
    }
    pub async fn open_position(
        &self,
        req: OpenPositionRequest,
    ) -> Result<CurrentUserPositionResponse> {
        self.strategy.enter(req.value).await?;
        let current_state = self.strategy.state().await?;
        Ok(CurrentUserPositionResponse {
            asset: self.asset.clone(),
            signer: self.signer.clone(),
            state: current_state,
        })
    }
    pub async fn current_user_position(&self) -> Result<CurrentUserPositionResponse> {
        let current_state = self.strategy.state().await?;
        Ok(CurrentUserPositionResponse {
            asset: self.asset.clone(),
            signer: self.signer.clone(),
            state: current_state,
        })
    }
    pub async fn close_position(&self) -> Result<String> {
        self.strategy.exit().await?;
        Ok("closed position".to_owned())
    }
}
