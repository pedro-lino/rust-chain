use std::collections::HashMap;
use std::net::SocketAddr;

use crate::txs::User;
pub struct Node<'a> {
    owner: &'a User,
    address: SocketAddr,
}

pub struct NodeManager<'a> {
    map: HashMap<SocketAddr, Node<'a>>,
    active: SocketAddr,
}

impl<'a> Node<'a> {
    pub fn new(owner: &'a User, address: SocketAddr) -> Self {
        Node { owner, address }
    }
}
impl<'a> NodeManager<'a> {
    pub fn new(active: Node<'a>) -> Self {
        let addr = active.address;
        let mut map: HashMap<SocketAddr, Node> = HashMap::new();
        map.insert(active.address, active);

        NodeManager {
            map: map,
            active: addr,
        }
    }
}
