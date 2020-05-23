#[macro_use] extern crate mithril_codegen;

pub use codec::RunescapeCodec;
pub use packet::{cast_packet, Packet, PacketDirection, PacketId, PacketStage, PacketType, PacketLength};

pub mod buf;
mod codec;
mod packet;

pub mod packets;
