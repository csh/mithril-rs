use crate::{Packet, PacketType};
use bytes::buf::{Buf, BufMut};
use bytes::BytesMut;
use mithril_buf::Transform;
use mithril_buf::{GameBuf, GameBufMut};
use mithril_pos::{Direction, Position};

#[derive(Debug)]
pub struct RemoveObject {
    position: Position,
    object_type: u8,
    orientation: Direction,
}

impl RegionUpdatePacket for RemoveObject {}

impl Packet for RemoveObject {
    fn try_write(&self, buffer: &mut BytesMut) -> anyhow::Result<()> {
        let orientation = self.orientation.to_orientation()?;
        let data = self.object_type << 2 | orientation & 0x3;
        buffer.put_u8t(data, Transform::Negate);
        buffer.put_u8(to_offset(&self.position));
        Ok(())
    }

    fn get_type(&self) -> PacketType {
        PacketType::RemoveObject
    }
}

#[derive(Debug, Packet)]
pub struct RemoveTileItem {
    #[transform = "add"]
    position_offset: u8,
    id: u16,
}

impl RegionUpdatePacket for RemoveTileItem {}

impl RemoveTileItem {
    pub fn new(position: &Position, id: u16) -> Self {
        RemoveTileItem {
            position_offset: to_offset(&position),
            id,
        }
    }
}

#[derive(Debug, Packet)]
pub struct AddTileItem {
    #[endian = "little"]
    #[transform = "add"]
    id: u16,
    amount: u16,
    position_offset: u8,
}

impl RegionUpdatePacket for AddTileItem {}

impl AddTileItem {
    pub fn new(id: u16, amount: u16, position: &Position) -> Self {
        AddTileItem {
            id,
            amount,
            position_offset: to_offset(&position),
        }
    }
}

#[derive(Debug, Packet)]
pub struct SendObject {
    #[transform = "add"]
    position_offset: u8,
    #[endian = "little"]
    id: u16,
    #[transform = "subtract"]
    type_and_orientation: u8,
}

impl RegionUpdatePacket for SendObject {}

impl SendObject {
    pub fn new(
        id: u16,
        object_type: u8,
        orientation: Direction,
        position: &Position,
    ) -> anyhow::Result<Self> {
        let type_and_orientation = object_type << 2 | orientation.to_orientation()? & 0x3;
        Ok(SendObject {
            position_offset: to_offset(position),
            id,
            type_and_orientation,
        })
    }
}

#[derive(Debug, Packet)]
pub struct AddGlobalTileItem {
    #[transform = "add"]
    id: u16,
    #[transform = "subtract"]
    position_offset: u8,
    #[transform = "add"]
    owner: u16,
    amount: u16,
}

impl RegionUpdatePacket for AddGlobalTileItem {}

impl AddGlobalTileItem {
    pub fn new(id: u16, position: &Position, owner: u16, amount: u16) -> Self {
        AddGlobalTileItem {
            id,
            position_offset: to_offset(position),
            owner,
            amount,
        }
    }
}

#[derive(Debug, Packet)]
pub struct UpdateTileItem {
    position_offset: u8,
    id: u16,
    old_amount: u16,
    amount: u16,
}

impl RegionUpdatePacket for UpdateTileItem {}

impl UpdateTileItem {
    pub fn new(id: u16, position: &Position, old_amount: u16, amount: u16) -> Self {
        UpdateTileItem {
            position_offset: to_offset(position),
            id,
            old_amount,
            amount,
        }
    }
}

fn to_offset(position: &Position) -> u8 {
    let dx = (position.get_x() % 8) as u8;
    let dy = (position.get_y() % 8) as u8;
    dx << 4 | dy & 0x7
}

#[derive(Debug)]
pub struct GroupedRegionUpdate {
    pub position: Position,
    pub updates: Vec<Box<dyn RegionUpdatePacket>>,
}

pub trait RegionUpdatePacket: Packet + std::fmt::Debug {}

impl Packet for GroupedRegionUpdate {
    fn try_write(&self, buffer: &mut BytesMut) -> anyhow::Result<()> {
        let dx = (self.position.get_x() / 8 * 8) as u8;
        let dy = (self.position.get_y() / 8 * 8) as u8;
        buffer.put_u8(dy);
        buffer.put_u8t(dx, Transform::Negate);
        let result: Result<Vec<_>, _> = self
            .updates
            .iter()
            .map(|packet| {
                buffer.put_u8(packet.get_type().get_id().id);
                packet.try_write(buffer)
            })
            .collect();
        result.map(|_| ())
    }

    fn get_type(&self) -> PacketType {
        PacketType::GroupedRegionUpdate
    }
}
