use bytes::{Buf, BufMut, BytesMut};
use rand::Rng;
use rand_isaac::IsaacRng;

use crate::{Packet, PacketDirection, PacketId, PacketLength, PacketStage, PacketType};
use crate::packets::PacketEvent;

pub fn encode_packet(
    isaac: Option<&mut IsaacRng>,
    packet: PacketEvent,
    dst: &mut BytesMut,
) -> anyhow::Result<()> {
    log::info!("Encoding a {:?}", packet.get_type());
    let isaac = match isaac {
        Some(isaac) => {
            debug_assert!(packet.is_gameplay(), "encoding requested using the ISAAC generator was not a gameplay packet");
            isaac
        },
        None => {
            debug_assert!(packet.is_handshake(), "encoding requested without the ISAAC generator was not a handshake packet");
            packet.try_write(dst)?;
            return Ok(());
        }
    };

    let encoding_buf = &mut BytesMut::new();
    packet.try_write(encoding_buf)?;

    let packet_type = packet.get_type();
    let packet_id = packet_type.get_id().id.wrapping_add(isaac.gen::<u8>());
    dst.put_u8(packet_id);
    match packet_type.packet_length() {
        Some(PacketLength::Fixed(len)) => debug_assert_eq!(
            encoding_buf.len(),
            len,
            "packet length is fixed but did not match"
        ),
        Some(PacketLength::VariableByte) => dst.put_u8(encoding_buf.len() as u8),
        Some(PacketLength::VariableShort) => dst.put_u16(encoding_buf.len() as u16),
        None => {}
    }
    dst.extend_from_slice(encoding_buf);
    Ok(())
}

pub fn decode_packet(
    isaac: Option<&mut IsaacRng>,
    src: &mut BytesMut,
) -> anyhow::Result<PacketEvent> {
    let packet_id = match isaac {
        Some(isaac) => {
            let decoded_id = src.get_u8().wrapping_sub(isaac.gen::<u8>());
            PacketId::new(
                decoded_id,
                PacketDirection::Serverbound,
                PacketStage::Gameplay,
            )
        }
        None => PacketId::new(src[0], PacketDirection::Serverbound, PacketStage::Handshake),
    };

    let packet_type = match PacketType::get_from_id(packet_id) {
        Some(packet_type) => packet_type,
        None => anyhow::bail!("Unknown packet"),
    };

    log::info!("Decoding a {:?}", packet_type);
    let mut read_buffer = match packet_type.packet_length() {
        Some(PacketLength::VariableByte) => {
            let split_index = src.get_u8() as usize;
            src.split_to(split_index)
        }
        Some(PacketLength::VariableShort) => {
            let split_index = src.get_u16() as usize;
            src.split_to(split_index)
        }
        Some(PacketLength::Fixed(expected_len)) => src.split_to(expected_len),
        None => src.split_to(src.len()),
    };

    let mut packet = packet_type.create().expect("packet creation");
    packet.try_read(&mut read_buffer).map(|_| packet)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packets::ServerMessage;
    use rand::SeedableRng;
    use crate::packets::GameplayEvent;

    #[test]
    fn test_plain_encode() {
        let packet = ServerMessage {
            message: "Hello World".to_string(),
        };

        let mut buf = BytesMut::new();
        assert!(
            encode_packet(None, GameplayEvent::ServerMessage(packet).into(), &mut buf).is_ok(),
            "ServerMessage is encodable"
        );

        println!("{:02X}", buf);
        println!("{:?}", buf);
    }

    #[test]
    fn test_isaac_encode() {
        let mut encode_isaac = Some(rand_isaac::IsaacRng::seed_from_u64(0));
        let message = "Hello World".to_owned();

        let packet = crate::packets::ServerMessage {
            message: message.clone(),
        };

        let mut buf = BytesMut::new();
        assert!(
            encode_packet(encode_isaac.as_mut(), GameplayEvent::ServerMessage(packet).into(), &mut buf).is_ok(),
            "ServerMessage is encodable"
        );
        println!("{:02X}", buf);
        println!("{:?}", buf);
    }
}
