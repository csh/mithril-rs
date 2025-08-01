use crate::{Packet, PacketType};
use ahash::AHashMap;
use bytes::buf::BufMut;
use bytes::{Bytes, BytesMut};
use mithril_buf::{BitWriter, GameBufMut, Transform};
use mithril_pos::Position;
use mithril_text::{compress, encode_base37};
use std::convert::TryInto;

#[derive(Debug, PartialEq)]
pub struct Animation {
    id: u16,
    delay: u8,
}

impl Animation {
    fn write(&self, buf: &mut BytesMut) {
        buf.put_u16_le(self.id);
        buf.put_u8t(self.delay, Transform::Negate);
    }
}

#[derive(Debug, PartialEq)]
pub struct Item {
    pub id: u16,
}

#[derive(Debug, Default, PartialEq)]
pub struct Equipment {
    pub hat: Option<Item>,
    pub cape: Option<Item>,
    pub amulet: Option<Item>,
    pub weapon: Option<Item>,
    pub chest: Option<Item>,
    pub shield: Option<Item>,
    pub legs: Option<Item>,
    pub hands: Option<Item>,
    pub feet: Option<Item>,
    pub ring: Option<Item>,
    pub arrows: Option<Item>,
}

#[derive(Debug, PartialEq)]
pub enum AppearanceType {
    Npc(u16),
    Player(Equipment, Vec<u16>),
}

#[derive(Debug, PartialEq)]
pub struct Appearance {
    pub name: String,
    pub gender: u8,
    pub combat_level: u8,
    pub skill_level: u16,
    pub appearance_type: AppearanceType,
    pub colours: Vec<u8>, // Enums are cool I tell you!
}

impl Appearance {
    fn write(&self, buf: &mut BytesMut) {
        // I'm cheating the system here, I'll fake buffers
        let mut buf2 = BytesMut::new(); // buf2 = new buffer in your case
        buf2.put_u8(self.gender);
        buf2.put_u8(0);

        match &self.appearance_type {
            AppearanceType::Npc(id) => {
                buf2.put_u8(255);
                buf2.put_u8(255);
                buf2.put_u16(*id);
            }
            AppearanceType::Player(equipment, style) => {
                self.write_appearance(&mut buf2, self.gender, &equipment, &style)
            }
        }

        // Can we method ref this? (i.e. pass buf.put_u8)
        self.colours.iter().for_each(|b| buf2.put_u8(*b));
        buf2.put_u16(0x328);
        buf2.put_u16(0x337);
        buf2.put_u16(0x333);
        buf2.put_u16(0x334);
        buf2.put_u16(0x335);
        buf2.put_u16(0x336);
        buf2.put_u16(0x338);
        buf2.put_u64(encode_base37(&self.name));
        buf2.put_u8(self.combat_level);
        buf2.put_u16(self.skill_level);
        // Of course, buf and buf2 are actually different :P
        let props_len = buf2.len();
        buf.put_u8t(props_len as u8, Transform::Negate);
        buf.put(buf2);
    }
}

macro_rules! item_or_zero {
    ($item:expr, $buffer:expr) => {
        if let Some(item) = $item {
            $buffer.put_u16(0x200 + item.id);
        } else {
            $buffer.put_u8(0);
        }
    };
}

