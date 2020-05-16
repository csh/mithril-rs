use super::prelude::*;

macro_rules! read_base37_username_only {
    ($name:ident) => {
        #[derive(Debug, Default)]
        pub struct $name {
            pub username: String,
        }

        impl Packet for $name {
            fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
                self.username = crate::util::text::decode_base37(src.get_u64())?;
                Ok(())
            }

            fn get_type(&self) -> PacketType {
                PacketType::$name
            }
        }
    }
}

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
pub struct PrivateChat {
    pub recipient: String,
    pub message: String,
}

impl Packet for PrivateChat {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        self.recipient = crate::util::text::decode_base37(src.get_u64())?;
        let len = src.remaining();
        let mut compressed = vec![0u8; len];
        src.copy_to_slice(&mut compressed[..]);
        self.message = crate::util::text::decompress(&compressed[..], len);
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::PrivateChat
    }
}

#[derive(Debug, Default)]
pub struct ArrowKey {
    pub roll: u16,
    pub yaw: u16,
}

impl Packet for ArrowKey {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        self.roll = src.get_u16_le();
        self.yaw = src.get_u16_le();
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::ArrowKey
    }
}

#[derive(Debug, Default)]
pub struct EnteredAmount {
    pub amount: u32,
}

impl Packet for EnteredAmount {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        self.amount = src.get_u32();
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::EnteredAmount
    }
}

#[derive(Debug)]
pub struct ItemOption {
    pub packet_type: PacketType,
    pub item_id: u16,
    pub slot: u16,
    pub interface_id: u16,
}

impl Packet for ItemOption {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        self.item_id = src.get_u16();
        self.slot = src.get_u16t(Transform::Add);
        self.interface_id = src.get_u16t(Transform::Add);
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        self.packet_type
    }
}

#[derive(Debug)]
pub struct NpcAction {
    pub packet_type: PacketType,
    pub index: u16,
}

impl Packet for NpcAction {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        self.index = src.get_u16t(Transform::Add);
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        self.packet_type
    }
}

#[derive(Debug)]
pub struct PlayerAction {
    pub packet_type: PacketType,
    pub index: u16,
}

impl Packet for PlayerAction {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        self.index = src.get_u16t_le(Transform::Add);
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        self.packet_type
    }
}

#[derive(Debug, Default)]
pub struct DialogueContinue {
    pub interface_id: u16
}

impl Packet for DialogueContinue {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        self.interface_id = src.get_u16();
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::DialogueContinue
    }
}

read_base37_username_only!(AddFriend);
read_base37_username_only!(AddIgnore);
read_base37_username_only!(RemoveFriend);
read_base37_username_only!(RemoveIgnore);

#[derive(Debug, Default)]
pub struct Button {
    pub interface_id: u16
}

impl Packet for Button {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        self.interface_id = src.get_u16();
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::Button
    }
}

#[derive(Debug, Default)]
pub struct ItemOnItem {
    pub target_slot: u16,
    pub source_slot: u16,
    pub target_id: u16,
    pub target_interface: u16,
    pub source_id: u16,
    pub source_interface: u16,
}

impl Packet for ItemOnItem {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        self.target_slot = src.get_u16();
        self.source_slot = src.get_u16t(Transform::Add);
        self.target_id = src.get_u16t_le(Transform::Add);
        self.target_interface = src.get_u16();
        self.source_id = src.get_u16_le();
        self.source_interface = src.get_u16();
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::ItemOnItem
    }
}

#[derive(Debug, Default)]
pub struct ItemOnNpc {
    pub source_id: u16,
    pub npc_id: u16,
    pub source_slot: u16,
    pub source_interface: u16,
}

impl Packet for ItemOnNpc {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        self.source_id = src.get_u16t(Transform::Add);
        self.npc_id = src.get_u16t(Transform::Add);
        self.source_slot = src.get_u16_le();
        self.source_interface = src.get_u16t(Transform::Add);
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::ItemOnItem
    }
}

#[derive(Debug, Default)]
pub struct ItemOnObject {
    pub interface_id: u16,
    pub item_id: u16,
    pub object_id: u16,
    pub slot: u16,
    pub x: u16,
    pub y: u16,
}

impl Packet for ItemOnObject {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        self.interface_id = src.get_u16();
        self.object_id = src.get_u16_le();
        self.y = src.get_u16t_le(Transform::Add);
        self.slot = src.get_u16_le();
        self.x = src.get_u16t_le(Transform::Add);
        self.item_id = src.get_u16();
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::ItemOnObject
    }
}

#[derive(Debug, Default)]
pub struct PrivacyOption {
    pub public_state: u8,
    pub private_state: u8,
    pub trade_state: u8,
}

impl Packet for PrivacyOption {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        self.public_state = src.get_u8();
        self.private_state = src.get_u8();
        self.trade_state = src.get_u8();
        Ok(())
    }

    fn try_write(&self, src: &mut BytesMut) -> anyhow::Result<()> {
        src.put_u8(self.public_state);
        src.put_u8(self.private_state);
        src.put_u8(self.trade_state);
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::PrivacyOption
    }
}

