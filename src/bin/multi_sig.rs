use std::str::FromStr;

use alloy::primitives::Address;
use envconfig::Envconfig;
use hyperqit::*;
use log::info;
use tracing_subscriber::EnvFilter;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "PRIVATE_KEY_OWNER")]
    pub private_key_owner: String,

    #[envconfig(from = "PRIVATE_KEY_A")]
    pub private_key_a: String,

    #[envconfig(from = "PRIVATE_KEY_B")]
    pub private_key_b: String,

    #[envconfig(from = "MULTI_SIG_ADDRESS")]
    pub multi_sig: String,

    #[envconfig(from = "RUST_LOG")]
    pub log_level: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let config = Config::init_from_env().unwrap();

    let user_a = Box::new(hyperqit::LocalWallet::signer(config.private_key_a));
    let user_b = Box::new(hyperqit::LocalWallet::signer(config.private_key_b));

    let user_a_addr = user_a.address();
    let user_b_addr = user_b.address();

    let signer = Box::new(hyperqit::LocalWallet::signer(config.private_key_owner));

    let multi_sig_user = Address::from_str(&config.multi_sig).unwrap();

    let core_executor = crate::HyperliquidClient::new(Network::Testnet, signer, multi_sig_user);
    core_executor
        .convert_to_multi_sig("0x66eee".to_string(), vec![user_a_addr, user_b_addr], 2)
        .await
        .unwrap();

    let multi_sig_config = core_executor
        .get_user_multi_sig_config(multi_sig_user)
        .await
        .unwrap();
    info!("config {:?}", multi_sig_config);

    let executor = crate::HyperliquidClient::new(Network::Testnet, user_b, user_b_addr);
    executor
        .multi_sig_convert_to_multisig_user(
            None,
            0,
            "0x66eee".to_string(),
            vec![user_a.clone()],
            multi_sig_user,
        )
        .await
        .unwrap();

    executor
        .multi_sig_usd_class_transfer(
            1,
            false,
            "0x66eee".to_string(),
            vec![user_a.clone()],
            multi_sig_user,
        )
        .await
        .unwrap();

    executor
        .multi_sig_send_asset(
            user_a_addr,
            "dex".to_string(),
            "".to_string(),
            "USDC".to_string(),
            "2.0".to_string(),
            None,
            "0x66eee".to_string(),
            vec![user_a.clone()],
            multi_sig_user,
        )
        .await
        .unwrap();

    executor
        .multi_sig_usd_send(
            user_a_addr,
            "2.0".to_string(),
            "0x66eee".to_string(),
            vec![user_a.clone()],
            multi_sig_user,
        )
        .await
        .unwrap();

    executor
        .multi_sig_l1_action(
            Actions::PerpDeploy(PerpDeployAction::SetOracle(SetOracle {
                dex: "dex".to_string(),
                oracle_pxs: vec![["dex:COIN".to_string(), "69.69".to_string()]],
                mark_pxs: vec![],
                external_perp_pxs: vec![],
            })),
            "0x66eee".to_string(),
            vec![user_a.clone()],
            multi_sig_user,
        )
        .await
        .unwrap();

    executor
        .multi_sig_l1_action(
            Actions::PerpDeploy(PerpDeployAction::SetFundingMultipliers(vec![[
                "dex:COIN".to_string(),
                "0".to_string(),
            ]])),
            "0x66eee".to_string(),
            vec![user_a.clone()],
            multi_sig_user,
        )
        .await
        .unwrap();

    executor
        .multi_sig_l1_action(
            Actions::PerpDeploy(PerpDeployAction::HaltTrading(HaltTrading {
                coin: "dex:BET".to_string(),
                is_halted: false,
            })),
            "0x66eee".to_string(),
            vec![user_a.clone()],
            multi_sig_user,
        )
        .await
        .unwrap();

    executor
        .multi_sig_l1_action(
            Actions::PerpDeploy(PerpDeployAction::SetOpenInterestCaps(vec![(
                "dex:COIN".to_string(),
                10000000000000,
            )])),
            "0x66eee".to_string(),
            vec![user_a.clone()],
            multi_sig_user,
        )
        .await
        .unwrap();

    executor
        .multi_sig_l1_action(
            Actions::PerpDeploy(PerpDeployAction::InsertMarginTable(InsertMarginTable {
                dex: "dex".to_string(),
                margin_table: RawMarginTable {
                    description: "insert margin table".to_string(),
                    margin_tiers: vec![
                        RawMarginTier {
                            lower_bound: 0,
                            max_leverage: 50,
                        },
                        RawMarginTier {
                            lower_bound: 500,
                            max_leverage: 20,
                        },
                        RawMarginTier {
                            lower_bound: 2500,
                            max_leverage: 10,
                        },
                    ],
                },
            })),
            "0x66eee".to_string(),
            vec![user_a.clone()],
            multi_sig_user,
        )
        .await
        .unwrap();

    executor
        .multi_sig_l1_action(
            Actions::PerpDeploy(PerpDeployAction::SetMarginTableIds(vec![(
                "dex:COIN".to_string(),
                1,
            )])),
            "0x66eee".to_string(),
            vec![user_a.clone()],
            multi_sig_user,
        )
        .await
        .unwrap();
}
