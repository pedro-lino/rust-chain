mod node;
pub(crate) mod txs;

//use crate::txs;
use std::{env, net::SocketAddr};

pub struct Config {
    pub boot_addr: SocketAddr,
    pub admin_addr: String,
}

pub fn load_config() -> Config {
    let _ = dotenvy::dotenv();

    let bootnode_addr = env::var("BOOT_ADDR")
        .expect("BOOT_ADDR env not set.")
        .parse::<SocketAddr>()
        .expect("Invalid BOOTNODE address format");

    let admin_addr = env::var("ADMIN_ADDR").expect("ADMIN_ADDR env not set.");

    Config {
        boot_addr: bootnode_addr,
        admin_addr,
    }
}

pub fn run() {
    let cfg = load_config();
    let user_map = txs::UserMap::new(&cfg.admin_addr);
    let node_manager = node::NodeManager::new(&cfg, &user_map);
    let mempool = txs::Mempool::new(&cfg.admin_addr);
}
