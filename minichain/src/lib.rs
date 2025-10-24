mod node;

use anyhow::anyhow;
use std::{env, net::SocketAddr, sync::Arc};
use tokio::sync::{Mutex, broadcast, mpsc};
use tokio::task;
use tokio::task::JoinHandle;

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
    let chain = Arc::new(Mutex::new(node::Blockchain::new()));
    let (tx_txn, rx_txn) = mpsc::channel(cfg.mempool_buf);
    let (tx_miner, rx_miner) = broadcast::channel(1);
    let mut handles: Vec<JoinHandle<()>> = vec![];
    let node_manager = node::NodeManager::new(&cfg, rx_miner);

    let bootnode = node_manager
        .get_node(cfg.boot_addr)
        .ok_or_else(|| anyhow!("Node {} not found", cfg.boot_addr))?;

    let map_mutex = Arc::new(Mutex::new(node::UserMap::new()));

    let mut users = map_mutex.lock().await;
    let admin_addr = users.add_user(User::new(Some(cfg.admin_addr.clone()), tx_txn.clone()))?;
    users.set_admin(admin_addr.clone(), true);
    let usr1_addr = users.add_user(User::new(None, tx_txn.clone()))?;
    users.fund_user(&admin_addr, &usr1_addr, 99999)?;

    let mempool = Arc::new(Mutex::new(node::Mempool::new(
        &admin_addr,
        rx_txn,
        tx_miner,
    )));

    let mut mp_mutex = mempool.clone();
    let mp_handle = task::spawn(async move {
        mp_mutex.lock().await.rcv_txs();
    });
    handles.push(mp_handle);

    let chain_mutex = chain.clone();

    let node_handle = tokio::spawn(async move {
        let mut bootlock = bootnode.lock().await;
        bootlock.wait_to_mine(chain_mutex).await;
    });

    handles.push(node_handle);

    for h in handles {
        let result = h.await.unwrap();
        println!("Task finished with: {:?}", result);
    }

    Ok(())
}
