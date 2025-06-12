use envconfig::Envconfig;
use hlmm::*;
use qrcode::QrCode;
use qrcode::render::unicode;

#[tokio::main]
async fn main() {
    env_logger::init();
    let config = hlmm::Config::init_from_env().unwrap();
    let signer = hlmm::AgentWallet::signer(config.private_key.into());
    signer.print_wallet();

    let executor = crate::HyperliquidClient::new(Network::Mainnet, signer);

    //executor.get_perp_info().await.unwrap();
    // executor.transfer_usd_to_spot(10).await.unwrap();
    // match executor.open_position().await {
    //     Err(err) => {
    //         panic!("failed {}", err.to_string())
    //     }
    //     Ok(_) => {
    //         println!("sucess")
    //     }
    // }

    let spotInfo = executor.get_spot_info().await.unwrap();
    let perpInfo = executor.get_perp_info().await.unwrap();
    let out = create_unified_market_info(perpInfo, spotInfo);
    let hype_data = find_market_by_name(&out, "HYPE");
    println!("{:?}", hype_data)
}
