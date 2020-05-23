use ahash::AHashMap;
use specs::Entity;
use num_traits::cast::ToPrimitive;
use parking_lot::{Mutex, RwLock};
use strum::IntoEnumIterator;

use mithril_core::net::{cast_packet, Packet, PacketType};

pub struct Packets {
    buffer: Vec<PacketsInner>,
}

impl Default for Packets {
    fn default() -> Self {
        Self::new()
    }
}

impl Packets {
    pub fn new() -> Self {
        Self {
            buffer: PacketType::iter()
                .map(|_| PacketsInner::default())
                .collect(),
        }
    }

    pub fn push(&self, player: Entity, packet: Box<dyn Packet>) {
        let index = packet.get_type().to_usize().unwrap();
        self.buffer[index].push(player, packet);
    }

    pub fn received_from<T>(
        &self,
        player: Entity,
        packet_type: PacketType,
    ) -> impl Iterator<Item = T>
    where
        T: Packet + 'static,
    {
        let index = packet_type.to_usize().unwrap();
        self.buffer[index]
            .received_from(player)
            .map(|boxed| cast_packet(boxed))
    }
}

enum Either<A, B> {
    Left(A),
    Right(B),
}

impl<A, B, I> Iterator for Either<A, B>
where
    A: Iterator<Item = I>,
    B: Iterator<Item = I>,
{
    type Item = I;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Either::Left(left) => left.next(),
            Either::Right(right) => right.next(),
        }
    }
}

#[derive(Default)]
struct PacketsInner {
    inner: RwLock<AHashMap<Entity, Mutex<Vec<Box<dyn Packet>>>>>,
}

impl PacketsInner {
    fn push(&self, player: Entity, packet: Box<dyn Packet>) {
        let guard = self.inner.read();
        if let Some(queued) = guard.get(&player) {
            queued.lock().push(packet);
        } else {
            drop(guard);
            let mut queued = Vec::with_capacity(8);
            queued.push(packet);
            self.inner.write().insert(player, Mutex::new(queued));
        }
    }

    fn received_from(&self, player: Entity) -> impl Iterator<Item = Box<dyn Packet>> {
        let guard = self.inner.read();
        if let Some(queued) = guard.get(&player) {
            let vec = queued.lock().drain(..).collect::<Vec<Box<dyn Packet>>>();
            Either::Left(vec.into_iter())
        } else {
            Either::Right(std::iter::empty())
        }
    }
}
