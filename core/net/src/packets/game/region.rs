use crate::{Packet, PacketType};
use bytes::buf::{Buf, BufMut};
use bytes::BytesMut;
use mithril_buf::Transform;
use mithril_buf::{GameBuf, GameBufMut};
use mithril_pos::{Direction, Position, Region};

#[derive(Debug, Clone, Copy)]
pub enum ObjectType {
    LengthwiseWall = 0,
    TriangularCorner = 1,
    WallCorner = 2,
    RectangularCorner = 3,
    DiagonalWall = 9,
    Interactable = 10,
    DiagonalInteractable = 11,
    FloorDecoration = 12,
}

#[derive(Debug, Packet, PartialEq)]
pub struct RemoveObject {
    #[transform = "negate"]
    type_and_orientation: u8,
    position_offset: u8,
}

impl RemoveObject {
    pub fn new(
        object_type: ObjectType,
        orientation: Direction,
        position: &Position,
    ) -> anyhow::Result<Self> {
        let type_and_orientation =
            ((object_type as u8) << 2) | (orientation.to_orientation()? & 0x3);
        dbg!(type_and_orientation);
        Ok(RemoveObject {
            type_and_orientation,
            position_offset: to_offset(position),
        })
    }
}

#[derive(Debug, Packet, PartialEq)]
pub struct RemoveTileItem {
    #[transform = "add"]
    position_offset: u8,
    id: u16,
}

impl RemoveTileItem {
    pub fn new(item: u16, position: &Position) -> Self {
        RemoveTileItem {
            position_offset: to_offset(&position),
            id: item,
        }
    }
}

#[derive(Debug, Packet, PartialEq)]
pub struct AddTileItem {
    #[endian = "little"]
    #[transform = "add"]
    id: u16,
    amount: u16,
    position_offset: u8,
}

impl AddTileItem {
    pub fn new(item: u16, amount: u16, position: &Position) -> Self {
        AddTileItem {
            id: item,
            amount,
            position_offset: to_offset(&position),
        }
    }
}

#[derive(Debug, Packet, PartialEq)]
pub struct SendObject {
    #[transform = "add"]
    position_offset: u8,
    #[endian = "little"]
    id: u16,
    #[transform = "subtract"]
    type_and_orientation: u8,
}

impl SendObject {
    pub fn new(
        id: u16,
        object_type: ObjectType,
        orientation: Direction,
        position: &Position,
    ) -> anyhow::Result<Self> {
        let type_and_orientation = (object_type as u8) << 2 | orientation.to_orientation()? & 0x3;
        Ok(SendObject {
            position_offset: to_offset(position),
            id,
            type_and_orientation,
        })
    }
}

#[derive(Debug, Packet, PartialEq)]
pub struct AddGlobalTileItem {
    #[transform = "add"]
    id: u16,
    #[transform = "subtract"]
    position_offset: u8,
    #[transform = "add"]
    owner: u16,
    amount: u16,
}

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

#[derive(Debug, Packet, PartialEq)]
pub struct UpdateTileItem {
    position_offset: u8,
    id: u16,
    old_amount: u16,
    amount: u16,
}

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

#[derive(Debug, EventFromPacket, PartialEq)]
pub struct GroupedRegionUpdate {
    pub region: Region,
    pub viewport_center: Position,
    pub updates: Vec<RegionUpdate>,
}

impl GroupedRegionUpdate { 
    pub fn new(position: Position, region: Region) -> Self {
        GroupedRegionUpdate {
            region,
            viewport_center: position,
            updates: vec![],
        }
    }

    pub fn add_all(mut self, updates: Vec<RegionUpdate>) -> Self {
        self.updates.extend(updates);
        self
    }

    pub fn add_update<T : Into<RegionUpdate>>(mut self, update: T) -> Self {
        self.updates.push(update.into());
        self    
    }
}

