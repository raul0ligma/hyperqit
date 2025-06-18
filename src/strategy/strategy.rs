use crate::{
    Amount, AssetPosition, Balance, HlAgentWallet, HyperliquidClient, PerpMarketInfo,
    SpotMarketInfo, StrategyStatus, create_unified_market_info,
    errors::{self, Result},
    find_market_by_name,
    strategy::{
        Asset::{self, CommonAsset, WithPerpAndSpot},
        Position, StrategyState,
    },
};
use std::time::{Duration, SystemTime};

use anyhow::Ok;
use log::{info, warn};
use tokio::time;
use tokio_util::sync::CancellationToken;
pub struct Strategy<S>
where
    S: HlAgentWallet,
{
    asset: Asset,
    slippage: f64,
    leverage: u32,
    dust_threshold: f64,
    tick_interval: Duration,
    executor: HyperliquidClient<S>,
}

impl<S> Strategy<S>
where
    S: HlAgentWallet,
{
    pub fn new(
        leverage: u32,
        tick_interval: Duration,
        asset: Asset,
        slippage: f64,
        dust_threshold: f64,
        executor: HyperliquidClient<S>,
    ) -> Self {
        Strategy {
            asset,
            leverage,
            slippage,
            tick_interval,
            dust_threshold,
            executor,
        }
    }

    pub async fn run(&self, cancellation: CancellationToken) -> Result<()> {
        info!("starting strategy runner");
        let mut ticker = time::interval(self.tick_interval);

        loop {
            tokio::select! {
                tick_out = ticker.tick() =>{

                }
                _ = cancellation.cancelled() =>{
                        warn!("cancelling strategy runner");
                        break;
                }
            }
        }

        Ok(())
    }

    pub async fn get_market_data(&self) -> Result<(PerpMarketInfo, SpotMarketInfo)> {
        let unified_info = create_unified_market_info(
            self.executor.get_perp_info().await?,
            self.executor.get_spot_info().await?,
        );
        return match &self.asset {
            CommonAsset(key) => {
                let common_info = find_market_by_name(&unified_info, key.as_str()).ok_or(
                    errors::Errors::DataError("unified_info".to_owned(), key.to_string()),
                )?;
                Ok((
                    common_info.perp.clone().ok_or(errors::Errors::DataError(
                        "perp_info_from_common".to_owned(),
                        key.to_string(),
                    ))?,
                    common_info.spot.clone().ok_or(errors::Errors::DataError(
                        "spot_info_from_common".to_owned(),
                        key.to_string(),
                    ))?,
                ))
            }
            WithPerpAndSpot(perp_asset_name, spot_asset_name) => {
                let perp_item = unified_info
                    .perp_markets
                    .get(perp_asset_name.as_str())
                    .ok_or(errors::Errors::DataError(
                        "from_perp_info".to_owned(),
                        perp_asset_name.to_string(),
                    ))?;
                let spot_item = unified_info
                    .spot_markets
                    .get(spot_asset_name.as_str())
                    .ok_or(errors::Errors::DataError(
                        "from_spot_info".to_owned(),
                        spot_asset_name.to_string(),
                    ))?;
                Ok((perp_item.clone(), spot_item.clone()))
            }
        };
    }

    async fn user_state(
        &self,
        perp_info: &PerpMarketInfo,
        spot_info: &SpotMarketInfo,
    ) -> Result<(Option<AssetPosition>, Option<Balance>)> {
        let user_perp = self.executor.get_user_perp_info().await?;
        let user_spot = self.executor.get_user_spot_info().await?;
        let current_spot_pos = user_spot
            .balances
            .iter()
            .find(|item| item.coin == spot_info.name)
            .cloned();

        let info_name = perp_info.clone().name;
        let current_perp_pos = user_perp
            .asset_positions
            .iter()
            .find(|item| item.position.coin == info_name)
            .cloned();
        Ok((current_perp_pos, current_spot_pos))
    }

    pub async fn state(&self) -> Result<StrategyState> {
        let (perp_info, spot_info) = self.get_market_data().await?;
        let perp_key = perp_info.name.clone();
        let spot_key = spot_info.asset_id.clone().to_string();
        let (current_perp_pos, current_spot_pos) = self.user_state(&perp_info, &spot_info).await?;

        if current_spot_pos.is_none()
            || current_perp_pos.is_none()
            || current_spot_pos
                .clone()
                .is_some_and(|spot| spot.total.parse::<f64>().unwrap_or(0f64) > self.dust_threshold)
        {
            return Ok(StrategyState {
                status: super::StrategyStatus::InActive,
                position: None,
            });
        }
        let perp_pos = current_perp_pos.ok_or(errors::Errors::DataError(
            "user_perp_pos".to_owned(),
            perp_key,
        ))?;
        let spot_pos = current_spot_pos.ok_or(errors::Errors::DataError(
            "user_spot_pos".to_owned(),
            spot_key,
        ))?;
        let perp_price: f64 = perp_info
            .clone()
            .mid_px
            .unwrap_or(perp_info.clone().mark_px)
            .parse()?;
        let spot_price: f64 = spot_info
            .clone()
            .mid_px
            .unwrap_or(spot_info.clone().mark_px)
            .parse()?;
        let perp_and_spot_diff: f64 = perp_price - spot_price;

        let user_funding_since = self
            .executor
            .get_user_funding_history(24 * 60 * 60 * 1000)
            .await?
            .iter()
            .fold(0 as f64, |val, cur| {
                val + cur.delta.usdc.parse().unwrap_or(0 as f64)
            });

        let pos = Position {
            perp_amount: perp_pos.position.szi.parse()?,
            perp_mid_px: perp_info
                .clone()
                .mid_px
                .unwrap_or(perp_info.clone().mark_px)
                .parse()?,
            perp_pos_usd: perp_pos.position.position_value.parse()?,
            liq_px: perp_pos
                .position
                .clone()
                .liquidation_px
                .unwrap_or("0".to_string())
                .parse()?,
            spot_amount: spot_pos.total.parse()?,
            spot_mid_px: spot_info
                .clone()
                .mid_px
                .unwrap_or(spot_info.clone().mark_px)
                .parse()?,
            spot_pos_usd: spot_pos.entry_ntl.parse()?,
            perp_funding_rate: perp_info.funding.parse()?,
            dn_diff: perp_and_spot_diff,
            funding_earning_nh: user_funding_since,
            at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_millis() as u64,
        };
        Ok(StrategyState {
            status: super::StrategyStatus::Active,
            position: Some(pos),
        })
    }

    pub async fn exit(&self) -> Result<()> {
        let (perp_info, spot_info) = self.get_market_data().await?;
        let perp_key = perp_info.name.clone();
        let spot_key = spot_info.asset_id.clone().to_string();
        let (current_perp_pos, current_spot_pos) = self.user_state(&perp_info, &spot_info).await?;
        if current_spot_pos.is_none()
            || current_perp_pos.is_none()
            || current_spot_pos
                .clone()
                .is_some_and(|spot| spot.total.parse::<f64>().unwrap_or(0f64) > self.dust_threshold)
        {
            return Err(errors::CmpError {
                expected: StrategyStatus::Active,
                actual: StrategyStatus::InActive,
            }
            .into());
        }
        let perp_pos = current_perp_pos.ok_or(errors::Errors::DataError(
            "user_perp_pos".to_owned(),
            perp_key,
        ))?;
        let spot_pos = current_spot_pos.ok_or(errors::Errors::DataError(
            "user_spot_pos".to_owned(),
            spot_key,
        ))?;
        let (perp_info, spot_info) = self.get_market_data().await?;
        let spot_mid: f64 = spot_info
            .mid_px
            .unwrap_or(perp_info.clone().mark_px)
            .parse()?;

        let spot_decimals = spot_info.sz_decimals as i32;

        let perp_mid: f64 = perp_info.mid_px.unwrap_or(perp_info.mark_px).parse()?;
        let perp_decimals = perp_info.sz_decimals as i32;
        let (is_perp_buy, perp_sz) = perp_pos.position.get_close_order_info()?;
        self.executor
            .create_position_with_size(
                perp_info.asset_id,
                true,
                is_perp_buy,
                perp_mid,
                perp_sz,
                true,
                self.slippage,
                perp_decimals,
            )
            .await
            .map_err(|err| errors::Errors::PlaceOrderError(err.to_string()))?;

        self.executor
            .create_position_with_size(
                spot_info.asset_id,
                false,
                false,
                spot_mid,
                spot_pos.total.parse()?,
                false,
                self.slippage,
                spot_decimals,
            )
            .await
            .map_err(|err| errors::Errors::PlaceOrderError(err.to_string()))?;

        Ok(())
    }
    pub async fn enter(&self, amount: Amount) -> Result<()> {
        let existing = self.state().await?;
        if existing.status == StrategyStatus::Active {
            return Err(errors::CmpError {
                expected: StrategyStatus::InActive,
                actual: existing.status,
            }
            .into());
        }

        let (perp_info, spot_info) = self.get_market_data().await?;
        let current_rate: f64 = perp_info.funding.parse()?;
        if current_rate < 0.0 {
            return Err(errors::Errors::FundRateNegative(current_rate).into());
        }

        let is_size_usd = matches!(amount, Amount::Usd(_));
        let sz: f64 = match amount {
            Amount::Raw(v) | Amount::Usd(v) => v.parse()?,
        };

        let spot_mid: f64 = spot_info
            .mid_px
            .unwrap_or(perp_info.clone().mark_px)
            .parse()?;

        let spot_decimals = spot_info.sz_decimals as i32;

        let perp_mid: f64 = perp_info.mid_px.unwrap_or(perp_info.mark_px).parse()?;
        let perp_decimals = perp_info.sz_decimals as i32;
        self.executor
            .update_leverage(perp_info.asset_id, true, self.leverage)
            .await?;
        if is_size_usd {
            self.executor
                .create_position_with_size_in_usd(
                    spot_info.asset_id,
                    false,
                    true,
                    spot_mid,
                    sz,
                    false,
                    self.slippage,
                    spot_decimals,
                )
                .await
                .map_err(|err| errors::Errors::PlaceOrderError(err.to_string()))?;

            self.executor
                .create_position_with_size_in_usd(
                    perp_info.asset_id,
                    true,
                    false,
                    perp_mid,
                    sz,
                    false,
                    self.slippage,
                    perp_decimals,
                )
                .await
                .map_err(|err| errors::Errors::PlaceOrderError(err.to_string()))?;
        } else {
            self.executor
                .create_position_with_size(
                    spot_info.asset_id,
                    false,
                    true,
                    spot_mid,
                    sz,
                    false,
                    self.slippage,
                    spot_decimals,
                )
                .await
                .map_err(|err| errors::Errors::PlaceOrderError(err.to_string()))?;

            self.executor
                .create_position_with_size(
                    perp_info.asset_id,
                    true,
                    false,
                    perp_mid,
                    sz,
                    false,
                    self.slippage,
                    perp_decimals,
                )
                .await
                .map_err(|err| errors::Errors::PlaceOrderError(err.to_string()))?;
        }

        Ok(())
    }
}
