use crate::errors::Result;

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

pub const MAX_SIGNIFICANT_DIGITS: i32 = 5i32;
pub const MAX_DECIMALS_SPOT: i32 = 8i32;
pub const MAX_DECIMALS_PERP: i32 = 6i32;

pub fn format_decimals(v: f64, decimals: i32) -> f64 {
    let decimal_shift = 10f64.powi(decimals);

    (v * decimal_shift).round() / decimal_shift
}

pub fn format_significant_digits_and_decimals(v: f64, decimals: i32) -> f64 {
    // m is magnitude,
    let m = v.abs().log10().floor() as i32;
    let scale = 10f64.powi(MAX_SIGNIFICANT_DIGITS - m - 1);
    let shifted = (v * scale).round() / scale;
    format_decimals(shifted, decimals)
}

pub fn get_formatted_position_with_amount(
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

pub fn get_formatted_position_with_amount_raw(
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

pub fn parse_chain_id(chain_id: &str) -> Result<u64> {
    if let Some(stripped) = chain_id.strip_prefix("0x") {
        u64::from_str_radix(stripped, 16)
    } else if chain_id.chars().all(|c| c.is_ascii_hexdigit()) {
        u64::from_str_radix(chain_id, 16)
    } else {
        chain_id.parse::<u64>()
    }
    .map_err(|e| anyhow::anyhow!("Invalid chain ID format: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::Result;

    #[test]
    fn test_fmt() -> Result<()> {
        let result = format_significant_digits_and_decimals(696969.6969696969, MAX_DECIMALS_PERP);
        println!("format test result: {}", result);
        Ok(())
    }
}
