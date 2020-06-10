use super::prelude::*;
use crate::PacketLength;
use mithril_pos::Position;

#[derive(Debug, Default, Packet)]
pub struct KeepAlive;

#[derive(Debug, Default, Packet)]
pub struct FocusUpdate {
    pub in_focus: bool,
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
        self.message = mithril_text::decompress(&compressed[..], len);

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
        self.recipient = mithril_text::decode_base37(src.get_u64())?;
        let len = src.remaining();
        let mut compressed = vec![0u8; len];
        src.copy_to_slice(&mut compressed[..]);
        self.message = mithril_text::decompress(&compressed[..], len);
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::PrivateChat
    }
}

#[derive(Debug, Default, Packet)]
pub struct ArrowKey {
    pub roll: u16,
    pub yaw: u16,
}

#[derive(Debug, Default, Packet)]
pub struct EnteredAmount {
    pub amount: u32,
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

#[derive(Debug, Default, Packet)]
pub struct DialogueContinue {
    pub interface_id: u16,
}

#[derive(Debug, Default, Packet)]
pub struct AddFriend {
    #[base37]
    pub username: String,
}

#[derive(Debug, Default, Packet)]
pub struct AddIgnore {
    #[base37]
    pub username: String,
}

#[derive(Debug, Default, Packet)]
pub struct RemoveFriend {
    #[base37]
    pub username: String,
}

#[derive(Debug, Default, Packet)]
pub struct RemoveIgnore {
    #[base37]
    pub username: String,
}

#[derive(Debug, Default, Packet)]
pub struct Button {
    pub interface_id: u16,
}

#[derive(Debug, Default, Packet)]
pub struct ItemOnItem {
    pub target_slot: u16,
    #[transform = "add"]
    pub source_slot: u16,
    #[transform = "add"]
    #[endian = "little"]
    pub target_id: u16,
    pub target_interface: u16,
    #[endian = "little"]
    pub source_id: u16,
    pub source_interface: u16,
}

#[derive(Debug, Default, Packet)]
pub struct ItemOnNpc {
    #[transform = "add"]
    pub source_id: u16,
    #[transform = "add"]
    pub npc_id: u16,
    #[endian = "little"]
    pub source_slot: u16,
    #[transform = "add"]
    pub source_interface: u16,
}

#[derive(Debug, Default, Packet)]
pub struct ItemOnObject {
    pub interface_id: u16,
    #[endian = "little"]
    pub object_id: u16,
    #[transform = "add"]
    #[endian = "little"]
    pub y: u16,
    #[endian = "little"]
    pub slot: u16,
    #[transform = "add"]
    #[endian = "little"]
    pub x: u16,
    pub item_id: u16,
}

#[derive(Debug, Default, Packet)]
pub struct PrivacyOption {
    pub public_state: u8,
    pub private_state: u8,
    pub trade_state: u8,
}

#[derive(Debug, Default, Packet)]
pub struct Command {
    pub command: String,
}

#[derive(Debug, Default, Packet)]
pub struct FlashingTabClicked {
    pub tab: u8,
}

#[derive(Debug, Default, Packet)]
pub struct ClosedInterface;

#[derive(Debug, Default, Packet)]
pub struct MagicOnNpc {
    #[transform = "add"]
    #[endian = "little"]
    pub entity_id: u16,
    #[transform = "add"]
    pub spell: u16,
}

#[derive(Debug, Default, Packet)]
pub struct MagicOnItem {
    pub slot: u16,
    #[transform = "add"]
    pub item_id: u16,
    pub interface_id: u16,
    #[transform = "add"]
    pub spell: u16,
}

#[derive(Debug, Default, Packet)]
pub struct MagicOnPlayer {
    #[transform = "add"]
    pub index: u16,
    #[endian = "little"]
    pub spell: u16,
}

#[derive(Debug, Default, Packet)]
pub struct ReportAbuse {
    #[base37]
    pub username: String,
    pub rule: u8,
    pub muted: bool,
}

#[derive(Debug)]
pub struct SpamPacket(pub PacketLength);

impl Packet for SpamPacket {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        if !src.is_empty() {
            src.advance(src.len());
        }
        Ok(())
    }

