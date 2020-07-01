use super::prelude::*;
use crate::PacketLength;
use mithril_codegen::EventFromPacket;
use mithril_pos::{Position, Region};

mod sync;
use crate::packets::GameplayEvent;
pub use sync::*;

mod region;
pub use region::*;

#[derive(Debug, Default, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct KeepAlive;

#[derive(Debug, Default, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct FocusUpdate {
    pub in_focus: bool,
}

#[derive(Debug, Default, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
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

#[derive(Debug, Default, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
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

#[derive(Debug, Default, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct ArrowKey {
    pub roll: u16,
    pub yaw: u16,
}

#[derive(Debug, Default, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct EnteredAmount {
    pub amount: u32,
}

#[derive(Debug)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct ItemOption {
    pub option_index: usize,
    pub item_id: u16,
    pub slot: u16,
    pub interface_id: u16,
}

impl Default for ItemOption {
    fn default() -> Self {
        Self {
            option_index: 0,
            item_id: 0,
            slot: 0,
            interface_id: 0,
        }
    }
}

impl Packet for ItemOption {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        match self.option_index {
            0 => {
                self.interface_id = src.get_u16t_le(Transform::Add);
                self.slot = src.get_u16t(Transform::Add);
                self.item_id = src.get_u16_le();
            }
            1 => {
                self.item_id = src.get_u16();
                self.slot = src.get_u16t(Transform::Add);
                self.interface_id = src.get_u16t(Transform::Add);
            }
            2 => {
                self.item_id = src.get_u16t(Transform::Add);
                self.slot = src.get_u16t_le(Transform::Add);
                self.interface_id = src.get_u16t_le(Transform::Add);
            }
            3 => {
                self.interface_id = src.get_u16t_le(Transform::Add);
                self.slot = src.get_u16_le();
                self.item_id = src.get_u16t(Transform::Add);
            }
            4 => {
                self.item_id = src.get_u16t(Transform::Add);
                self.interface_id = src.get_u16();
                self.slot = src.get_u16t(Transform::Add);
            }
            _ => anyhow::bail!("invalid ItemOption index"),
        }
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        match self.option_index {
            0 => PacketType::FirstItemOption,
            1 => PacketType::SecondItemOption,
            2 => PacketType::ThirdItemOption,
            3 => PacketType::FourthItemOption,
            _ => PacketType::FifthItemOption,
        }
    }
}

