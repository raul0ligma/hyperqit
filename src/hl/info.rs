use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

pub type PerpetualsInfo = (UniverseInfo, Vec<PerpetualMetadata>);

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniverseInfo {
    pub universe: Vec<Universe>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Universe {
    pub name: String,
    pub sz_decimals: i64,
    pub max_leverage: i64,
    pub margin_table_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only_isolated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_delisted: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerpetualMetadata {
    pub day_ntl_vlm: String,
    pub funding: String,
    pub impact_pxs: Option<Vec<String>>,
    pub mark_px: String,
    pub mid_px: Option<String>,
    pub open_interest: String,
    pub oracle_px: String,
    pub premium: Option<String>,
    pub prev_day_px: String,
    pub day_base_vlm: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetInfoReq {
    #[serde(rename = "type")]
    pub asset_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dex: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpotResponse(pub SpotInfo, pub Vec<MarketData>);

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotInfo {
    pub universe: Vec<SpotUniverse>,
    pub tokens: Vec<Token>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub name: String,
    pub sz_decimals: i64,
    pub wei_decimals: i64,
    pub index: i64,
    pub token_id: String,
    pub is_canonical: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    pub deployer_trading_fee_share: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotUniverse {
    pub tokens: Vec<i64>,
    pub name: String,
    pub index: i64,
    pub is_canonical: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketData {
    pub prev_day_px: String,
    pub day_ntl_vlm: String,
    pub mark_px: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mid_px: Option<String>,
    pub circulating_supply: String,
    pub coin: String,
    pub total_supply: String,
    pub day_base_vlm: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnifiedMarketInfo {
    pub perp_markets: HashMap<String, PerpMarketInfo>,
    pub spot_markets: HashMap<String, SpotMarketInfo>,
    pub unified_markets: HashMap<String, CombinedMarketInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerpMarketInfo {
    pub asset_id: u32,
    pub name: String,
    pub mark_px: String,
    pub mid_px: Option<String>,
    pub funding: String,
    pub max_leverage: i64,
    pub sz_decimals: i64,
    pub oracle_px: String,
    pub open_interest: String,
    pub day_ntl_vlm: String,
    pub prev_day_px: String,
    pub margin_table_id: i64,
    pub only_isolated: Option<bool>,
    pub is_delisted: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpotMarketInfo {
    pub asset_id: u32,
    pub name: String,
    pub mark_px: String,
    pub mid_px: Option<String>,
    pub circulating_supply: String,
    pub total_supply: String,
    pub sz_decimals: i64,
    pub wei_decimals: i64,
    pub token_id: String,
    pub day_ntl_vlm: String,
    pub prev_day_px: String,
    pub deployer_trading_fee_share: String,
    pub is_canonical: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombinedMarketInfo {
    pub base_name: String,
    pub perp: Option<PerpMarketInfo>,
    pub spot: Option<SpotMarketInfo>,
}

pub fn create_unified_market_info(
    perp_info: PerpetualsInfo,
    spot_info: SpotResponse,
) -> UnifiedMarketInfo {
    let mut perp_markets = HashMap::new();
    let mut spot_markets = HashMap::new();
    let mut unified_markets = HashMap::new();

    process_perp_markets(perp_info, &mut perp_markets, &mut unified_markets);

    process_spot_markets(spot_info, &mut spot_markets, &mut unified_markets);

    UnifiedMarketInfo {
        perp_markets,
        spot_markets,
        unified_markets,
    }
}

fn process_perp_markets(
    perp_info: PerpetualsInfo,
    perp_markets: &mut HashMap<String, PerpMarketInfo>,
    unified_markets: &mut HashMap<String, CombinedMarketInfo>,
) {
    for (index, (universe_entry, perp_metadata)) in perp_info
        .0
        .universe
        .into_iter()
        .zip(perp_info.1.into_iter())
        .enumerate()
    {
        let perp_market = PerpMarketInfo {
            asset_id: index as u32,
            name: universe_entry.name.clone(),
            mark_px: perp_metadata.mark_px,
            mid_px: perp_metadata.mid_px,
            funding: perp_metadata.funding,
            max_leverage: universe_entry.max_leverage,
            sz_decimals: universe_entry.sz_decimals,
            oracle_px: perp_metadata.oracle_px,
            open_interest: perp_metadata.open_interest,
            day_ntl_vlm: perp_metadata.day_ntl_vlm,
            prev_day_px: perp_metadata.prev_day_px,
            margin_table_id: universe_entry.margin_table_id,
            only_isolated: universe_entry.only_isolated,
            is_delisted: universe_entry.is_delisted,
        };

        let base_name = extract_base_name(&universe_entry.name, true);
        perp_markets.insert(universe_entry.name.clone(), perp_market.clone());

        unified_markets
            .entry(base_name.clone())
            .or_insert_with(|| CombinedMarketInfo {
                base_name: base_name.clone(),
                perp: None,
                spot: None,
            })
            .perp = Some(perp_market);
    }
}

fn process_spot_markets(
    spot_info: SpotResponse,
    spot_markets: &mut HashMap<String, SpotMarketInfo>,
    unified_markets: &mut HashMap<String, CombinedMarketInfo>,
) {
    let market_data_map: HashMap<String, MarketData> = spot_info
        .1
        .into_iter()
        .map(|data| (data.coin.clone(), data))
        .collect();

    let universe_map: HashMap<i64, SpotUniverse> = spot_info
        .0
        .universe
        .into_iter()
        .filter_map(|universe| {
            if universe.tokens.len() >= 2 && universe.tokens[1] == 0 {
                let token_index = universe.tokens[0];
                Some((token_index, universe))
            } else {
                None
            }
        })
        .collect();

    for token in spot_info.0.tokens {
        if let Some(universe_entry) = universe_map.get(&token.index) {
            if let Some(market_data) = market_data_map.get(&universe_entry.name) {
                let spot_market = SpotMarketInfo {
                    asset_id: 10000 + universe_entry.index as u32,
                    name: token.name.clone(),
                    mark_px: market_data.mark_px.clone(),
                    mid_px: market_data.mid_px.clone(),
                    circulating_supply: market_data.circulating_supply.clone(),
                    total_supply: market_data.total_supply.clone(),
                    sz_decimals: token.sz_decimals,
                    wei_decimals: token.wei_decimals,
                    token_id: token.token_id.clone(),
                    day_ntl_vlm: market_data.day_ntl_vlm.clone(),
                    prev_day_px: market_data.prev_day_px.clone(),
                    deployer_trading_fee_share: token.deployer_trading_fee_share.clone(),
                    is_canonical: token.is_canonical,
                };

                let base_name = extract_base_name(&token.name, false);
                spot_markets.insert(token.name.clone(), spot_market.clone());

                unified_markets
                    .entry(base_name.clone())
                    .or_insert_with(|| CombinedMarketInfo {
                        base_name: base_name.clone(),
                        perp: None,
                        spot: None,
                    })
                    .spot = Some(spot_market);
            } else {
                debug!(
                    "No market data found for universe entry: {}",
                    universe_entry.name
                );
            }
        } else {
            debug!(
                "No matching universe entry found for token: {} (index: {})",
                token.name, token.index
            );
        }
    }
}

fn extract_base_name(market_name: &str, is_perp: bool) -> String {
    let delimiter = if is_perp { '-' } else { '/' };
    market_name
        .split(delimiter)
        .next()
        .unwrap_or(market_name)
        .to_string()
}

pub fn find_market_by_name<'a>(
    unified_info: &'a UnifiedMarketInfo,
    name: &str,
) -> Option<&'a CombinedMarketInfo> {
    if let Some(market) = unified_info.unified_markets.get(name) {
        return Some(market);
    }

    let name_lower = name.to_lowercase();
    unified_info
        .unified_markets
        .values()
        .find(|market| market.base_name.to_lowercase() == name_lower)
}

pub fn get_asset_id(unified_info: &UnifiedMarketInfo, name: &str, is_perp: bool) -> Option<u32> {
    find_market_by_name(unified_info, name).and_then(|market| {
        if is_perp {
            market.perp.as_ref().map(|p| p.asset_id)
        } else {
            market.spot.as_ref().map(|s| s.asset_id)
        }
    })
}

pub fn get_current_price(
    unified_info: &UnifiedMarketInfo,
    name: &str,
    is_perp: bool,
    use_mid: bool,
) -> Option<f64> {
    find_market_by_name(unified_info, name).and_then(|market| {
        let price_str = if is_perp {
            market.perp.as_ref().and_then(|perp| {
                if use_mid && perp.mid_px.is_some() {
                    perp.mid_px.as_ref()
                } else {
                    Some(&perp.mark_px)
                }
            })?
        } else {
            market.spot.as_ref().and_then(|spot| {
                if use_mid && spot.mid_px.is_some() {
                    spot.mid_px.as_ref()
                } else {
                    Some(&spot.mark_px)
                }
            })?
        };

        price_str.parse().ok()
    })
}
