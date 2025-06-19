use crate::{
    Amount, AssetPosition, Balance, HyperliquidClient, PerpMarketInfo, SpotMarketInfo,
    StrategyStatus, create_unified_market_info,
    errors::{self, Result},
    find_market_by_name,
    strategy::{
        Asset::{self, CommonAsset, WithPerpAndSpot},
        Position, StrategyState,
    },
};
use std::time::{Duration, SystemTime};

use log::{info, warn};
use std::result::Result::Ok;
use tokio::time;
use tokio_util::sync::CancellationToken;

pub struct Strategy {
    asset: Asset,
    slippage: f64,
    liq_threshold: f64,
    leverage: u32,
    dust_threshold: f64,
    tick_interval: Duration,
    executor: HyperliquidClient,
}

impl Strategy {
    pub fn new(
        leverage: u32,
        tick_interval: Duration,
        asset: Asset,
        slippage: f64,
        dust_threshold: f64,
        liq_threshold: f64,
        executor: HyperliquidClient,
    ) -> Self {
        Strategy {
            asset,
            leverage,
            slippage,
            liq_threshold,
            tick_interval,
            dust_threshold,
            executor,
        }
    }

    pub async fn info(&self) -> Result<()> {
        match self.state().await {
            Ok(state) => match state.status {
                StrategyStatus::Active => {
                    if let Some(pos) = state.position {
                        info!(
                            "active | funding: {:.3}% | mark: {:.2} | liq_risk: {:.1}% | pnl: {:.9}",
                            pos.perp_funding_rate * 100.0,
                            pos.perp_mid_px,
                            if pos.liq_px > 0.0 {
                                (1.0 - pos.perp_mid_px / pos.liq_px) * 100.0
                            } else {
                                0.0
                            },
                            pos.funding_earning_nh
                        );
                    }
                }
                StrategyStatus::InActive => {
                    info!("inactive");
                }
            },
            Err(e) => {
                warn!("state check failed: {}", e);
            }
        }
        Ok(())
    }

    async fn check_health(&self) -> Result<bool> {
        let (perp_info, _) = self.get_market_data().await?;
        let user_state = self.state().await?;

        if user_state.status == StrategyStatus::InActive {
            info!("check: strategy inactive, no action needed");
            return Ok(false);
        }

        let current_funding_rate: f64 = perp_info.funding.parse()?;
        info!("check: funding rate {:.4}%", current_funding_rate * 100.0);

        if current_funding_rate < 0.0 {
            info!(
                "decision: funding negative ({:.4}%), exiting position",
                current_funding_rate * 100.0
            );
            return Ok(true);
        }

        let current_mark_px: f64 = perp_info.mark_px.parse()?;
        let user_pos = user_state.position.ok_or(errors::Errors::DataError(
            "user_state".to_owned(),
            "perp_position".to_owned(),
        ))?;

        let price_ratio = current_mark_px / user_pos.liq_px;
        info!(
            "check: price at {:.1}% of liq price (threshold {:.1}%)",
            price_ratio * 100.0,
            self.liq_threshold * 100.0
        );

        let should_exit = price_ratio >= self.liq_threshold;
        if should_exit {
            info!(
                "decision: price {:.1}% of liq price exceeds threshold {:.1}%, exiting position",
                price_ratio * 100.0,
                self.liq_threshold * 100.0
            );
        } else {
            info!("decision: conditions good, maintaining position");
        }

        Ok(should_exit)
    }

    pub async fn run(&self, cancellation: CancellationToken) -> Result<()> {
        info!("starting strategy runner");
        let mut ticker = time::interval(self.tick_interval);

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    let _ = self.info().await;

                    match self.check_health().await {
                        Ok(should_exit) => {
                            if should_exit {
                                match self.exit().await {
                                    Ok(_) => info!("exited position"),
                                    Err(e) => warn!("exit failed: {}", e),
                                }
                            }
                        }
                        Err(e) => {
                            warn!("health check failed: {}", e);
                        }
                    }
                }
                _ = cancellation.cancelled() => {
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
                .is_some_and(|spot| spot.total.parse::<f64>().unwrap_or(0f64) < self.dust_threshold)
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
                .is_some_and(|spot| spot.total.parse::<f64>().unwrap_or(0f64) < self.dust_threshold)
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
