use alloy::primitives::Address;

use hyperqit::*;

use envconfig::Envconfig;
use tracing_subscriber::EnvFilter;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "V2_DEPLOYER")]
    pub private_key: String,

    #[envconfig(from = "RUST_LOG")]
    pub log_level: String,

    #[envconfig(from = "V2_DEPLOYER_ADDR")]
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
    let signer = Box::new(hyperqit::LocalWallet::signer(config.private_key));

    let user_address: Address = config.user_address.parse().unwrap();

    println!("{:?}", signer.address());
    let executor = crate::HyperliquidClient::new(Network::Testnet, signer, user_address);

    let sz_decimals = 0;
    // executor
    //     .perp_deploy_action(PerpDeployAction::RegisterAsset(RegisterAsset {
    //         max_gas: None,
    //         asset_request: RegisterAssetRequest {
    //             coin: "gg:CORE".into(),
    //             sz_decimals: 3,
    //             oracle_px: format_significant_digits_and_decimals(
    //                 69.6969696996,
    //                 MAX_DECIMALS_PERP - sz_decimals,
    //             )
    //             .to_string(),
    //             margin_table_id: 20,
    //             only_isolated: true,
    //         },
    //         dex: "gg".into(),
    //         schema: Some(PerpDexSchemaInput {
    //             full_name: "hyperliquid blockchain is my bitch".into(),
    //             collateral_token: 0,
    //             oracle_updater: None,
    //         }),
    //     }))
    //     .await
    //     .unwrap();

    _ = executor
        .perp_deploy_action(PerpDeployAction::HaltTrading(HaltTrading {
            coin: "gg:CORE".to_string(),
            is_halted: true,
        }))
        .await
        .unwrap()

    // executor
    //     .perp_deploy_action(PerpDeployAction::SetSubDeployers(SetSubDeployer {
    //         dex: "dex".to_string(),
    //         sub_deployers: vec![SubDeployerInput {
    //             variant: "setOracle".to_string(),
    //             user: "0x187a265dC357C2E8d4681c300959893f9723a9d2"
    //                 .to_string()
    //                 .to_lowercase(),
    //             allowed: true,
    //         }],
    //     }))
    //     .await
    //     .unwrap();
}
