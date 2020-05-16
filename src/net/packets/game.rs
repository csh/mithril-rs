use super::prelude::*;

#[derive(Debug)]
pub struct KeepAlive;

impl Packet for KeepAlive {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::KeepAlive
    }
}

#[derive(Debug, Default)]
pub struct FocusUpdate {
    pub in_focus: bool
}

impl Packet for FocusUpdate {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        self.in_focus = src.get_u8() == 1;
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::FocusUpdate
    }
}

#[derive(Debug, Default)]
pub struct PublicChat {
    pub effects: u8,
    pub colour: u8,
    pub message: String,
}

impl Packet for PublicChat {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        self.effects = src.get_u8t(Transform::Subtract);
        self.colour = src.get_u8t(Transform::Subtract);

        let len = src.remaining();
        let mut compressed = vec![0u8; len];
        src.get_reverse(&mut compressed, Transform::Add);
        self.message = crate::util::text::decompress(&compressed[..], len);

        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::PublicChat
    }
}

#[derive(Debug, Default)]
pub struct AddFriend {
    pub username: String,
}

impl Packet for AddFriend {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        self.username = crate::util::text::decode_base37(src.get_u64())?;
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::AddFriend
    }
}

#[derive(Debug)]
pub struct IdAssignment;

impl Packet for IdAssignment {
    fn try_write(&self, src: &mut BytesMut) -> anyhow::Result<()> {
        src.put_u8t(1, Transform::Add);
        src.put_u16t(rand::random(), Transform::Add);
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::IdAssignment
    }
}