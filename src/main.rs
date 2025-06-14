use std::ops::Add;

use alloy::primitives::{Address, address};
use envconfig::Envconfig;
use ethers::providers::RetryClient;
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

    let spotInfo = executor.get_spot_info().await.unwrap();
    let perpInfo = executor.get_perp_info().await.unwrap();
    let out = create_unified_market_info(perpInfo, spotInfo);
    let hype_data = find_market_by_name(&out, "HYPE");
    println!("{:?}", hype_data);
    let perp_data = hype_data.unwrap().perp.clone().unwrap();
    let funding_rate: f64 = perp_data.funding.parse().unwrap();
    let size_in_usd = 50.69f64;
    if funding_rate <= 0.0 {
        println!("can't run strategy as funding rate is negative");
        return;
    }

    // execute a buy order on spot

    let spot_data = hype_data.unwrap().spot.clone().unwrap();
    println!("spot data {:?}", spot_data.clone());

    let spot_mid: f64 = spot_data.mid_px.unwrap().parse().unwrap();
    let spot_decimals = spot_data.sz_decimals as i32;

    executor
        .create_position_with_size_in_usd(
            spot_data.asset_id,
            false,
            true,
            spot_mid,
            size_in_usd,
            false,
            0.5,
            spot_decimals,
        )
        .await
        .unwrap();

    let mid: f64 = perp_data.mid_px.unwrap().parse().unwrap();
    let decimals = perp_data.sz_decimals as i32;

    // set leverage to 1x
    executor
        .update_leverage(perp_data.asset_id, true, 1)
        .await
        .unwrap();

    executor
        .create_position_with_size_in_usd(
            perp_data.asset_id,
            true,
            false,
            mid,
            size_in_usd,
            false,
            0.5,
            decimals,
        )
        .await
        .unwrap();
    // let oid: i64 = config.existing_order_id.parse().unwrap();
    // executor.cancel_order(oid, 3).await.unwrap()
    //let outA = executor.get_user_perp_info().await.unwrap();
    // let outB = executor.get_user_spot_info().await.unwrap();
    // println!("{:?} ", outA);
    // let pos = outA.asset_positions.get(0).unwrap();
    // let (is_buy, close_sz) = pos.position.get_close_order_info().unwrap();

    // println!("closing pos  {} {}", is_buy, close_sz);
    // executor
    //     .create_position_with_size(
    //         data.asset_id,
    //         true,
    //         is_buy,
    //         mid,
    //         close_sz + 5.0,
    //         true,
    //         0.001,
    //         decimals,
    //     )
    //     .await
    //     .unwrap();
}
