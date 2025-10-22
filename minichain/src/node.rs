use crate::Config;
use anyhow::Ok;
use anyhow::{Result, bail, ensure};
use jsonrpsee::client_transport::ws::Sender;
use rand::{RngCore, rng};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::hash_map::Entry;
use std::fmt::Write;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;

type Address = String;
type Hash = String;

pub struct User {
    address: Address,
    balance: u128,
    tx_sender: mpsc::Sender<Transaction>,
}
pub struct UserMap {
    users: HashMap<Address, User>,
    admins: HashSet<Address>,
}

pub struct Node {
    owner_addr: Address,
    ip_address: SocketAddr,
}

pub struct NodeManager {
    map: HashMap<SocketAddr, Node>,
}

pub struct Transaction {
    id: Hash,
    gas: u128,
    data: Box<[u8]>,
    sender: Address,
}

struct Block {
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
    tx_rcv: mpsc::Receiver<Transaction>,
}

impl User {
    pub fn new(addr: Option<Address>, tx_sender: mpsc::Sender<Transaction>) -> Self {
        let address = addr.unwrap_or_else(|| random_address());

        User {
            address,
            balance: 0,
            tx_sender,
        }
    }

    pub fn get_address(&self) -> &Address {
        &self.address
    }
    pub fn get_balance(&self) -> &u128 {
        &self.balance
    }

    fn send_tx(&mut self, data: Box<[u8]>, gas: u128) -> Result<()> {
        ensure!(
            self.balance >= gas,
            "Sender doesn't have enough funds to pay for gas."
        );

        self.balance -= gas;

        self.tx_sender.send(Transaction {
            id: random_address(),
            gas,
            data,
            sender: self.address.clone(),
        });
        Ok(())
    }
}

impl<'a> UserMap {
    pub fn new() -> Self {
        let users: HashMap<Address, User> = HashMap::new();
        let admins: HashSet<Address> = HashSet::new();
        UserMap { users, admins }
    }

    pub fn add_user(&mut self, usr: User) -> Result<(String)> {
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
    fn new(owner_addr: Address, ip_address: SocketAddr) -> Self {
        Node {
            owner_addr: owner_addr,
            ip_address,
        }
    }

    pub async fn execute_txs(
        &self,
        mempool: &mut Mempool,
        chain: &mut Blockchain,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.owner_addr == mempool.miner,
            "Current node was not elected to execute mempool txs."
        );

        let mut b = Block {
            number: chain.get_height(),
            timestamp: SystemTime::now(),
            txs: Vec::new(),
        };

        while let Some(tx) = mempool.txs.pop() {
            b.txs.push(tx);
        }

        Ok(())
    }
}
impl NodeManager {
    pub fn new(cfg: &Config) -> Self {
        let bootnode = Node::new(cfg.admin_addr.clone(), cfg.boot_addr);

        let mut map: HashMap<SocketAddr, Node> = HashMap::new();
        map.insert(cfg.boot_addr, bootnode);

        NodeManager { map: map }
    }

    pub fn get_node(&self, ip: SocketAddr) -> Option<&Node> {
        self.map.get(&ip)
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
    pub fn new(admin_addr: &Address, tx_rcv: mpsc::Receiver<Transaction>) -> Self {
        let mut txs: BinaryHeap<Transaction> = BinaryHeap::new();
        Mempool {
            txs,
            miner: admin_addr.clone(),
            tx_rcv,
        }
    }

    pub fn rcv_txs() {
        let mut tick = tokio::time::interval(Duration::from_secs(60));
        loop {
            tokio::select! {

                Some(tx) =
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
