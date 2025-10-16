use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt::Write;

use anyhow::{Result, bail, ensure};
use rand::{RngCore, rng};

type Address = String;
pub struct User {
    address: Address,
    balance: u128,
}
pub struct UserMap {
    map: HashMap<Address, User>,
}

struct Transaction {
    data: Box<[u8]>,
    sender: Address,
}

struct Block {
    number: u128,
    timestamp: i64,
    txs: Vec<Transaction>,
}

pub struct Mempool {
    txs: Vec<(Transaction, u128)>,
    miner: String,
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

        mempool.txs.push((
            Transaction {
                data,
                sender: self.address.clone(),
            },
            gas,
        ));
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

impl Mempool {
    pub fn new(admin_addr: &Address) -> Self {
        let mut txs: Vec<(Transaction, u128)> = Vec::new();
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
