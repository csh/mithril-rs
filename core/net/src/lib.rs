#[macro_use]
extern crate mithril_codegen;

pub use codec::RunescapeCodec;
pub use packet::{
    cast_packet, Packet, PacketDirection, PacketId, PacketLength, PacketStage, PacketType,
};

mod codec;
mod packet;

pub mod packets;
