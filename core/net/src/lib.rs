#[macro_use]
extern crate mithril_codegen;

#[macro_use]
extern crate downcast;

pub use packet::{
    Packet, PacketDirection, PacketId, PacketLength, PacketStage, PacketType,
};

#[cfg(feature = "jaggrab")]
pub mod jaggrab;
mod codec;
mod packet;

pub mod packets;
pub use codec::{decode_packet, encode_packet};
