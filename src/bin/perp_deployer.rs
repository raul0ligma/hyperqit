use alloy::primitives::Address;
use envconfig::Envconfig;
use hyperqit::*;

#[tokio::main]
async fn main() {
    env_logger::init();
    let config = hyperqit::Config::init_from_env().unwrap();
    let signer = Signers::Local(hyperqit::LocalWallet::signer(config.private_key));

    let user_address: Address = config.user_address.parse().unwrap();

    let executor = crate::HyperliquidClient::new(Network::Testnet, signer, user_address);

    let resp = executor
        .perp_deploy_action(PerpDeployAction::RegisterAsset(RegisterAsset {
            max_gas: None,
            asset_request: RegisterAssetRequest {
                coin: "dex:TICKER".into(),
                sz_decimals: 0,
                oracle_px: "69.0".into(),
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
