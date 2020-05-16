use anyhow::Context;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use rand::{Rng, SeedableRng};
use tokio_util::codec::{Decoder, Encoder};

use crate::net::packets::{Packet, PacketDirection, PacketId, PacketStage, PacketType};

pub struct RunescapeCodec {
    encoding_rng: Option<rand_isaac::IsaacRng>,
    decoding_rng: Option<rand_isaac::IsaacRng>,
    stage: Stage,
}

pub enum Stage {
    Handshake,
    Gameplay,
}

impl RunescapeCodec {
    pub fn new() -> Self {
        RunescapeCodec {
            decoding_rng: None,
            encoding_rng: None,
            stage: Stage::Handshake,
        }
    }

    pub fn advance_stage(&mut self) {
        log::debug!("Enabling processing of gameplay packets");
        match self.stage {
            Stage::Handshake => self.stage = Stage::Gameplay,
            Stage::Gameplay => unreachable!("advance_stage() should only be called once after the login sequence has finished"),
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
            Stage::Handshake => item.try_write(dst),
            Stage::Gameplay => {
                let isaac = self.encoding_rng.as_mut().expect("ISAAC has not been configured");
                let packet_type = item.get_type();
                let mut encoding_buf = BytesMut::new();
                item.try_write(&mut encoding_buf)?;
                let packet_id = packet_type.get_id().id + isaac.gen::<u8>();
                dst.put_u8(packet_id);
                if packet_type.is_variable_length() {
                    dst.put_u8(encoding_buf.len() as u8);
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
        if src.len() > 0 {
            let len = src.len();

            let mut buf = src.split_to(len);
            let packet_id = buf[0];
            let packet_type = match self.stage {
                Stage::Handshake => {
                    // TODO: Figure out a more elegant solution than peeking packet_id
                    PacketType::get_from_id(PacketId::new(packet_id, PacketDirection::Serverbound, PacketStage::Handshake))
                }
                Stage::Gameplay => {
                    let isaac = self.decoding_rng.as_mut().expect("ISAAC has not been configured");
                    let decoded_id = buf.get_u8() - isaac.gen::<u8>();
                    PacketType::get_from_id(PacketId::new(decoded_id, PacketDirection::Serverbound, PacketStage::Gameplay))
                }
            };

            match packet_type {
                Some(packet_type) => {
                    log::debug!("We received a {:?}", packet_type);
                    let mut packet = packet_type.create().context("packet construction failed")?;
                    packet.try_read(&mut buf).context("packet read failed")?;
                    Ok(Some(packet))
                }
                None => {
                    log::warn!("Received unknown packet ID; skipping packet {:02X}", packet_id);
                    src.advance(src.len());
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }
}