use crate::{Packet, PacketType};
use ahash::AHashMap;
use bytes::buf::BufMut;
use bytes::{Bytes, BytesMut};
use mithril_buf::{BitWriter, GameBufMut, Transform};
use mithril_pos::Position;
use mithril_text::compress;
use std::convert::TryInto;
use std::fmt;

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum SyncBlockType {
    ForceMovement,
    Graphic,
    Animation,
    ForceChat,
    Chat,
    InteractingMob,
    Appearance,
    TurnToPosition,
    HitUpdate,
    SecondaryHitUpdate,
}

pub trait SyncBlock: Send + Sync + fmt::Debug {
    fn get_type(&self) -> SyncBlockType;
    fn write(&self, buffer: &mut BytesMut);
}

#[derive(Debug)]
pub struct Animation {
    id: u16,
    delay: u8,
}

impl SyncBlock for Animation {
    fn get_type(&self) -> SyncBlockType {
        SyncBlockType::Animation
    }

    fn write(&self, buf: &mut BytesMut) {
        buf.put_u16_le(self.id);
        buf.put_u8t(self.delay, Transform::Negate);
    }
}

#[derive(Debug)]
pub struct Item {
    id: u16,
}

#[derive(Debug)]
pub struct Equipment {
    chest: Option<Item>,
    shield: Option<Item>,
    legs: Option<Item>,
    hat: Option<Item>,
    hands: Option<Item>,
    feet: Option<Item>,
}

impl Equipment {
    #[allow(unused_variables)]
    fn get_slot(&self, slot: u8) -> Option<Item> {
        None
    }
}

#[derive(Debug)]
pub enum AppearanceType {
    Npc(u16),
    Player(Equipment, Vec<u16>),
}

#[derive(Debug)]
pub struct Appearance {
    name: String,
    gender: u8,
    combat_level: u8,
    skill_level: u16,
    appearance_type: AppearanceType,
    colours: Vec<u8>, // Enums are cool I tell you!
}

impl SyncBlock for Appearance {
    fn get_type(&self) -> SyncBlockType {
        SyncBlockType::Appearance
    }

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
        buf2.put_u64(0); // encode name here!
        buf2.put_u8(self.combat_level);
        buf2.put_u16(self.skill_level);
        // Of course, buf and buf2 are actually different :P
        let props_len = buf2.len();
        buf.put_u8t(props_len as u8, Transform::Negate);
        buf.put(buf2);
    }
}

impl Appearance {
    fn write_appearance(
        &self,
        buf: &mut BytesMut,
        gender: u8,
        equipment: &Equipment,
        style: &[u16],
    ) {
        for i in 0..4 {
            if let Some(item) = equipment.get_slot(i) {
                buf.put_u16(0x200 + item.id);
            } else {
                buf.put_u8(0);
            }
        }

        buf.put_u16(if let Some(item) = &equipment.chest {
            (0x200 + item.id) as u16
        } else {
            (0x100 + style[2]) as u16
        });

        if let Some(item) = &equipment.shield {
            buf.put_u16(0x200 + item.id);
        } else {
            buf.put_u8(0);
        }

        if let Some(_item) = &equipment.chest {
            // && item.is_full_body() {
            buf.put_u8(0);
        } else {
            buf.put_u16(0x100 + style[3]);
        }

        buf.put_u16(if let Some(item) = &equipment.legs {
            (0x200 + item.id) as u16
        } else {
            (0x100 + style[5]) as u16
        });

        if let Some(_item) = &equipment.hat {
            // && (item.is_full_hat() || item.is_full_mask()) {
            buf.put_u8(0);
        } else {
            buf.put_u16(0x100 + style[0]);
        }

        buf.put_u16(if let Some(item) = &equipment.hands {
            (0x200 + item.id) as u16
        } else {
            (0x100 + style[4]) as u16
        });

        buf.put_u16(if let Some(item) = &equipment.feet {
            (0x200 + item.id) as u16
        } else {
            (0x100 + style[6]) as u16
        });

        if gender != 0 {
            buf.put_u8(0);
        } else if let Some(_item) = &equipment.hat {
            // && item.is_full_mask(){
            buf.put_u8(0);
        } else {
            buf.put_u16(0x100 + style[1]);
        }
    }
}

#[derive(Debug)]
pub struct Chat {
    message: String,
    color: u8,           // TODO: EnumSet pls
    effects: u8,         // TODO: EnumSet pls
    privilege_level: u8, // TODO: Enum pls
}

impl SyncBlock for Chat {
    fn get_type(&self) -> SyncBlockType {
        SyncBlockType::Chat
    }

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

#[derive(Debug)]
pub struct ForceChat {
    message: String,
}

impl SyncBlock for ForceChat {
    fn get_type(&self) -> SyncBlockType {
        SyncBlockType::ForceChat
    }

