use alloy::primitives::Address;
use envconfig::Envconfig;
use hyperqit::*;
use tracing_subscriber::EnvFilter;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "PRIVATE_KEY_OWNER")]
    pub private_key_owner: String,

    #[envconfig(from = "PRIVATE_KEY_A")]
    pub private_key_a: String,

    #[envconfig(from = "PRIVATE_KEY_B")]
    pub private_key_b: String,

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

    let user_a = hyperqit::LocalWallet::signer(config.private_key_a);
    let user_b = hyperqit::LocalWallet::signer(config.private_key_b);

    let user_a_addr = user_a.address();
    let user_b_addr = user_b.address();

    let signer: Signers = Signers::Local(hyperqit::LocalWallet::signer(config.private_key_owner));

    let user_address: Address = config.user_address.parse().unwrap();

    let executor = crate::HyperliquidClient::new(Network::Testnet, signer, user_address);

    let _ = executor
        .convert_to_multi_sig("0x01".to_string(), vec![user_a_addr, user_b_addr], 2)
        .await
        .unwrap();
}