#[derive(Debug, Default)]
pub struct Command {
    pub command: String,
}

impl Packet for Command {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        self.command = src.get_rs_string();
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::Command
    }
}

#[derive(Debug, Default)]
pub struct FlashingTabClicked {
    pub tab: u8,
}

impl Packet for FlashingTabClicked {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        self.tab = src.get_u8();
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::FlashingTabClicked
    }
}

#[derive(Debug, Default)]
pub struct ClosedInterface;

impl Packet for ClosedInterface {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::ClosedInterface
    }
}

#[derive(Debug, Default)]
pub struct MagicOnNpc {
    pub entity_id: u16,
    pub spell: u16,
}

impl Packet for MagicOnNpc {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        self.entity_id = src.get_u16t_le(Transform::Add);
        self.spell = src.get_u16t(Transform::Add);
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::MagicOnNpc
    }
}

#[derive(Debug, Default)]
pub struct MagicOnItem {
    pub interface_id: u16,
    pub item_id: u16,
    pub slot: u16,
    pub spell: u16,
}

impl Packet for MagicOnItem {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        self.slot = src.get_u16();
        self.item_id = src.get_u16t(Transform::Add);
        self.interface_id = src.get_u16();
        self.spell = src.get_u16t(Transform::Add);
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::MagicOnItem
    }
}

#[derive(Debug, Default)]
pub struct MagicOnPlayer {
    pub index: u16,
    pub spell: u16,
}

impl Packet for MagicOnPlayer {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        self.index = src.get_u16t(Transform::Add);
        self.spell = src.get_u16_le();
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::MagicOnPlayer
    }
}

#[derive(Debug, Default)]
pub struct ReportAbuse {
    pub name: String,
    pub rule: u8,
    pub muted: bool,
}

impl Packet for ReportAbuse {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        self.name = crate::util::text::decode_base37(src.get_u64())?;
        self.rule = src.get_u8();
        self.muted = src.get_u8() == 1;
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::ReportAbuse
    }
}

#[derive(Debug, Default)]
pub struct SpamPacket;

impl Packet for SpamPacket {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        if src.len() > 0 {
            src.advance(src.len());
        }
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::SpamPacket
    }
}

#[derive(Debug,Default)]
pub struct TakeTileItem {
    pub x: u16,
    pub y: u16,
    pub item_id: u16,
}

impl Packet for TakeTileItem {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        self.y = src.get_u16_le();
        self.item_id = src.get_u16();
        self.x = src.get_u16_le();
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::TakeTileItem
    }
}

#[derive(Debug, Default)]
pub struct MouseClicked {
    pub delay: u64,
    pub right_click: bool,
    pub x: u32,
    pub y: u32
}

impl Packet for MouseClicked {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        let value = src.get_u32();
        self.delay = (value >> 20) as u64 * 50;
        self.right_click = (value >> 19 & 0x1) == 1;

        let coordinates = value & 0x3FFFF;
        self.x = coordinates % 765;
        self.y = coordinates / 765;
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::MouseClicked
    }
}

#[derive(Debug, Default)]
pub struct PlayerDesign {
    pub style: [u8; 7],
    pub colours: [u8; 5],
    pub gender: u8,
}

impl Packet for PlayerDesign {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        self.style = [0u8; 7];
        self.colours = [0u8; 5];
        src.copy_to_slice(&mut self.style);
        src.copy_to_slice(&mut self.colours);
        self.gender = src.get_u8();
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::PlayerDesign
    }
}

#[derive(Debug, Default)]
pub struct Walk {
    pub path: Vec<(i16, i16)>
}

impl Packet for Walk {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        let length = src.remaining();
        let steps = (length - 5) / 2;
        let mut path: Vec<(i16, i16)> = Vec::with_capacity(steps + 1);
        let x = src.get_u16t_le(Transform::Add);
        for i in 0..steps {
            path.insert(i, (
                src.get_i8() as i16,
                src.get_i8() as i16
            ))
        }
        let y = src.get_u16_le();
        let running = src.get_u8t(Transform::Negate) == 1;
        path.insert(0, (x as i16, y as i16));
        self.path = path.iter().map(|(a, b)| {
            (
                a + x as i16,
                b + y as i16
            )
        }).collect::<Vec<_>>();
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::Walk
    }
}

#[derive(Debug, Default)]
pub struct WalkWithAnticheat {
    pub path: Vec<(i16, i16)>
}

impl Packet for WalkWithAnticheat {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        let length = src.remaining() - 14;
        let steps = (length - 5) / 2;
        let mut path: Vec<(i16, i16)> = Vec::with_capacity(steps + 1);
        let x = src.get_u16t_le(Transform::Add);
        for i in 0..steps {
            path.insert(i, (
                src.get_i8() as i16,
                src.get_i8() as i16
            ))
        }
        let y = src.get_u16_le();
        let running = src.get_u8t(Transform::Negate) == 1;
        path.insert(0, (x as i16, y as i16));
        self.path = path.iter().map(|(a, b)| {
            (
                a + x as i16,
                b + y as i16
            )
        }).collect::<Vec<_>>();
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::WalkWithAnticheat
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