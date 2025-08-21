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

    let _ = executor
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
}
