mod node;

use anyhow::anyhow;
use std::{env, net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;

use crate::node::User;

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

pub async fn run() -> anyhow::Result<()> {
    let cfg = load_config();
    let node_manager: node::NodeManager;
    node_manager = node::NodeManager::new(&cfg);
    let mempool = node::Mempool::new(&cfg.admin_addr);
    let blockchain = node::Blockchain::new();

    let bootnode = node_manager
        .get_node(cfg.boot_addr)
        .ok_or_else(|| anyhow!("Node {} not found", &cfg.boot_addr))?;

    let usr1 = User::new();
    let addr1 = usr1.address.clone();

    let map_mutex = Arc::new(Mutex::new(node::UserMap::new(&cfg.admin_addr)));
    let mut usr_map = map_mutex.lock().await;

    usr_map.add_user(usr1)?;

    println!(
        "user balance is {}",
        usr_map.get_user(&addr1).unwrap().balance
    );

    usr_map.fund_user(&cfg.admin_addr, &addr1, 99999)?;

    println!(
        "user balance is {}",
        usr_map.get_user(&addr1).unwrap().balance
    );

    Ok(())
}