    // TODO: Can we drop this?
    fn get_type(&self) -> PacketType {
        PacketType::SpamPacket(self.0)
    }
}

#[derive(Debug, Default, Packet)]
pub struct TakeTileItem {
    #[endian = "little"]
    pub y: u16,
    pub item_id: u16,
    #[endian = "little"]
    pub x: u16,
}

#[derive(Debug, Default)]
pub struct MouseClicked {
    pub delay: u64,
    pub right_click: bool,
    pub x: u32,
    pub y: u32,
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

#[derive(Debug)]
pub struct Walk {
    pub packet_type: PacketType,
    pub path: Vec<Position>,
    pub running: bool,
}

impl Packet for Walk {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        let length = match self.packet_type {
            PacketType::Walk => src.remaining(),
            PacketType::WalkWithAnticheat => src.remaining() - 14,
            _ => unreachable!("packet type should always be Walk or WalkWithAnticheat"),
        };
        let steps = (length - 5) / 2;
        let mut path = Vec::with_capacity(steps + 1);
        let x = src.get_u16t_le(Transform::Add) as i16;
        for _ in 0..steps {
            path.push((src.get_i8() as i16, src.get_i8() as i16));
        }
        let y = src.get_i16_le();
        self.running = src.get_u8t(Transform::Negate) == 1;
        self.path = path
            .iter()
            .map(|(a, b)| Position::new(a + x, b + y))
            .collect::<Vec<_>>();
        self.path.insert(0, Position::new(x, y));
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        self.packet_type
    }
}

#[derive(Debug, Packet)]
pub struct SetWidgetModel {
    #[transform = "add"]
    #[endian = "little"]
    pub interface_id: u16,
    pub model_id: u16,
}

#[derive(Debug, Packet)]
pub struct EnterAmount;

#[derive(Debug, Packet)]
pub struct DisplayCrossbones {
    pub shown: bool,
}

#[derive(Debug, Packet)]
pub struct SwitchTabInterface {
    pub interface_id: u16,
    #[transform = "add"]
    pub tab_id: u8,
}

#[derive(Debug, Packet)]
pub struct SetWidgetNpcModel {
    #[transform = "add"]
    #[endian = "little"]
    pub model_id: u16,
    #[transform = "add"]
    #[endian = "little"]
    pub interface_id: u16,
}

#[derive(Debug, Packet)]
pub struct OpenInterface {
    pub id: u16,
}

#[derive(Debug, Packet)]
pub struct SetPlayerAction {
    #[transform = "negate"]
    pub slot: u8,
    #[transform = "add"]
    pub is_primary_action: bool,
    pub action: String,
}

#[derive(Debug, Packet)]
pub struct DisplayTabInterface {
    #[transform = "negate"]
    pub tab_id: u8,
}

#[derive(Debug, Packet)]
pub struct Logout;

#[derive(Debug, Packet)]
pub struct UpdateRunEnergy {
    pub energy: u8,
}

#[derive(Debug, Packet)]
pub struct SetWidgetText {
    pub message: String,
    #[transform = "add"]
    pub widget_id: u16,
}

#[derive(Debug, Packet)]
pub struct UpdateSkill {
    pub skill_id: u8,
    pub experience: u32,
    pub level: u8,
}

#[derive(Debug, Packet)]
pub struct OpenDialogueInterface {
    #[endian = "little"]
    pub interface_id: u16,
}

#[derive(Debug, Packet)]
pub struct SetWidgetVisibility {
    pub is_visible: bool,
    #[transform = "add"]
    pub widget_id: u16,
}

#[derive(Debug, Packet)]
pub struct SetWidgetPlayerModel {
    #[endian = "little"]
    pub interface_id: u16,
}

#[derive(Debug, Packet)]
pub struct SetWidgetModelAnimation {
    pub interface_id: u16,
    pub animation_id: u16,
}

#[derive(Debug, Packet)]
pub struct CloseInterface;