impl From<ItemOption> for GameplayEvent {
    fn from(packet: ItemOption) -> Self {
        match packet.option_index {
            0 => GameplayEvent::FirstItemOption(packet),
            1 => GameplayEvent::SecondItemOption(packet),
            2 => GameplayEvent::ThirdItemOption(packet),
            3 => GameplayEvent::FourthItemOption(packet),
            _ => GameplayEvent::FifthItemOption(packet),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct ItemAction {
    pub action_index: usize,
    pub item_id: u16,
    pub slot: u16,
    pub interface_id: u16,
}

impl Default for ItemAction {
    fn default() -> Self {
        Self {
            action_index: 0,
            item_id: 0,
            slot: 0,
            interface_id: 0,
        }
    }
}

impl Packet for ItemAction {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        match self.action_index {
            0 => {
                self.interface_id = src.get_u16t(Transform::Add);
                self.slot = src.get_u16t(Transform::Add);
                self.item_id = src.get_u16t(Transform::Add);
            }
            1 => {
                self.interface_id = src.get_u16t_le(Transform::Add);
                self.item_id = src.get_u16t_le(Transform::Add);
                self.slot = src.get_u16_le();
            }
            2 => {
                self.interface_id = src.get_u16_le();
                self.item_id = src.get_u16t(Transform::Add);
                self.slot = src.get_u16t(Transform::Add);
            }
            3 => {
                self.slot = src.get_u16t(Transform::Add);
                self.interface_id = src.get_u16();
                self.item_id = src.get_u16t(Transform::Add);
            }
            4 => {
                self.slot = src.get_u16_le();
                self.interface_id = src.get_u16t(Transform::Add);
                self.slot = src.get_u16_le();
            }
            _ => anyhow::bail!("invalid ItemOption index"),
        }
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        match self.action_index {
            0 => PacketType::FirstItemAction,
            1 => PacketType::SecondItemAction,
            2 => PacketType::ThirdItemAction,
            3 => PacketType::FourthItemAction,
            _ => PacketType::FifthItemAction,
        }
    }
}

impl From<ItemAction> for GameplayEvent {
    fn from(packet: ItemAction) -> Self {
        match packet.action_index {
            0 => GameplayEvent::FirstItemAction(packet),
            1 => GameplayEvent::SecondItemAction(packet),
            2 => GameplayEvent::ThirdItemAction(packet),
            3 => GameplayEvent::FourthItemAction(packet),
            _ => GameplayEvent::FifthItemAction(packet),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct NpcAction {
    pub action_index: u16,
    // TODO: Investigate if correct name once NPC spawning is functional
    pub npc_id: u16,
}

impl Default for NpcAction {
    fn default() -> Self {
        Self {
            action_index: 0,
            npc_id: 0,
        }
    }
}

impl Packet for NpcAction {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        match self.action_index {
            0 => self.npc_id = src.get_u16_le(),
            1 => self.npc_id = src.get_u16t(Transform::Add),
            2 => self.npc_id = src.get_u16t_le(Transform::Add),
            3 => self.npc_id = src.get_u16(),
            4 => self.npc_id = src.get_u16_le(),
            _ => anyhow::bail!("invalid NpcAction index"),
        }
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        match self.action_index {
            0 => PacketType::FirstNpcAction,
            1 => PacketType::SecondNpcAction,
            2 => PacketType::ThirdNpcAction,
            3 => PacketType::FourthNpcAction,
            _ => PacketType::FifthNpcAction,
        }
    }
}

impl From<NpcAction> for GameplayEvent {
    fn from(packet: NpcAction) -> Self {
        match packet.action_index {
            0 => GameplayEvent::FirstNpcAction(packet),
            1 => GameplayEvent::SecondNpcAction(packet),
            2 => GameplayEvent::ThirdNpcAction(packet),
            3 => GameplayEvent::FourthNpcAction(packet),
            _ => GameplayEvent::FifthNpcAction(packet),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct PlayerAction {
    pub action_index: u16,
    pub player_id: u16,
}

impl Default for PlayerAction {
    fn default() -> Self {
        Self {
            action_index: 0,
            player_id: 0,
        }
    }
}

impl Packet for PlayerAction {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        match self.action_index {
            0 => self.player_id = src.get_u16(),
            1 => self.player_id = src.get_u16_le(),
            2 => self.player_id = src.get_u16_le(),
            3 => self.player_id = src.get_u16_le(),
            4 => self.player_id = src.get_u16_le(),
            _ => anyhow::bail!("invalid NpcAction index"),
        }
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        match self.action_index {
            0 => PacketType::FirstPlayerAction,
            1 => PacketType::SecondPlayerAction,
            2 => PacketType::ThirdPlayerAction,
            3 => PacketType::FourthPlayerAction,
            _ => PacketType::FifthPlayerAction,
        }
    }
}

impl From<PlayerAction> for GameplayEvent {
    fn from(packet: PlayerAction) -> Self {
        match packet.action_index {
            0 => GameplayEvent::FirstPlayerAction(packet),
            1 => GameplayEvent::SecondPlayerAction(packet),
            2 => GameplayEvent::ThirdPlayerAction(packet),
            3 => GameplayEvent::FourthPlayerAction(packet),
            _ => GameplayEvent::FifthPlayerAction(packet),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct ObjectAction {
    pub action_index: u16,
    pub object_id: u16,
    pub x: u16,
    pub y: u16,
}

impl Default for ObjectAction {
    fn default() -> Self {
        Self {
            action_index: 0,
            object_id: 0,
            x: 0,
            y: 0,
        }
    }
}

impl Packet for ObjectAction {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        match self.action_index {
            0 => {
                self.x = src.get_u16t_le(Transform::Add);
                self.object_id = src.get_u16();
                self.y = src.get_u16t(Transform::Add);
            }
            1 => {
                self.object_id = src.get_u16t_le(Transform::Add);
                self.y = src.get_u16_le();
                self.x = src.get_u16t(Transform::Add)
            }
            2 => {
                self.x = src.get_u16_le();
                self.y = src.get_u16();
                self.object_id = src.get_u16t_le(Transform::Add);
            }
            _ => anyhow::bail!("invalid ObjectAction index"),
        }
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        match self.action_index {
            0 => PacketType::FirstObjectAction,
            1 => PacketType::SecondObjectAction,
            _ => PacketType::ThirdObjectAction,
        }
    }
}

impl From<ObjectAction> for GameplayEvent {
    fn from(packet: ObjectAction) -> Self {
        match packet.action_index {
            0 => GameplayEvent::FirstObjectAction(packet),
            1 => GameplayEvent::SecondObjectAction(packet),
            _ => GameplayEvent::ThirdObjectAction(packet),
        }
    }
}

#[derive(Debug, Default, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct DialogueContinue {
    pub interface_id: u16,
}

#[derive(Debug, Default, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct AddFriend {
    #[base37]
    pub username: String,
}

#[derive(Debug, Default, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct AddIgnore {
    #[base37]
    pub username: String,
}

#[derive(Debug, Default, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct RemoveFriend {
    #[base37]
    pub username: String,
}

#[derive(Debug, Default, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct RemoveIgnore {
    #[base37]
    pub username: String,
}

#[derive(Debug, Default, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct Button {
    pub interface_id: u16,
}

#[derive(Debug, Default, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
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

#[derive(Debug, Default, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
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

#[derive(Debug, Default, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
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

#[derive(Debug, Default, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct PrivacyOption {
    pub public_state: u8,
    pub private_state: u8,
    pub trade_state: u8,
}

#[derive(Debug, Default, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct Command {
    pub command: String,
}

