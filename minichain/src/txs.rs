use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt::Write;

use anyhow::{Result, bail, ensure};
use rand::{RngCore, rng};

const ADMN_ADDR: &str = "0x4b6e99d0d66b1caa1f0377fbb7a97c12ecb3b457";
pub struct User {
    address: String,
    balance: u128,
}
pub struct UserMap {
    map: HashMap<String, User>,
}

struct Transaction<'a> {
    data: Box<[u8]>,
    sender: &'a str,
}

struct Block<'a> {
    number: u128,
    timestamp: i64,
    txs: Vec<Transaction<'a>>,
}

impl UserMap {
    pub fn new() -> Self {
        let map: HashMap<String, User> = HashMap::new();
        let admin = User {
            address: ADMN_ADDR.to_string(),
            balance: 9999999,
        };

        let user_map = UserMap { map };
        user_map.add_user(admin).unwrap();
        user_map
    }

    fn add_user(&mut self, usr: User) -> anyhow::Result<_> {
        match self.map.entry(usr.address.clone()) {
            Entry::Vacant(v) => {
                v.insert(usr);
                Ok(())
            }
            Entry::Occupied(..) => bail!("User was already registered"),
        }
    }
}

impl User {
    pub fn new() -> Self {
        let address = random_address();
        User {
            address,
            balance: 0,
        }
    }
}

struct Mempool<'a> {
    txs: Vec<(Transaction<'a>, u128)>,
}

impl<'a> Mempool<'a> {
    fn new(&mut self, sender: &'a mut User, gas: u128, data: Box<[u8]>) -> Result<()> {
        ensure!(
            sender.balance >= gas,
            "Sender doesn't have enough funds to pay for gas."
        );

        sender.balance -= gas;

        self.txs.push((
            Transaction {
                data,
                sender: &sender.address,
            },
            gas,
        ));
        Ok(())
    }
}

pub fn random_address() -> String {
    let mut bytes = [0u8; 20];
    rng().fill_bytes(&mut bytes);
    let mut s = String::with_capacity(2 + bytes.len() * 2);
    s.push_str("0x");

    for byte in bytes {
        write!(&mut s, "{:02x}", byte).expect("writing to String cannot fail");
    }
    s
}
