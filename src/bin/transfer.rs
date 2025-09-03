use alloy::primitives::Address;
use envconfig::Envconfig;
use hyperqit::*;
use tracing_subscriber::EnvFilter;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "PRIVATE_KEY_SENDER")]
    pub private_key_sender: String,

    #[envconfig(from = "SENDER_ADDRESS")]
    pub sender_address: String,
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

    let signer: Signers = Signers::Local(hyperqit::LocalWallet::signer(config.private_key_sender));

    let user_address: Address = config.sender_address.parse().unwrap();

    let executor = crate::HyperliquidClient::new(Network::Testnet, signer, user_address);

    executor
        .transfer_usd(10, false, "0x1".to_owned())
        .await
        .unwrap();
}
