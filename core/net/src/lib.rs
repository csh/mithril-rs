#[macro_use]
extern crate mithril_codegen;

pub use packet::{Packet, PacketDirection, PacketId, PacketLength, PacketStage, PacketType};

mod codec;
#[cfg(feature = "jaggrab")]
pub mod jaggrab;
mod packet;

pub mod packets;
pub use codec::{decode_packet, encode_packet};