impl Appearance {
    fn write_appearance(
        &self,
        buf: &mut BytesMut,
        gender: u8,
        equipment: &Equipment,
        style: &[u16],
    ) {
        item_or_zero!(&equipment.hat, buf);
        item_or_zero!(&equipment.cape, buf);
        item_or_zero!(&equipment.amulet, buf);
        item_or_zero!(&equipment.weapon, buf);
        // body / clothing
        buf.put_u16(if let Some(item) = &equipment.chest {
            (0x200 + item.id) as u16
        } else {
            (0x100 + style[2]) as u16
        });
        item_or_zero!(&equipment.shield, buf);
        buf.put_u16(0x100 + style[3]);
        // legs
        buf.put_u16(if let Some(item) = &equipment.legs {
            (0x200 + item.id) as u16
        } else {
            (0x100 + style[5]) as u16
        });
        buf.put_u16(0x100 + style[0]);
        // hands
        buf.put_u16(if let Some(item) = &equipment.hands {
            (0x200 + item.id) as u16
        } else {
            (0x100 + style[4]) as u16
        });
        // feet
        buf.put_u16(if let Some(item) = &equipment.feet {
            (0x200 + item.id) as u16
        } else {
            (0x100 + style[6]) as u16
        });
        // Mustache / beard
        if gender != 0 {
            buf.put_u8(0);
        } else {
            buf.put_u16(0x100 + style[1]);
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Chat {
    message: String,
    color: u8,           // TODO: EnumSet pls
    effects: u8,         // TODO: EnumSet pls
    privilege_level: u8, // TODO: Enum pls
}

impl Chat {
    fn write(&self, buf: &mut BytesMut) {
        buf.put_u16_le((self.color as u16) << 8 | self.effects as u16);
        buf.put_u8(self.privilege_level);

        let mut compressed = compress(&self.message);
        compressed.reverse();
        // Check length I suppose
        buf.put_u8t(compressed.len().try_into().unwrap(), Transform::Negate);
        buf.put::<Bytes>(compressed.into());
    }
}

#[derive(Debug, PartialEq)]
pub struct ForceChat {
    message: String,
}

impl ForceChat {
    fn write(&self, buf: &mut BytesMut) {
        buf.put_rs_string(self.message.clone());
    }
}

#[derive(Debug, PartialEq)]
pub struct ForceMovement {
    initial_pos: (u8, u8),
    final_pos: (u8, u8),
    travel_duration: (u16, u16),
    direction: u8,
}

impl ForceMovement {
    fn write(&self, buf: &mut BytesMut) {
        buf.put_u8t(self.initial_pos.0, Transform::Subtract);
        buf.put_u8t(self.initial_pos.1, Transform::Subtract);
        buf.put_u8t(self.final_pos.0, Transform::Subtract);
        buf.put_u8t(self.final_pos.1, Transform::Subtract);
        buf.put_u16t_le(self.travel_duration.0, Transform::Add);
        buf.put_u16t(self.travel_duration.1, Transform::Add);
        buf.put_u8t(self.direction, Transform::Add);
    }
}

#[derive(Debug, PartialEq)]
pub struct Graphic {
    id: u16,
    height: u16,
    delay: u16,
}

impl Graphic {
    fn write(&self, buf: &mut BytesMut) {
        buf.put_u16_le(self.id);
        buf.put_u32((self.height as u32) << 16 | self.delay as u32);
    }
}

#[derive(Debug, PartialEq)]
pub struct HitUpdate {
    damage: u8,
    damage_type: u8,
    health: u8,
    max_health: u8,
}

impl HitUpdate {
    fn write(&self, buf: &mut BytesMut) {
        buf.put_u8(self.damage);
        buf.put_u8t(self.damage_type, Transform::Add);
        buf.put_u8t(self.health, Transform::Negate);
        buf.put_u8(self.max_health);
    }
}

#[derive(Debug, PartialEq)]
pub struct InteractingMob {
    index: u16,
}

impl InteractingMob {
    fn write(&self, buf: &mut BytesMut) {
        buf.put_u16_le(self.index);
    }
}

#[derive(Debug, PartialEq)]
pub struct SecondaryHitUpdate {
    damage: u8,
    damage_type: u8,
    health: u8,
    max_health: u8,
}

impl SecondaryHitUpdate {
    fn write(&self, buf: &mut BytesMut) {
        buf.put_u8(self.damage);
        buf.put_u8t(self.damage_type, Transform::Subtract);
        buf.put_u8(self.health);
        buf.put_u8t(self.max_health, Transform::Negate);
    }
}

#[derive(Debug, PartialEq)]
pub struct TurnToPosition {
    position: (u16, u16),
}

impl TurnToPosition {
    fn write(&self, buf: &mut BytesMut) {
        buf.put_u16t_le(self.position.0 * 2 + 1, Transform::Add);
        buf.put_u16_le(self.position.1 * 2 + 1);
    }
}

#[derive(PartialEq, Debug)]
pub enum SyncBlock {
    ForceMovement(ForceMovement),
    Graphic(Graphic),
    Animation(Animation),
    ForceChat(ForceChat),
    Chat(Chat),
    InteractingMob(InteractingMob),
    Appearance(Appearance),
    TurnToPosition(TurnToPosition),
    HitUpdate(HitUpdate),
    SecondaryHitUpdate(SecondaryHitUpdate),
}

impl SyncBlock {}

#[derive(Debug, PartialEq)]
pub enum EntityMovement {
    Teleport {
        destination: Position,
        current: Position,
        changed_region: bool,
    },
    Move {
        direction: i32,
    },
    Run {
        directions: (i32, i32),
    },
}

#[derive(Debug, PartialEq)]
pub struct AddPlayer {
    id: u16,
    // From player
    dx: u8,
    // From player
    dy: u8,
}

impl AddPlayer {
    pub fn new(id: u16, player_position: Position, new_player_position: Position) -> AddPlayer {
        let (dx, dy) = new_player_position - player_position;
        AddPlayer {
            id,
            dx: dx as u8,
            dy: dy as u8,
        }
    }
}

impl SyncBlock {
    fn to_player_id(&self) -> u16 {
        match self {
            Self::ForceMovement(_) => 0x400,
            Self::Graphic(_) => 0x100,
            Self::Animation(_) => 0x8,
            Self::ForceChat(_) => 0x4,
            Self::Chat(_) => 0x80,
            Self::InteractingMob(_) => 0x1,
            Self::Appearance(_) => 0x10,
            Self::TurnToPosition(_) => 0x2,
            Self::HitUpdate(_) => 0x20,
            Self::SecondaryHitUpdate(_) => 0x200,
        }
    }

    fn write(&self, buffer: &mut BytesMut) {
        match self {
            Self::ForceMovement(packet) => packet.write(buffer),
            Self::Graphic(packet) => packet.write(buffer),
            Self::Animation(packet) => packet.write(buffer),
            Self::ForceChat(packet) => packet.write(buffer),
            Self::Chat(packet) => packet.write(buffer),
            Self::InteractingMob(packet) => packet.write(buffer),
            Self::Appearance(packet) => packet.write(buffer),
            Self::TurnToPosition(packet) => packet.write(buffer),
            Self::HitUpdate(packet) => packet.write(buffer),
            Self::SecondaryHitUpdate(packet) => packet.write(buffer),
        }
    }
}

macro_rules! into_syncblock {
    ($type:ident) => {
        impl From<$type> for SyncBlock {
            fn from(packet: $type) -> Self {
                Self::$type(packet)
            }
        }
    };
}

into_syncblock!(ForceMovement);
into_syncblock!(Graphic);
into_syncblock!(Animation);
into_syncblock!(ForceChat);
into_syncblock!(Chat);
into_syncblock!(InteractingMob);
into_syncblock!(Appearance);
into_syncblock!(TurnToPosition);
into_syncblock!(HitUpdate);
into_syncblock!(SecondaryHitUpdate);

const BLOCKS: [u16; 10] = [0x400, 0x100, 0x8, 0x4, 0x80, 0x1, 0x10, 0x2, 0x20, 0x200];

#[derive(Debug, Default, PartialEq)]
pub struct SyncBlocks {
    blocks: AHashMap<u16, SyncBlock>,
}

impl SyncBlocks {
    pub fn add_block(&mut self, block: SyncBlock) -> &mut Self {
        self.blocks.insert(block.to_player_id(), block);
        self
    }

    fn write(&self, buf: &mut BytesMut) {
        let mask: u16 = self.blocks.keys().fold(0, |acc, val| acc | val);
        if mask >= 0x100 {
            let mask = mask | 0x40;
            buf.put_u16_le(mask);
        } else {
            buf.put_u8(mask as u8);
        }

        BLOCKS
            .iter()
            .filter_map(|id| self.blocks.get(id))
            .for_each(|block| block.write(buf));
    }

    fn has_updates(&self) -> bool {
        !self.blocks.is_empty()
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub enum PlayerUpdate {
    Remove(),
    Add(AddPlayer, SyncBlocks),
    Update(Option<EntityMovement>, SyncBlocks),
}

#[derive(Debug, EventFromPacket)]
#[cfg_attr(feature = "test-equality", derive(PartialEq))]
pub struct PlayerSynchronization {
    pub player_update: Option<PlayerUpdate>,
    pub other_players: Vec<PlayerUpdate>,
}

impl Packet for PlayerSynchronization {
    fn try_write(&self, src: &mut BytesMut) -> anyhow::Result<()> {
        let mut block_buffer = BytesMut::new();
        src.put_bits(|mut writer| {
            if let Some(update) = &self.player_update {
                self.write_player(&mut writer, &mut block_buffer, &update);
            } else {
                writer.put_bits(1, 0); // No updates
            }

            let count = self
                .other_players
                .iter()
                .filter(|update| {
                    if let PlayerUpdate::Update(_, _) = update {
                        true
                    } else {
                        false
                    }
                })
                .count();

            writer.put_bits(8, count as u32);
            self.other_players
                .iter()
                .for_each(|update| self.write_player(&mut writer, &mut block_buffer, &update));
            if !block_buffer.is_empty() {
                writer.put_bits(11, 2047);
            }
            writer
        });
        if !block_buffer.is_empty() {
            src.put(block_buffer);
        }
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::PlayerSynchronization
    }
}

impl PlayerSynchronization {
    fn write_player(
        &self,
        writer: &mut BitWriter,
        block_buffer: &mut BytesMut,
        update: &PlayerUpdate,
    ) {
        match update {
            PlayerUpdate::Remove() => {
                writer.put_bits(1, 1);
                writer.put_bits(2, 3);
            }
            PlayerUpdate::Add(player, blocks) => {
                writer.put_bits(11, player.id as u32);
                writer.put_bits(1, if blocks.has_updates() { 1 } else { 0 });
                writer.put_bits(1, 1); // Teleported, so clears walking queue
                writer.put_bits(5, player.dy as u32);
                writer.put_bits(5, player.dx as u32);
                if blocks.has_updates() {
                    blocks.write(block_buffer);
                }
            }
            PlayerUpdate::Update(movement, blocks) => {
                match movement {
                    Some(EntityMovement::Teleport {
                        destination,
                        current,
                        changed_region,
                    }) => {
                        writer.put_bits(1, 1);
                        writer.put_bits(2, 3);
                        writer.put_bits(2, destination.get_plane() as _);
                        writer.put_bits(1, if *changed_region { 1 } else { 0 });
                        writer.put_bits(1, if blocks.has_updates() { 1 } else { 0 });
                        let (x, y) = destination.get_relative(*current);
                        writer.put_bits(7, y as _);
                        writer.put_bits(7, x as _);
                    }
                    Some(EntityMovement::Move { direction }) => {
                        writer.put_bits(1, 1);
                        writer.put_bits(2, 1);
                        writer.put_bits(3, *direction as u32);
                        writer.put_bits(1, if blocks.has_updates() { 1 } else { 0 });
                    }
                    Some(EntityMovement::Run { directions }) => {
                        writer.put_bits(1, 1);
                        writer.put_bits(2, 2);
                        writer.put_bits(3, directions.0 as _);
                        writer.put_bits(3, directions.1 as _);
                        writer.put_bits(1, if blocks.has_updates() { 1 } else { 0 });
                    }
                    None => {
                        writer.put_bits(1, 0);
                    }
                }
                if blocks.has_updates() {
                    blocks.write(block_buffer);
                }
            }
        }
    }

    #[allow(dead_code)]
    fn get_type(&self) -> PacketType {
        PacketType::PlayerSynchronization
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_sync() {
        const PACKET: [u8; 146] = [
            0xE2, 0xC1, 0xA8, 0x0F, 0xB0, 0x00, 0x70, 0x03, 0xFF, 0x80, 0x14, 0x73, 0x65, 0x6C,
            0x6C, 0x69, 0x6E, 0x67, 0x20, 0x67, 0x66, 0x20, 0x31, 0x30, 0x6B, 0x0A, 0xCD, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x12, 0x00, 0x01, 0x1A, 0x01, 0x24, 0x01, 0x00,
            0x01, 0x21, 0x01, 0x2A, 0x01, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x28, 0x03,
            0x37, 0x03, 0x33, 0x03, 0x34, 0x03, 0x35, 0x03, 0x36, 0x03, 0x38, 0x09, 0xF9, 0xE6,
            0x4D, 0xEA, 0xA3, 0x58, 0xF3, 0x45, 0x00, 0x00, 0x14, 0x73, 0x65, 0x6C, 0x6C, 0x69,
            0x6E, 0x67, 0x20, 0x67, 0x66, 0x20, 0x31, 0x30, 0x6B, 0x0A, 0xCD, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01, 0x12, 0x00, 0x01, 0x1A, 0x01, 0x24, 0x01, 0x00, 0x01, 0x21,
            0x01, 0x2A, 0x01, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x28, 0x03, 0x37, 0x03,
            0x33, 0x03, 0x34, 0x03, 0x35, 0x03, 0x36, 0x03, 0x38, 0x09, 0xF9, 0xE6, 0x4D, 0xEA,
            0xA3, 0x58, 0xF3, 0x45, 0x00, 0x00,
        ];
        let mut buf = BytesMut::new();

        let force_chat = ForceChat {
            message: String::from("selling gf 10k"),
        };
        let appearance = Appearance {
            appearance_type: AppearanceType::Player(
                Equipment::default(),
                vec![0, 10, 18, 26, 33, 36, 42],
            ),
            name: String::from("DarkSeraphim"),
            gender: 0x0,
            combat_level: 69,
            skill_level: 0,
            colours: vec![0, 0, 0, 0, 0],
        };
        let mut my_blocks = SyncBlocks::default();
        my_blocks.add_block(force_chat.into());
        my_blocks.add_block(appearance.into());

        let remove_player = PlayerUpdate::Remove();
        let move_player = PlayerUpdate::Update(
            Some(EntityMovement::Move { direction: 4 }),
            SyncBlocks::default(),
        );

        let force_chat = ForceChat {
            message: String::from("selling gf 10k"),
        };
        let appearance = Appearance {
            appearance_type: AppearanceType::Player(
                Equipment::default(),
                vec![0, 10, 18, 26, 33, 36, 42],
            ),
            name: String::from("DarkSeraphim"),
            gender: 0x0,
            combat_level: 69,
            skill_level: 0,
            colours: vec![0, 0, 0, 0, 0],
        };
        let mut add_blocks = SyncBlocks::default();
        add_blocks.add_block(appearance.into());
        add_blocks.add_block(force_chat.into());
        let add_player = PlayerUpdate::Add(
            AddPlayer {
                id: 1,
                dx: 0,
                dy: 0,
            },
            add_blocks,
        );

        let sync_packet = PlayerSynchronization {
            player_update: Some(PlayerUpdate::Update(
                Some(EntityMovement::Teleport {
                    destination: Position::default(),
                    current: Position::default(),
                    changed_region: false,
                }),
                my_blocks,
            )),
            other_players: vec![remove_player, move_player, add_player],
        };
        sync_packet
            .try_write(&mut buf)
            .expect("Failed to write packet");
        assert_eq!(&buf[..], &PACKET[..]);
    }
}
