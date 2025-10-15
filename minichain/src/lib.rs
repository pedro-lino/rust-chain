mod node;
pub(crate) mod txs;

//use crate::txs;
use std::{env, net::SocketAddr};

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
    let user_map = txs::UserMap::new();

    node::NodeManager::new(cfg.bootnode.parse().expect("Invalid socket address"));
}

fn launch_node_manager(boot_ip: SocketAddr, user: txs::User) {
    node::Node::new(&user, boot_ip);
}