#[derive(Debug, Packet)]
pub struct UpdateWeight {
    pub weight: u16,
}

#[derive(Debug, Packet)]
pub struct SetWidgetItemModel {
    #[endian = "little"]
    pub interface_id: u16,
    pub zoom: u16,
    pub model_id: u16,
}

#[derive(Debug, Packet)]
pub struct OpenInterfaceSidebar {
    #[transform = "add"]
    pub interface_id: u16,
    pub sidebar_id: u16,
}

#[derive(Debug, Packet)]
pub struct IdAssignment {
    #[transform = "add"]
    pub is_member: bool,
    #[transform = "add"]
    #[endian = "little"]
    pub entity_id: u16,
}

#[derive(Debug, Packet)]
pub struct ServerMessage {
    pub message: String,
}

pub enum Config {
    Byte(u16, u8),
    Int(u16, u32),
}

impl Packet for Config {
    fn try_write(&self, src: &mut BytesMut) -> anyhow::Result<()> {
        match self {
            Config::Byte(id, value) => {
                src.put_u16_le(*id);
                src.put_u8(*value);
            }
            Config::Int(id, value) => {
                src.put_u16_le(*id);
                src.put_u32_me(*value)
            }
        }
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        match self {
            Config::Byte(_, _) => PacketType::ConfigByte,
            Config::Int(_, _) => PacketType::ConfigInt,
        }
    }
}

pub struct RegionChange {
    pub position: Position,
}

impl Packet for RegionChange {
    fn try_write(&self, src: &mut BytesMut) -> anyhow::Result<()> {
        let central_x = self.position.get_x() / 8;
        let central_y = self.position.get_y() / 8;
        src.put_u16t(central_x as u16, Transform::Add);
        src.put_u16(central_y as u16);
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::RegionChange
    }
}

pub struct ClearRegion {
    pub player: Position,
    pub region: Position,
}

impl Packet for ClearRegion {
    fn try_write(&self, src: &mut BytesMut) -> anyhow::Result<()> {
        let (local_x, local_y) = self.region.get_relative(self.player);
        src.put_u8t(local_x, Transform::Negate);
        src.put_u8t(local_y, Transform::Subtract);
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::ClearRegion
    }
}

#[derive(Debug)]
pub enum EntityMovement {
    Teleport {
        destination: Position,
        current: Position,
        changed_region: bool,
    },
    Move {
        direction: i32,
    },
}

#[derive(Debug)]
pub struct PlayerSynchronization {
    pub player_update: Option<EntityMovement>,
}

impl Packet for PlayerSynchronization {
    fn try_write(&self, src: &mut BytesMut) -> anyhow::Result<()> {
        match self.player_update {
            Some(EntityMovement::Teleport {
                destination,
                current,
                changed_region,
            }) => {
                src.put_bits(|mut writer| {
                    writer.put_bits(1, 1);
                    writer.put_bits(2, 3);
                    writer.put_bits(2, destination.get_plane() as _);
                    writer.put_bits(1, if changed_region { 1 } else { 0 });
                    writer.put_bits(1, 0);
                    let (x, y) = destination.get_relative(current);
                    writer.put_bits(7, y as _);
                    writer.put_bits(7, x as _);
                    writer
                });
            }
            Some(EntityMovement::Move { direction }) => {
                src.put_bits(|mut writer| {
                    writer.put_bits(1, 1);
                    writer.put_bits(2, 1);
                    writer.put_bits(3, direction as _);
                    writer.put_bits(1, 0);
                    writer
                });
            }
            None => {
                src.put_bits(|mut writer| {
                    writer.put_bits(1, 0);
                    writer
                });
            }
        }

        src.put_bits(|mut writer| {
            writer.put_bits(8, 0); // zero players near us to update
            writer
        });

        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::PlayerSynchronization
    }
}

pub struct NpcSynchronization;

impl Packet for NpcSynchronization {
    fn try_write(&self, src: &mut BytesMut) -> anyhow::Result<()> {
        src.put_bits(|mut writer| {
            writer.put_bits(8, 0);
            writer
        });
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::NpcSynchronization
    }
}
