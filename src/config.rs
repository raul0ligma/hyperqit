use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "PRIVATE_KEY")]
    pub private_key: String,

    #[envconfig(from = "RUST_LOG")]
    pub log_level: String,
}
