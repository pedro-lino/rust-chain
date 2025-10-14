mod node;
pub(crate) mod txs;

use std::env;

pub struct Config {
    pub bootnode: String,
}

pub fn load_config() -> Config {
    let _ = dotenvy::dotenv();

    let bootnode = env::var("BOOTNODE").expect("Bootnode env not set.");
    Config { bootnode }
}

pub fn run() {
    let cfg = load_config();
    node::NodeManager::new(&cfg.bootnode.parse().expect("Invalid socket address"));
}
