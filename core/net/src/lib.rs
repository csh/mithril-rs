#[macro_use]
extern crate mithril_codegen;

pub use codec::RunescapeCodec;
#[cfg(feature = "jaggrab")]
pub use jaggrab::{JaggrabCodec, JaggrabError, JaggrabFile};
pub use packet::{
    cast_packet, Packet, PacketDirection, PacketId, PacketLength, PacketStage, PacketType,
};

mod codec;
#[cfg(feature = "jaggrab")]
mod jaggrab;
mod packet;

pub mod packets;
