use anyhow::Context;
use bytes::{Buf, BufMut, BytesMut};
use rand::{Rng, SeedableRng};
use tokio_util::codec::{Decoder, Encoder};

use crate::{Packet, PacketDirection, PacketId, PacketLength, PacketStage, PacketType};

pub struct RunescapeCodec {
    encoding_rng: Option<rand_isaac::IsaacRng>,
    decoding_rng: Option<rand_isaac::IsaacRng>,
    stage: PacketStage,
}

#[allow(clippy::new_without_default)]
impl RunescapeCodec {
    pub fn new() -> Self {
        RunescapeCodec {
            decoding_rng: None,
            encoding_rng: None,
            stage: PacketStage::Handshake,
        }
    }

    pub fn advance_stage(&mut self) {
        log::debug!("Enabling processing of gameplay packets");
        match self.stage {
            PacketStage::Handshake => self.stage = PacketStage::Gameplay,
            PacketStage::Gameplay => unreachable!(
                "advance_stage() should only be called once after the login sequence has finished"
            ),
        }
    }

    pub fn set_isaac_keys(&mut self, server_key: u64, client_key: u64) {
        log::debug!("Setting ISAAC encryption keys");
        let seed_inputs = [
            client_key.wrapping_shr(32) as u32,
            client_key as u32,
            server_key.wrapping_shr(32) as u32,
            server_key as u32,
        ];
        self.decoding_rng = Some(rand_isaac::IsaacRng::from_seed(prepare_isaac_seed(
            seed_inputs,
            0,
        )));
        self.encoding_rng = Some(rand_isaac::IsaacRng::from_seed(prepare_isaac_seed(
            seed_inputs,
            50,
        )));
    }
}

fn prepare_isaac_seed(seed_input: [u32; 4], increment: u32) -> [u8; 32] {
    let mut seed = BytesMut::with_capacity(32);
    for input in &seed_input {
        seed.put_u32_le(input + increment);
    }
    seed.put(&[0u8; 16][..]);

    let mut actual_seed = [0u8; 32];
    seed.copy_to_slice(&mut actual_seed);
    actual_seed
}

impl Encoder<Box<dyn Packet>> for RunescapeCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: Box<dyn Packet>, dst: &mut BytesMut) -> anyhow::Result<()> {
        match self.stage {
            PacketStage::Handshake => item.try_write(dst),
            PacketStage::Gameplay => {
                anyhow::ensure!(self.encoding_rng.is_some(), "ISAAC has not been configured");
                let isaac = self.encoding_rng.as_mut().unwrap();
                let packet_type = item.get_type();
                log::debug!("Sending a {:?}", packet_type);
                let mut encoding_buf = BytesMut::new();
                item.try_write(&mut encoding_buf)?;
                let packet_id = packet_type.get_id().id.wrapping_add(isaac.gen::<u8>());
                dst.put_u8(packet_id);
                if let Some(packet_length) = packet_type.packet_length() {
                    match packet_length {
                        PacketLength::VariableByte => dst.put_u8(encoding_buf.len() as u8),
                        PacketLength::VariableShort => dst.put_u16(encoding_buf.len() as u16),
                        PacketLength::Fixed(len) => {
                            assert_eq!(encoding_buf.len(), len, "fixed length mismatch");
                        }
                    }
                }
                dst.put(encoding_buf);
                Ok(())
            }
        }
    }
}

impl Decoder for RunescapeCodec {
    type Item = Box<dyn Packet>;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }

        let packet_id = src[0];
        let packet_type = match self.stage {
            PacketStage::Handshake => {
                // TODO: Figure out a more elegant solution than peeking packet_id
                PacketType::get_from_id(PacketId::new(
                    packet_id,
                    PacketDirection::Serverbound,
                    self.stage,
                ))
            }
            PacketStage::Gameplay => {
                anyhow::ensure!(self.decoding_rng.is_some(), "ISAAC has not been configured");
                let isaac = self.decoding_rng.as_mut().unwrap();
                let decoded_id = src.get_u8().wrapping_sub(isaac.gen::<u8>());
                PacketType::get_from_id(PacketId::new(
                    decoded_id,
                    PacketDirection::Serverbound,
                    self.stage,
                ))
            }
        };

        match packet_type {
            Some(packet_type) => {
                let mut src = match packet_type.packet_length() {
                    Some(PacketLength::Fixed(len)) => src.split_to(len),
                    Some(PacketLength::VariableByte) => {
                        let len = src.get_u8();
                        src.split_to(len as usize)
                    }
                    Some(PacketLength::VariableShort) => {
                        let len = src.get_u16();
                        src.split_to(len as usize)
                    }
                    None => src.split_to(src.remaining()),
                };
                log::debug!("We received a {:?}; len = {}", packet_type, src.len());
                let mut packet = packet_type.create().context("packet construction failed")?;
                packet.try_read(&mut src).context("packet read failed")?;
                if src.has_remaining() {
                    log::debug!(
                        "buf still contains {} bytes after reading {:?}; {:X}",
                        src.len(),
                        packet_type,
                        src
                    );
                }
                Ok(Some(packet))
            }
            None => {
                log::warn!(
                    "Received unknown packet ID for {:?}; skipping packet {:02X}",
                    self.stage,
                    packet_id
                );
                src.advance(src.len());
                Ok(None)
            }
        }
    }
}
