use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "PRIVATE_KEY")]
    pub private_key: String,

    #[envconfig(from = "RUST_LOG")]
    pub log_level: String,

    #[envconfig(from = "USER_ADDRESS")]
    pub user_address: String,

    #[envconfig(from = "BOT_URL")]
    pub bot_url: String,

    #[envconfig(from = "CHECK_EVERY")]
    pub check_every: u64,

    #[envconfig(from = "BIND_ADDR")]
    pub bind_addr: String,
}
