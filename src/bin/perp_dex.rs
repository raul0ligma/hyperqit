use alloy::primitives::Address;
use envconfig::Envconfig;
use hyperqit::*;
use tracing_subscriber::EnvFilter;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "PRIVATE_KEY")]
    pub private_key: String,

    #[envconfig(from = "RUST_LOG")]
    pub log_level: String,

    #[envconfig(from = "USER_ADDRESS")]
    pub user_address: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let config = Config::init_from_env().unwrap();
    let signer = Box::new(hyperqit::LocalWallet::signer(config.private_key));

    let user_address: Address = config.user_address.parse().unwrap();

    let executor = crate::HyperliquidClient::new(Network::Testnet, signer, user_address);

    executor
        .send_asset_to_dex(SendAssetRequest {
            chain: Network::Testnet.name(),
            sig_chain_id: "0xa4b1".to_string(),
            destination: user_address.to_string(),
            source_dex: "".to_string(),
            dst_dex: "dex".to_string(),
            amount: "10".to_string(),
            token: "USDC".to_string(),
            from_sub_account: "".to_string(),
            nonce: 0,
        })
        .await
        .unwrap();

    let perp_info = executor
        .get_perp_info(Some("dex".to_string()))
        .await
        .unwrap();
    println!("{:?}", perp_info);

    let user_info = executor
        .get_user_perp_info(Some("dex".to_string()))
        .await
        .unwrap();
    println!("{:?}", user_info);

    let response = executor.cancel_order(69, 1004).await.unwrap();
    println!("{:?}", response);

    let open_order = executor
        .get_user_open_orders(Some("dex".to_string()))
        .await
        .unwrap();
    println!("{:?}", open_order)
}
