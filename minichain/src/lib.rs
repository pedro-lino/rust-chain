mod node;

use anyhow::anyhow;
use std::{env, net::SocketAddr, sync::Arc};
use tokio::sync::{Mutex, mpsc};
use tokio::task;

use crate::node::User;

pub struct Config {
    pub boot_addr: SocketAddr,
    pub admin_addr: String,
    pub mempool_buf: usize,
}

pub fn load_config() -> Config {
    let _ = dotenvy::dotenv();

    let boot_addr = env::var("BOOT_ADDR")
        .expect("BOOT_ADDR env not set.")
        .parse::<SocketAddr>()
        .expect("Invalid BOOTNODE address format");

    let admin_addr = env::var("ADMIN_ADDR").expect("ADMIN_ADDR env not set.");
    let mempool_buf = env::var("MEMPOOL_BUF")
        .expect("MEMPOOL_BUF env not set.")
        .parse::<usize>()
        .expect("Invalid BOOTNODE address format");

    Config {
        boot_addr,
        admin_addr,
        mempool_buf,
    }
}

pub async fn run() -> anyhow::Result<()> {
    let cfg = load_config();
    let node_manager: node::NodeManager;
    node_manager = node::NodeManager::new(&cfg);
    let blockchain = node::Blockchain::new();
    let (tx_mp, rx_mp) = mpsc::channel(cfg.mempool_buf);

    let bootnode = node_manager
        .get_node(cfg.boot_addr)
        .ok_or_else(|| anyhow!("Node {} not found", cfg.boot_addr))?;

    let map_mutex = Arc::new(Mutex::new(node::UserMap::new()));

    let mut users = map_mutex.lock().await;

    let admin_addr = users.add_user(User::new(Some(cfg.admin_addr.clone()), tx_mp.clone()))?;

    users.set_admin(admin_addr.clone(), true);

    let usr1_addr = users.add_user(User::new(None, tx_mp))?;

    users.fund_user(&admin_addr, &usr1_addr, 99999)?;

    println!(
        "User balance {}",
        users.get_user(&usr1_addr).unwrap().get_balance()
    );

    let mempool = node::Mempool::new(&admin_addr, rx_mp);
    let mp_handle = task::spawn(async move{
        mempool.
    })
    Ok(())
}
