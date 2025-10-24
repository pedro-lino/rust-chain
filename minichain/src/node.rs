use crate::Config;
use anyhow::{Ok, anyhow};
use anyhow::{Result, bail, ensure};
use rand::{RngCore, rng};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::hash_map::Entry;
use std::fmt::Write;
use std::net::SocketAddr;
use std::ops::AddAssign;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::select;
use tokio::sync::{Mutex, broadcast, mpsc};

type Address = String;
type Hash = String;

pub struct User {
    address: Address,
    balance: u128,
    tx_txn: mpsc::Sender<Transaction>,
}
pub struct UserMap {
    users: HashMap<Address, User>,
    admins: HashSet<Address>,
}

pub struct Node {
    owner_addr: Address,
    ip_address: SocketAddr,
    rx_miner: broadcast::Receiver<(Address, Vec<Transaction>)>,
}

pub struct NodeManager {
    map: HashMap<SocketAddr, Arc<Mutex<Node>>>,
}

#[derive(Clone)]
pub struct Transaction {
    id: Hash,
    gas: u128,
    data: Box<[u8]>,
    sender: Address,
}

pub struct Block {
    number: u128,
    timestamp: SystemTime,
    txs: Vec<Transaction>,
}

pub struct Blockchain {
    blocks: Vec<Block>,
}

pub struct Mempool {
    txs: BinaryHeap<Transaction>,
    miner: Address,
    rx_txn: mpsc::Receiver<Transaction>,
    tx_mine: broadcast::Sender<(Address, Vec<Transaction>)>,
}

impl User {
    pub fn new(addr: Option<Address>, tx_sender: mpsc::Sender<Transaction>) -> Self {
        let address = addr.unwrap_or_else(|| random_address());

        User {
            address,
            balance: 0,
            tx_txn: tx_sender,
        }
    }

    pub fn get_address(&self) -> &Address {
        &self.address
    }
    pub fn get_balance(&self) -> &u128 {
        &self.balance
    }

    async fn send_tx(&mut self, data: Box<[u8]>, gas: u128) -> Result<()> {
        ensure!(
            self.balance >= gas,
            "Sender doesn't have enough funds to pay for gas."
        );

        self.balance -= gas;

        self.tx_txn
            .send(Transaction {
                id: random_address(),
                gas,
                data,
                sender: self.address.clone(),
            })
            .await?;
        Ok(())
    }
}

impl<'a> UserMap {
    pub fn new() -> Self {
        let users: HashMap<Address, User> = HashMap::new();
        let admins: HashSet<Address> = HashSet::new();
        UserMap { users, admins }
    }

    pub fn add_user(&mut self, usr: User) -> Result<Address> {
        match self.users.entry(usr.address.clone()) {
            Entry::Vacant(v) => {
                let addr = usr.address.clone();
                v.insert(usr);
                Ok(addr)
            }
            Entry::Occupied(..) => bail!("User was already registered"),
        }
    }

    pub fn set_admin(&mut self, addr: Address, is_admin: bool) {
        if is_admin {
            self.admins.insert(addr);
        } else {
            self.admins.remove(&addr);
        }
    }

    pub fn get_user(&mut self, addr: &String) -> Option<&User> {
        self.users.get(addr)
    }
    pub fn is_admin(&self, addr: &String) -> bool {
        self.admins.contains(addr)
    }

    pub fn fund_user(
        &mut self,
        funder: &String,
        funded: &String,
        amount: u128,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(self.is_admin(funder), "Funder is not an admin.");
        self.users.get_mut(funded).unwrap().balance += amount;
        Ok(())
    }
}

impl Node {
    fn new(
        owner_addr: Address,
        ip_address: SocketAddr,
        rx_miner: broadcast::Receiver<(Address, Vec<Transaction>)>,
    ) -> Self {
        Node {
            owner_addr: owner_addr,
            ip_address,
            rx_miner,
        }
    }

    pub async fn wait_to_mine(&mut self, chain: Arc<Mutex<Blockchain>>) -> anyhow::Result<()> {
        loop {
            match self.rx_miner.recv().await {
                Result::Ok(n) if n.0 == self.owner_addr => {
                    self.execute_txs(chain, n.1).await?;
                    break;
                }
                Result::Ok(_) => continue,
                Err(e) => return Err(anyhow::anyhow!(e)),
            }
        }
        bail!("Error waiting to mine...")
    }

    pub async fn execute_txs(
        &self,
        chain: Arc<Mutex<Blockchain>>,
        txs: Vec<Transaction>,
    ) -> anyhow::Result<()> {
        let mut chain_lock = chain.lock().await;

        let mut b = Block {
            number: chain_lock.get_height(),
            timestamp: SystemTime::now(),
            txs: Vec::new(),
        };

        for tx in txs {
            b.txs.push(tx);
        }

        chain_lock.blocks.push(b);
        Ok(())
    }
}

impl NodeManager {
    pub fn new(cfg: &Config, rx_miner: broadcast::Receiver<(Address, Vec<Transaction>)>) -> Self {
        let bootnode = Arc::new(Mutex::new(Node::new(
            cfg.admin_addr.clone(),
            cfg.boot_addr,
            rx_miner,
        )));

        let mut map: HashMap<SocketAddr, Arc<Mutex<Node>>> = HashMap::new();
        map.insert(cfg.boot_addr, bootnode);

        NodeManager { map: map }
    }

    pub fn get_node(&self, ip: SocketAddr) -> Option<Arc<Mutex<Node>>> {
        self.map.get(&ip).cloned()
    }
}

impl Transaction {}

impl Blockchain {
    pub fn new() -> Self {
        let now = SystemTime::now();

        let b = Block {
            number: 0,
            timestamp: now,
            txs: Vec::new(),
        };

        Blockchain { blocks: vec![b] }
    }

    pub fn get_height(&self) -> u128 {
        self.blocks.len() as u128
    }
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        self.gas == other.gas
    }
}
impl Eq for Transaction {}

impl Ord for Transaction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.gas.cmp(&other.gas)
    } // max-heap by gas
}
impl PartialOrd for Transaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Mempool {
    pub fn new(
        admin_addr: &Address,
        rx_txn: mpsc::Receiver<Transaction>,
        tx_mine: broadcast::Sender<(Address, Vec<Transaction>)>,
    ) -> Self {
        let mut txs: BinaryHeap<Transaction> = BinaryHeap::new();
        Mempool {
            txs,
            miner: admin_addr.clone(),
            rx_txn,
            tx_mine,
        }
    }

    pub async fn rcv_txs(&mut self) {
        let mut tick = tokio::time::interval(Duration::from_secs(60));
        loop {
            tokio::select! {
                Some(tx) = self.rx_txn.recv()  => {self.txs.push(tx)},
              _ = tick.tick() => {
                let mut txs : Vec<Transaction> = vec!();
                println!("Sending miner address and txs");
                while let Some(tx) = self.txs.pop(){
                    txs.push(tx);
                }
                    self.tx_mine.send((self.miner.clone(),txs));
                },
            }
        }
    }
}

pub fn random_address() -> Address {
    let mut bytes = [0u8; 20];
    rng().fill_bytes(&mut bytes);
    let mut s = String::with_capacity(2 + bytes.len() * 2);
    s.push_str("0x");

    for byte in bytes {
        write!(&mut s, "{:02x}", byte).expect("writing to String cannot fail");
    }
    s
}
