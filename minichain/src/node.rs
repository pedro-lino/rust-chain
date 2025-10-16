use crate::Config;
use crate::txs::UserMap;
use std::collections::HashMap;
use std::net::SocketAddr;

use crate::txs::User;
pub struct Node<'a> {
    owner: &'a User,
    address: SocketAddr,
}

pub struct NodeManager<'a> {
    map: HashMap<SocketAddr, Node<'a>>,
}

impl<'a> Node<'a> {
    fn new(owner: &'a User, address: SocketAddr) -> Self {
        Node { owner, address }
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
