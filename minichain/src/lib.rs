pub mod node;

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
    let mut handles: Vec<JoinHandle<Result<(), anyhow::Error>>> = vec![];
    let node_manager = node::NodeManager::new(&cfg, rx_miner);

    let bootnode = node_manager
        .get_node(cfg.boot_addr)
        .ok_or_else(|| anyhow!("Node {} not found", cfg.boot_addr))?;

    let map_mutex = Arc::new(Mutex::new(node::UserMap::new()));

    let mut users = map_mutex.lock().await;

    println!("Creating and setting up users...");
    // Create admin and user accounts
    let admin_addr = users.add_user(User::new(Some(cfg.admin_addr.clone()), tx_txn.clone()))?;
    users.set_admin(admin_addr.clone(), true);

    let user1_addr = users.add_user(User::new(None, tx_txn.clone()))?;
    let user2_addr = users.add_user(User::new(None, tx_txn.clone()))?;

    // Fund the users
    println!("Funding users from admin account...");
    users.fund_user(&admin_addr, &user1_addr, 1000)?;
    users.fund_user(&admin_addr, &user2_addr, 2000)?;

    println!("Initial balances:");
    println!(
        "User 1: {} coins",
        users.get_user(&user1_addr).unwrap().get_balance()
    );
    println!(
        "User 2: {} coins",
        users.get_user(&user2_addr).unwrap().get_balance()
    );

    // Create and send transactions
    println!("\nSending transactions...");
    let test_data = "test transaction".as_bytes().to_vec().into_boxed_slice();

    if let Some(user) = users.get_user_mut(&user1_addr) {
        user.send_tx(test_data.clone(), 50).await?;
        println!("User 1 sent transaction with 50 gas");
    }

    if let Some(user) = users.get_user_mut(&user2_addr) {
        user.send_tx(test_data.clone(), 100).await?;
        println!("User 2 sent transaction with 100 gas");
    }

    // Drop users lock to avoid deadlock
    drop(users);

    let mempool = Arc::new(Mutex::new(node::Mempool::new(
        &admin_addr,
        rx_txn,
        tx_miner,
    )));

    let mp_mutex = mempool.clone();
    let mp_handle = task::spawn(async move {
        mp_mutex.lock().await.rcv_txs().await;
        Ok::<(), anyhow::Error>(())
    });
    handles.push(mp_handle);

    let chain_mutex = chain.clone();
    let node_handle = tokio::spawn(async move {
        let mut bootlock = bootnode.lock().await;
        bootlock.wait_to_mine(chain_mutex).await?;
        Ok::<(), anyhow::Error>(())
    });
    handles.push(node_handle);

    // Wait for transactions to be processed and mined
    println!("\nWaiting for transactions to be mined...");
    tokio::time::sleep(std::time::Duration::from_secs(65)).await;

    // Check final chain state
    let chain_state = chain.lock().await;
    println!("\nFinal blockchain state:");
    println!("Chain height: {}", chain_state.get_height());

    if let Some(last_block) = chain_state.blocks.last() {
        println!("\nLast block contents:");
        println!("Block number: {}", last_block.number);
        println!("Number of transactions: {}", last_block.txs.len());
        println!("\nTransactions:");
        for (i, tx) in last_block.txs.iter().enumerate() {
            println!("Transaction {}:", i + 1);
            println!("  - Sender: {}", tx.sender);
            println!("  - Gas: {}", tx.gas);
            println!("  - ID: {}", tx.id);
        }
    }

    // Cleanup handles
    for h in handles {
        let _ = h.await?;
    }

    Ok(())
}