#[derive(Debug, Default, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct FlashingTabClicked {
    pub tab: u8,
}

#[derive(Debug, Default, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct ClosedInterface;

#[derive(Debug, Default, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct MagicOnNpc {
    #[transform = "add"]
    #[endian = "little"]
    pub entity_id: u16,
    #[transform = "add"]
    pub spell: u16,
}

#[derive(Debug, Default, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct MagicOnItem {
    pub slot: u16,
    #[transform = "add"]
    pub item_id: u16,
    pub interface_id: u16,
    #[transform = "add"]
    pub spell: u16,
}

#[derive(Debug, Default, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct MagicOnPlayer {
    #[transform = "add"]
    pub index: u16,
    #[endian = "little"]
    pub spell: u16,
}

#[derive(Debug, Default, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct ReportAbuse {
    #[base37]
    pub username: String,
    pub rule: u8,
    pub muted: bool,
}

#[derive(Debug, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
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

#[derive(Debug, Default, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct TakeTileItem {
    #[endian = "little"]
    pub y: u16,
    pub item_id: u16,
    #[endian = "little"]
    pub x: u16,
}

#[derive(Debug, Default, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
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

#[derive(Debug, Default, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
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

#[derive(Debug, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
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

#[derive(Debug, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct SetWidgetModel {
    #[transform = "add"]
    #[endian = "little"]
    pub interface_id: u16,
    pub model_id: u16,
}

#[derive(Debug, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct EnterAmount;

#[derive(Debug, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct DisplayCrossbones {
    pub shown: bool,
}

#[derive(Debug, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct SwitchTabInterface {
    pub interface_id: u16,
    #[transform = "add"]
    pub tab_id: u8,
}

#[derive(Debug, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct SetWidgetNpcModel {
    #[transform = "add"]
    #[endian = "little"]
    pub model_id: u16,
    #[transform = "add"]
    #[endian = "little"]
    pub interface_id: u16,
}

#[derive(Debug, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct OpenInterface {
    pub id: u16,
}

#[derive(Debug, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct SetPlayerAction {
    #[transform = "negate"]
    pub slot: u8,
    #[transform = "add"]
    pub is_primary_action: bool,
    pub action: String,
}

#[derive(Debug, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct DisplayTabInterface {
    #[transform = "negate"]
    pub tab_id: u8,
}

#[derive(Debug, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct Logout;

#[derive(Debug, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct UpdateRunEnergy {
    pub energy: u8,
}

#[derive(Debug, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct SetWidgetText {
    pub message: String,
    #[transform = "add"]
    pub widget_id: u16,
}

#[derive(Debug, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct UpdateSkill {
    pub skill_id: u8,
    pub experience: u32,
    pub level: u8,
}

#[derive(Debug, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct OpenDialogueInterface {
    #[endian = "little"]
    pub interface_id: u16,
}

#[derive(Debug, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct SetWidgetVisibility {
    pub is_visible: bool,
    #[transform = "add"]
    pub widget_id: u16,
}

#[derive(Debug, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct SetWidgetPlayerModel {
    #[endian = "little"]
    pub interface_id: u16,
}

#[derive(Debug, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct SetWidgetModelAnimation {
    pub interface_id: u16,
    pub animation_id: u16,
}

#[derive(Debug, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct CloseInterface;

#[derive(Debug, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct UpdateWeight {
    pub weight: u16,
}

#[derive(Debug, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct SetWidgetItemModel {
    #[endian = "little"]
    pub interface_id: u16,
    pub zoom: u16,
    pub model_id: u16,
}

#[derive(Debug, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct OpenInterfaceSidebar {
    #[transform = "add"]
    pub interface_id: u16,
    pub sidebar_id: u16,
}

#[derive(Debug, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct IdAssignment {
    #[transform = "add"]
    pub is_member: bool,
    #[transform = "add"]
    #[endian = "little"]
    pub entity_id: u16,
}

#[derive(Debug, Packet, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct ServerMessage {
    pub message: String,
}

#[cfg_attr(feature = "test-equality", derive(PartialEq))]
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

#[derive(Debug, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
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

#[derive(Packet, Debug, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct ClearRegion {
    #[transform = "negate"]
    local_x: u8,
    #[transform = "subtract"]
    local_y: u8,
}

impl ClearRegion {
    pub fn new(player: Position, region: Region) -> Self {
        let local_x = ((region.x - (player.get_x() / 8 - 6)) * 8) as u8;
        let local_y = ((region.y - (player.get_y() / 8 - 6)) * 8) as u8;

        ClearRegion {
            local_x,
            local_y
        }
    }   
}

#[derive(Debug, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
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
