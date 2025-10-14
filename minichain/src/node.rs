use dotenvy::dotenv;
use std::collections::HashMap;
use std::net::SocketAddr;

use crate::txs::User;
struct Node<'a> {
    owner: &'a User,
    address: SocketAddr,
}

pub struct NodeManager<'a> {
    map: HashMap<&'a String, &'a Node<'a>>,
    active: &'a SocketAddr,
}

impl<'a> NodeManager<'a> {
    pub fn new(active: &'a SocketAddr) -> Self {
        NodeManager {
            map: (HashMap::new()),
            active,
        }
    }
}
