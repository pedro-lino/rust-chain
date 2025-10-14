use anyhow::{Result, ensure};

pub struct User {
    address: String,
    balance: u128,
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
