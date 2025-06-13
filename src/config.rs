use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "PRIVATE_KEY")]
    pub private_key: String,

    #[envconfig(from = "RUST_LOG")]
    pub log_level: String,

    #[envconfig(from = "USER_ADDRESS")]
    pub user_address: String,

    #[envconfig(from = "EXISTING_ORDER_ID")]
    pub existing_order_id: String,
}
