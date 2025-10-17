use crate::Config;
use anyhow::{Result, bail, ensure};
use rand::{RngCore, rng};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt::Write;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

type Address = String;
type Hash = String;

pub struct User {
    address: Address,
    balance: u128,
}
pub struct UserMap {
    map: HashMap<Address, User>,
}

pub struct Node<'a> {
    owner: &'a User,
    address: SocketAddr,
}

pub struct NodeManager<'a> {
    map: HashMap<SocketAddr, Node<'a>>,
}

struct Transaction {
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
}

impl User {
    pub fn new() -> Self {
        let address = random_address();
        User {
            address,
            balance: 0,
        }
    }

    fn send_tx<'a>(&mut self, data: Box<[u8]>, gas: u128, mempool: &'a mut Mempool) -> Result<()> {
        ensure!(
            self.balance >= gas,
            "Sender doesn't have enough funds to pay for gas."
        );

        self.balance -= gas;

        mempool.txs.push(Transaction {
            id: random_address(),
            gas,
            data,
            sender: self.address.clone(),
        });
        Ok(())
    }
}

impl UserMap {
    pub fn new(admin_addr: &String) -> Self {
        let map: HashMap<String, User> = HashMap::new();
        let admin = User {
            address: admin_addr.clone(),
            balance: 9999999,
        };

        let mut user_map = UserMap { map };
        user_map.add_user(admin).unwrap();
        user_map
    }

    fn add_user(&mut self, usr: User) -> Result<()> {
        match self.map.entry(usr.address.clone()) {
            Entry::Vacant(v) => {
                v.insert(usr);
                Ok(())
            }
            Entry::Occupied(..) => bail!("User was already registered"),
        }
    }

    pub fn get_user(&self, addr: &String) -> Option<&User> {
        self.map.get(addr)
    }
}

impl<'a> Node<'a> {
    fn new(owner: &'a User, address: SocketAddr) -> Self {
        Node { owner, address }
    }

    fn execute_txs(&self, mempool: &mut Mempool, chain: &mut Blockchain) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.owner.address == mempool.miner,
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
impl<'a> NodeManager<'a> {
    pub fn new(cfg: &Config, user_map: &'a UserMap) -> Self {
        let admin = user_map
            .get_user(&cfg.admin_addr)
            .expect("Admin address was not properly registered.");
        let bootnode = Node::new(admin, cfg.boot_addr);

        let mut map: HashMap<SocketAddr, Node> = HashMap::new();
        map.insert(cfg.boot_addr, bootnode);

        NodeManager { map: map }
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
    pub fn new(admin_addr: &Address) -> Self {
        let mut txs: BinaryHeap<Transaction> = BinaryHeap::new();
        Mempool {
            txs,
            miner: admin_addr.clone(),
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
