use super::prelude::*;

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

#[derive(Debug, Default)]
pub struct Walk {
    pub path: Vec<(i16, i16)>,
    pub running: bool,
}

impl Packet for Walk {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        let length = src.remaining();
        let steps = (length - 5) / 2;
        let mut path: Vec<(i16, i16)> = Vec::with_capacity(steps + 1);
        let x = src.get_u16t_le(Transform::Add);
        for i in 0..steps {
            path.insert(i, (src.get_i8() as i16, src.get_i8() as i16))
        }
        let y = src.get_u16_le();
        self.running = src.get_u8t(Transform::Negate) == 1;
        path.insert(0, (x as i16, y as i16));
        self.path = path
            .iter()
            .map(|(a, b)| (a + x as i16, b + y as i16))
            .collect::<Vec<_>>();
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::Walk
    }
}

#[derive(Debug, Default)]
pub struct WalkWithAnticheat {
    pub path: Vec<(i16, i16)>,
    pub running: bool,
}

impl Packet for WalkWithAnticheat {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        let length = src.remaining() - 14;
        let steps = (length - 5) / 2;
        let mut path: Vec<(i16, i16)> = Vec::with_capacity(steps + 1);
        let x = src.get_u16t_le(Transform::Add);
        for i in 0..steps {
            path.insert(i, (src.get_i8() as i16, src.get_i8() as i16))
        }
        let y = src.get_u16_le();
        self.running = src.get_u8t(Transform::Negate) == 1;
        path.insert(0, (x as i16, y as i16));
        self.path = path
            .iter()
            .map(|(a, b)| (a + x as i16, b + y as i16))
            .collect::<Vec<_>>();
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::WalkWithAnticheat
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
    pub shown: bool
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
    pub action: String
}

#[derive(Debug, Packet)]
pub struct DisplayTabInterface {
    #[transform = "negate"]
    pub tab_id: u8
}

#[derive(Debug, Packet)]
pub struct Logout;

#[derive(Debug, Packet)]
pub struct UpdateRunEnergy {
    pub energy: u8
}

#[derive(Debug, Packet)]
pub struct SetWidgetText {
    pub message: String,
    #[transform = "add"]
    pub widget_id: u16
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
    pub interface_id: u16
}

#[derive(Debug, Packet)]
pub struct SetWidgetVisibility {
    pub is_visible: bool,
    #[transform = "add"]
    pub widget_id: u16
}

#[derive(Debug, Packet)]
pub struct SetWidgetPlayerModel {
    #[endian = "little"]
    pub interface_id: u16
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
    pub weight: u16
}

#[derive(Debug, Packet)]
pub struct SetWidgetItemModel {
    #[endian = "little"]
    pub interface_id: u16,
    pub zoom: u16,
    pub model_id: u16
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