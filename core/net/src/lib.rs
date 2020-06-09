#[macro_use] extern crate mithril_codegen;

#[cfg(feature = "jaggrab")]
pub use jaggrab::{JaggrabCodec, JaggrabFile, JaggrabError};
pub use codec::RunescapeCodec;
pub use packet::{cast_packet, Packet, PacketDirection, PacketId, PacketStage, PacketType, PacketLength};

#[cfg(feature = "jaggrab")]
mod jaggrab;
mod codec;
mod packet;

pub mod packets;
