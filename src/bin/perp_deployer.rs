use alloy::primitives::Address;

use hyperqit::*;

use envconfig::Envconfig;
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
        .json()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let config = Config::init_from_env().unwrap();
    let signer = Signers::Local(hyperqit::LocalWallet::signer(config.private_key));

    let user_address: Address = config.user_address.parse().unwrap();

    let executor = crate::HyperliquidClient::new(Network::Testnet, signer, user_address);

    let sz_decimals = 0;
    let resp = executor
        .perp_deploy_action(PerpDeployAction::RegisterAsset(RegisterAsset {
            max_gas: None,
            asset_request: RegisterAssetRequest {
                coin: "dex:TICKER".into(),
                sz_decimals: 0,
                oracle_px: format_significant_digits_and_decimals(
                    69.6969696996,
                    MAX_DECIMALS_PERP - sz_decimals,
                )
                .to_string(),
                margin_table_id: 5,
                only_isolated: true,
            },
            dex: "dex".into(),
            schema: Some(PerpDexSchemaInput {
                full_name: "dex-full-name".into(),
                collateral_token: 0,
                oracle_updater: None,
            }),
        }))
        .await
        .unwrap();
}
