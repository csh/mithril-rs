use std::iter;

use ahash::AHashMap;
use parking_lot::{Mutex, RwLock};
use specs::Entity;

use mithril_core::net::{cast_packet, Packet, PacketType};
use smallvec::SmallVec;

pub struct Packets {
    buffer: AHashMap<PacketType, PacketsInner>,
}

impl Default for Packets {
    fn default() -> Self {
        Self::new()
    }
}

impl Packets {
    pub fn new() -> Self {
        let mut buffer = AHashMap::new();
        PacketType::iter().for_each(|packet_type| {
            buffer.insert(*packet_type, PacketsInner::default());
        });
        Self { buffer }
    }

    pub fn push(&self, player: Entity, packet: Box<dyn Packet>) {
        if let Some(inner) = self.buffer.get(&packet.get_type()) {
            inner.push(player, packet)
        }
    }

    pub fn received_from<T>(
        &self,
        player: Entity,
        packet_type: PacketType,
    ) -> impl Iterator<Item = T>
    where
        T: Packet + 'static,
    {
        self.buffer
            .get(&packet_type)
            .map(|inner| {
                let packets = inner
                    .received_from(player)
                    .map(|packet| cast_packet(packet));
                Either::Left(packets)
            })
            .unwrap_or_else(|| Either::Right(iter::empty()))
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

type PacketsVec = SmallVec<[Box<dyn Packet>; 4]>;
type PacketsMap = AHashMap<Entity, Mutex<PacketsVec>>;

#[derive(Default)]
struct PacketsInner {
    inner: RwLock<PacketsMap>,
}

impl PacketsInner {
    fn push(&self, player: Entity, packet: Box<dyn Packet>) {
        let guard = self.inner.read();
        if let Some(queued) = guard.get(&player) {
            queued.lock().push(packet);
        } else {
            drop(guard);
            self.inner
                .write()
                .insert(player, Mutex::new(iter::once(packet).collect()));
        }
    }

    fn received_from(&self, player: Entity) -> impl Iterator<Item = Box<dyn Packet>> {
        let guard = self.inner.read();
        if let Some(queued) = guard.get(&player) {
            let vec = queued
                .lock()
                .drain(..)
                .collect::<SmallVec<[Box<dyn Packet>; 4]>>();
            Either::Left(vec.into_iter())
        } else {
            Either::Right(std::iter::empty())
        }
    }
}