impl Packet for GroupedRegionUpdate {
    fn try_write(&self, buffer: &mut BytesMut) -> anyhow::Result<()> {
        let vx = self.viewport_center.get_x() / 8 - 6;
        let vy = self.viewport_center.get_y() / 8 - 6;
        let dx = ((self.region.x - vx) * 8) as u8;
        let dy = ((self.region.y - vy) * 8) as u8;
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

#[derive(Debug, PartialEq)]
pub enum RegionUpdate {
    RemoveObject(RemoveObject),
    RemoveTileItem(RemoveTileItem),
    AddTileItem(AddTileItem),
    SendObject(SendObject),
    AddGlobalTileItem(AddGlobalTileItem),
    UpdateTileItem(UpdateTileItem)
}

macro_rules! into_regionupdate {
    ($update:ident) => {
        impl From<$update> for RegionUpdate {
            fn from(packet: $update) -> Self {
                RegionUpdate::$update(packet)
            }    
        }
    };
}

into_regionupdate!(RemoveObject);
into_regionupdate!(RemoveTileItem);
into_regionupdate!(AddTileItem);
into_regionupdate!(SendObject);
into_regionupdate!(AddGlobalTileItem);
into_regionupdate!(UpdateTileItem);

impl Packet for RegionUpdate {
    fn try_write(&self, buffer: &mut BytesMut) -> anyhow::Result<()> {
        match self {
            Self::RemoveObject(packet) => packet.try_write(buffer),
            Self::RemoveTileItem(packet) => packet.try_write(buffer),
            Self::AddTileItem(packet) => packet.try_write(buffer),
            Self::SendObject(packet) => packet.try_write(buffer),
            Self::AddGlobalTileItem(packet) => packet.try_write(buffer),
            Self::UpdateTileItem(packet) => packet.try_write(buffer),
        }

    }

    fn get_type(&self) -> PacketType {
        match self {
            Self::RemoveObject(packet) => packet.get_type(),
            Self::RemoveTileItem(packet) => packet.get_type(),
            Self::AddTileItem(packet) => packet.get_type(),
            Self::SendObject(packet) => packet.get_type(),
            Self::AddGlobalTileItem(packet) => packet.get_type(),
            Self::UpdateTileItem(packet) => packet.get_type(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_object() {
        const PACKET: [u8; 2] = [0xd9, 0x50];
        let mut buf = BytesMut::new();
        RemoveObject::new(
            ObjectType::DiagonalWall,
            Direction::South,
            &Position::default(),
        )
        .expect("Direction should be none")
        .try_write(&mut buf)
        .expect("Write failed?");

        assert_eq!(&buf[..], &PACKET[..]);
    }

    #[test]
    fn test_remove_tile_item() {
        const PACKET: [u8; 3] = [0xd0, 0x00, 0x14];
        let mut buf = BytesMut::new();
        RemoveTileItem::new(20, &Position::default())
            .try_write(&mut buf)
            .expect("Write failed?");
        assert_eq!(&buf[..], &PACKET[..]);
    }

    #[test]
    fn test_add_tile_item() {
        const PACKET: [u8; 5] = [0x94, 0x00, 0x00, 0x01, 0x50];
        let mut buf = BytesMut::new();
        AddTileItem::new(20, 1, &Position::default())
            .try_write(&mut buf)
            .expect("Write failed?");
        assert_eq!(&buf[..], &PACKET[..]);
    }

    #[test]
    fn test_send_object() {
        const PACKET: [u8; 4] = [0xd0, 0x01, 0x00, 0x59];
        let mut buf = BytesMut::new();
        SendObject::new(
            1,
            ObjectType::DiagonalWall,
            Direction::South,
            &Position::default(),
        )
        .expect("Direction was none")
        .try_write(&mut buf)
        .expect("Write failed");
        assert_eq!(&buf[..], &PACKET[..]);
    }

    #[test]
    fn test_add_global_tile_item() {
        const PACKET: [u8; 7] = [0x00, 0x94, 0x30, 0x00, 0x83, 0x00, 0x01];
        let mut buf = BytesMut::new();
        AddGlobalTileItem::new(20, &Position::default(), 3, 1)
            .try_write(&mut buf)
            .expect("Write failed?");
        assert_eq!(&buf[..], &PACKET[..]);
    }

    #[test]
    fn test_update_tile_item() {
        const PACKET: [u8; 7] = [0x50, 0x00, 0x14, 0x00, 0x05, 0x00, 0x01];
        let mut buf = BytesMut::new();
        UpdateTileItem::new(20, &Position::default(), 5, 1)
            .try_write(&mut buf)
            .expect("Write failed?");
        assert_eq!(&buf[..], &PACKET[..]);
    }

    #[test]
    fn test_empty_grouped_update() {
        const PACKET: [u8; 2] = [0x30, 0xd0];
        let mut buf = BytesMut::new();
        GroupedRegionUpdate {
            region: (&Position::default()).into(),
            viewport_center: Position::default(),
            updates: vec![],
        }
        .try_write(&mut buf)
        .expect("Write failed?");

        assert_eq!(&buf[..], &PACKET[..]);
    }

    #[test]
    fn test_grouped_update() {
        const PACKET: [u8; 13] = [
            0x30, 0xd0, 0xd7, 0x00, 0x94, 0x30, 0x00, 0x83, 0x00, 0x01, 0x65, 0xd7, 0x50,
        ];
        let mut buf = BytesMut::new();

        let add_global = AddGlobalTileItem::new(20, &Position::default(), 3, 1);
        let remove_obj = RemoveObject::new(
            ObjectType::Interactable,
            Direction::North,
            &Position::default(),
        )
        .expect("Direction was none");

        GroupedRegionUpdate {
            region: (&Position::default()).into(),
            viewport_center: Position::default(),
            updates: vec![add_global.into(), remove_obj.into()],
        }
        .try_write(&mut buf)
        .expect("Write failed?");

        assert_eq!(&buf[..], &PACKET[..]);
    }
}
