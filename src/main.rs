use hlmm::*;

#[tokio::main]
async fn main() {
    env_logger::init();
    let signer = hlmm::AgentWallet::signer("0xa".into());

    let executor = crate::HyperliquidClient::new(Network::Testnet, signer);

    executor.transfer_usd_to_spot(10).await.unwrap();
    // match executor.open_position().await {
    //     Err(err) => {
    //         panic!("failed {}", err.to_string())
    //     }
    //     Ok(_) => {
    //         println!("sucess")
    //     }
    // }
}
