use std::net::SocketAddr;

use specs::{Component, VecStorage};

use rand::SeedableRng;
use rand_isaac::IsaacRng;

pub struct NetworkAddress(pub SocketAddr);

impl Component for NetworkAddress {
    type Storage = VecStorage<Self>;
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct ConnectionIsaac {
    pub encoding: IsaacRng,
    pub decoding: IsaacRng
}

impl ConnectionIsaac {
    pub fn new(decoding_seed: [u8; 32], encoding_seed: [u8; 32]) -> Self {
        Self {
            decoding: IsaacRng::from_seed(decoding_seed),
            encoding: IsaacRng::from_seed(encoding_seed)
        }
    }
}
