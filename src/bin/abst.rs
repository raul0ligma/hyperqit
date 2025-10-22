use alloy::primitives::Address;

use hyperqit::*;

use envconfig::Envconfig;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "PRIVATE_KEY")]
    pub private_key: String,

    #[envconfig(from = "RUST_LOG")]
    pub log_level: String,

    #[envconfig(from = "USER_ADDRESS")]
    pub user_address: String,

    #[envconfig(from = "EXISTING_ORDER_ID")]
    pub existing_order_id: String,

    #[envconfig(from = "BOT_URL")]
    pub bot_url: String,

    #[envconfig(from = "CHECK_EVERY")]
    pub check_every: u64,

    #[envconfig(from = "BIND_ADDR")]
    pub bind_addr: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let config = Config::init_from_env().unwrap();
    let signer = Box::new(hyperqit::LocalWallet::signer(config.private_key));

    let user_address: Address = config.user_address.parse().unwrap();

    let executor = crate::HyperliquidClient::new(Network::Mainnet, signer, user_address);

    let sz_decimals = 4;
    let asset_id = 110000;

    let main_dex_info = executor.get_user_perp_info(None).await.unwrap();
    let other_dex_info = executor
        .get_user_perp_info(Some("xyz".into()))
        .await
        .unwrap();

    let mut bal_main: f64 = main_dex_info.withdrawable.parse().unwrap();
    let mut bal_other: f64 = other_dex_info.withdrawable.parse().unwrap();

    info!("balances main: {} xyz: {}", bal_main, bal_other);

    executor
        .update_dex_abstraction("0xa4b1".to_string(), false)
        .await
        .unwrap();
    let abs_state_2 = executor.get_dex_abstraction().await.unwrap();
    info!("{:?}", abs_state_2);

    if bal_other < 2.0 {
        _ = executor
            .send_asset_to_dex(SendAssetRequest {
                sig_chain_id: "0xa4b1".to_string(),
                chain: Network::Mainnet.name(),
                destination: user_address.to_string(),
                source_dex: "".into(),
                dst_dex: "xyz".into(),
                token: "USDC".into(),
                amount: format!("{:.2}", bal_main / 2.0),
                from_sub_account: "".into(),
                nonce: 0,
            })
            .await
            .unwrap();
        let main_dex_info = executor.get_user_perp_info(None).await.unwrap();
        let other_dex_info = executor
            .get_user_perp_info(Some("xyz".into()))
            .await
            .unwrap();
        bal_main = main_dex_info.withdrawable.parse().unwrap();
        bal_other = other_dex_info.withdrawable.parse().unwrap();
        info!("rebalanced balances main: {} xyz: {}", bal_main, bal_other);
    }

    // set a 20x leverage
    executor.update_leverage(asset_id, false, 20).await.unwrap();
    let lev = 20.0;

    let double_margin = 8.0 * bal_other;

    let info = executor.get_perp_info(Some("xyz".into())).await.unwrap();
    let mid_px: f64 = info
        .1
        .get(0)
        .unwrap()
        .mid_px
        .clone()
        .unwrap()
        .parse()
        .unwrap();

    let sz = (lev * double_margin / mid_px);

    let resp = executor
        .create_position_raw(BulkOrder {
            orders: vec![OrderRequest {
                asset: asset_id,
                is_buy: true,
                // add a 50% slippage
                limit_px: format_significant_digits_and_decimals(mid_px * 1.05, sz_decimals)
                    .to_string(),
                sz: format_decimals(sz, sz_decimals).to_string(),
                reduce_only: false,
                order_type: OrderType::Limit(Limit { tif: "Ioc".into() }),
                cloid: None,
            }],
            grouping: "na".into(),
        })
        .await
        .unwrap();
    println!("{:?}", resp)
}
