use std::ops::Add;

use alloy::primitives::{Address, address};
use envconfig::Envconfig;
use futures::executor;
use hlmm::*;
use qrcode::QrCode;
use qrcode::render::unicode;

#[tokio::main]
async fn main() {
    env_logger::init();
    let config = hlmm::Config::init_from_env().unwrap();
    let signer = hlmm::AgentWallet::signer(config.private_key.into());
    //signer.print_wallet();

    let user_address: Address = config.user_address.parse().unwrap();

    let executor = crate::HyperliquidClient::new(Network::Testnet, signer, user_address);

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

    // let outA = executor.get_user_perp_info().await.unwrap();
    // let outB = executor.get_user_spot_info().await.unwrap();
    //println!("{:?} {:?}", outA, outB);
    let spotInfo = executor.get_spot_info().await.unwrap();
    let perpInfo = executor.get_perp_info().await.unwrap();
    let out = create_unified_market_info(perpInfo, spotInfo);
    let hype_data = find_market_by_name(&out, "BTC");
    println!("{:?}", hype_data);
    // let oid: i64 = config.existing_order_id.parse().unwrap();
    // executor.cancel_order(oid, 3).await.unwrap()

    let data = hype_data.unwrap().perp.clone().unwrap();
    let mid: f64 = data.mid_px.unwrap().parse().unwrap();
    let decimals = data.sz_decimals as i32;

    let size_in_usd = 505.65f64;
    executor
        .open_position(
            data.asset_id,
            true,
            true,
            mid,
            size_in_usd,
            false,
            0.001,
            decimals,
        )
        .await
        .unwrap()
}