    fn write(&self, buf: &mut BytesMut) {
        buf.put_rs_string(self.message.clone());
    }
}

#[derive(Debug)]
pub struct ForceMovement {
    initial_pos: (u8, u8),
    final_pos: (u8, u8),
    travel_duration: (u16, u16),
    direction: u8,
}

impl SyncBlock for ForceMovement {
    fn get_type(&self) -> SyncBlockType {
        SyncBlockType::ForceMovement
    }

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

#[derive(Debug)]
pub struct Graphic {
    id: u16,
    height: u16,
    delay: u16,
}

impl SyncBlock for Graphic {
    fn get_type(&self) -> SyncBlockType {
        SyncBlockType::Graphic
    }

    fn write(&self, buf: &mut BytesMut) {
        buf.put_u16_le(self.id);
        buf.put_u32((self.height as u32) << 16 | self.delay as u32);
    }
}

#[derive(Debug)]
pub struct HitUpdate {
    damage: u8,
    damage_type: u8,
    health: u8,
    max_health: u8,
}

impl SyncBlock for HitUpdate {
    fn get_type(&self) -> SyncBlockType {
        SyncBlockType::HitUpdate
    }

    fn write(&self, buf: &mut BytesMut) {
        buf.put_u8(self.damage);
        buf.put_u8t(self.damage_type, Transform::Add);
        buf.put_u8t(self.health, Transform::Negate);
        buf.put_u8(self.max_health);
    }
}

#[derive(Debug)]
pub struct InteractingMob {
    index: u16,
}

impl SyncBlock for InteractingMob {
    fn get_type(&self) -> SyncBlockType {
        SyncBlockType::InteractingMob
    }

    fn write(&self, buf: &mut BytesMut) {
        buf.put_u16_le(self.index);
    }
}

#[derive(Debug)]
pub struct SecondaryHitUpdate {
    damage: u8,
    damage_type: u8,
    health: u8,
    max_health: u8,
}

impl SyncBlock for SecondaryHitUpdate {
    fn get_type(&self) -> SyncBlockType {
        SyncBlockType::SecondaryHitUpdate
    }

    fn write(&self, buf: &mut BytesMut) {
        buf.put_u8(self.damage);
        buf.put_u8t(self.damage_type, Transform::Subtract);
        buf.put_u8(self.health);
        buf.put_u8t(self.max_health, Transform::Negate);
    }
}

#[derive(Debug)]
pub struct TurnToPosition {
    position: (u16, u16),
}

impl SyncBlock for TurnToPosition {
    fn get_type(&self) -> SyncBlockType {
        SyncBlockType::TurnToPosition
    }

    fn write(&self, buf: &mut BytesMut) {
        buf.put_u16t_le(self.position.0 * 2 + 1, Transform::Add);
        buf.put_u16_le(self.position.1 * 2 + 1);
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
    Run {
        directions: (i32, i32),
    },
}

#[derive(Debug)]
pub struct AddPlayer {
    id: u16,
    // From player
    dx: u8,
    // From player
    dy: u8,
}

impl SyncBlockType {
    fn to_id(&self) -> u16 {
        match self {
            Self::ForceMovement => 0x400,
            Self::Graphic => 0x100,
            Self::Animation => 0x8,
            Self::ForceChat => 0x4,
            Self::Chat => 0x80,
            Self::InteractingMob => 0x1,
            Self::Appearance => 0x10,
            Self::TurnToPosition => 0x2,
            Self::HitUpdate => 0x20,
            Self::SecondaryHitUpdate => 0x200,
        }
    }
}

#[derive(Debug, Default)]
pub struct SyncBlocks {
    blocks: AHashMap<SyncBlockType, Box<dyn SyncBlock>>,
}

macro_rules! send_sync_block {
    ($block:ident, $map:expr, $buf:ident) => {
        if let Some(value) = $map.get(&SyncBlockType::$block) {
            value.write($buf);
        }
    };
}

impl SyncBlocks {
    pub fn add_block(&mut self, block: Box<dyn SyncBlock>) -> &mut Self {
        self.blocks.insert(block.get_type(), block);
        self
    }

    fn write(&self, buf: &mut BytesMut) {
        let mask: u16 = self
            .blocks
            .keys()
            .map(|t| t.to_id())
            .fold(0, |acc, val| acc | val);
        if mask >= 0x100 {
            let mask = mask | 0x40;
            buf.put_u16_le(mask);
        } else {
            buf.put_u8(mask as u8);
        }

        send_sync_block!(ForceMovement, self.blocks, buf);
        send_sync_block!(Graphic, self.blocks, buf);
        send_sync_block!(Animation, self.blocks, buf);
        send_sync_block!(ForceChat, self.blocks, buf);
        send_sync_block!(Chat, self.blocks, buf);
        send_sync_block!(InteractingMob, self.blocks, buf);
        send_sync_block!(Appearance, self.blocks, buf);
        send_sync_block!(TurnToPosition, self.blocks, buf);
        send_sync_block!(HitUpdate, self.blocks, buf);
        send_sync_block!(SecondaryHitUpdate, self.blocks, buf);
    }

    fn has_updates(&self) -> bool {
        !self.blocks.is_empty()
    }
}

#[derive(Debug)]
pub enum PlayerUpdate {
    Remove(),
    Add(AddPlayer, SyncBlocks),
    Update(Option<EntityMovement>, SyncBlocks),
}

#[derive(Debug)]
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
            writer.put_bits(8, self.other_players.len() as u32);
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
