use std::any::Any;

use ahash::AHashMap;
use bytes::BytesMut;
use num_derive::ToPrimitive;
use once_cell::sync::Lazy;
use strum_macros::EnumIter;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct PacketId {
    pub id: u8,
    pub stage: PacketStage,
    pub direction: PacketDirection,
}

impl PacketId {
    pub fn new(id: u8, direction: PacketDirection, stage: PacketStage) -> Self {
        Self {
            id,
            stage,
            direction,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum PacketDirection {
    Clientbound,
    Serverbound,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum PacketStage {
    Handshake,
    Gameplay,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumIter, ToPrimitive)]
pub enum PacketType {
    // Handshake
    HandshakeHello,
    HandshakeExchangeKey,
    HandshakeAttemptConnect,
    HandshakeConnectResponse,

    // region Gameplay - Serverbound
    KeepAlive,
    FocusUpdate,
    PublicChat,
    ThirdItemOption,
    ThirdNpcAction,
    FifthNpcAction,
    FourthNpcAction,
    FifthPlayerAction,
    DialogueContinue,
    SecondItemOption,
    ThirdItemAction,
    FlaggedMouseEvent,
    ItemOnItem,
    ItemOnNpc,
    ThirdObjectAction,
    SecondNpcAction,
    ThirdPlayerAction,
    RemoveIgnore,
    FourthItemOption,
    SpamPacket,
    ArrowKey,
    FifthItemOption,
    PrivacyOption,
    PlayerDesign,
    Command,
    SecondItemAction,
    FlashingTabClicked,
    FirstItemOption,
    PrivateChat,
    FirstPlayerAction,
    FourthItemAction,
    ClosedInterface,
    MagicOnNpc,
    FirstObjectAction,
    AddIgnore,
    FifthItemAction,
    FourthPlayerAction,
    FirstItemAction,
    SecondPlayerAction,
    FirstNpcAction,
    Button,
    AddFriend,
    ItemOnObject,
    EnteredAmount,
    SwitchItem,
    RemoveFriend,
    ReportAbuse,
    TakeTileItem,
    MagicOnItem,
    MouseClicked,
    Walk,
    WalkWithAnticheat,
    MagicOnPlayer,
    SecondObjectAction,
    // endregion

    // region Gameplay - Clientbound
    ForwardPrivateChat,
    OpenOverlay,
    SetWidgetItemModel,
    SendObject,
    ServerMessage,
    GroupedRegionUpdate,
    RemoveObject,
    SetUpdatedRegion,
    RemoveTileItem,
    Logout,
    OpenInterface,
    SendFriend,
    Config,
    UpdateRunEnergy,
    ClearRegion,
    SetWidgetModel,
    NpcSynchronization,
    SetPlayerAction,
    SetWidgetVisibility,
    AddGlobalTileItem,
    DisplayTabInterface,
    CloseInterface,
    SetWidgetPlayerModel,
    PositionHintIcon,
    RegionChange,
    EnterAmount,
    UpdateSlottedItems,
    SetWidgetText,
    UpdateTileItem,
    IdAssignment,
    OpenDialogueInterface,
    UpdateItems,
    IgnoreList,
    SetWidgetNpcModel,
    FriendServerStatus,
    AddTileItem,
    DisplayCrossbones,
    PlayerSynchronization,
    SetWidgetModelAnimation,
    OpenInterfaceSidebar,
    FlashTabInterface,
    UpdateSkill,
    UpdateWeight,
    MobHintIcon,
    SwitchTabInterface,
    OpenDialogueOverlay,
    OpenSidebar,
    // endregion
}

static PACKET_ID_MAP: Lazy<AHashMap<PacketId, PacketType>> = Lazy::new(|| {
    let mut packets = AHashMap::new();

    // Handshake
    packets.insert(
        PacketId::new(14, PacketDirection::Serverbound, PacketStage::Handshake),
        PacketType::HandshakeHello,
    );
    packets.insert(
        PacketId::new(0, PacketDirection::Clientbound, PacketStage::Handshake),
        PacketType::HandshakeExchangeKey,
    );
    packets.insert(
        PacketId::new(16, PacketDirection::Serverbound, PacketStage::Handshake),
        PacketType::HandshakeAttemptConnect,
    );
    packets.insert(
        PacketId::new(18, PacketDirection::Serverbound, PacketStage::Handshake),
        PacketType::HandshakeAttemptConnect,
    );
    packets.insert(
        PacketId::new(2, PacketDirection::Clientbound, PacketStage::Handshake),
        PacketType::HandshakeConnectResponse,
    );

    // region Gameplay serverbound packet definitions
    packets.insert(
        PacketId::new(0, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::KeepAlive,
    );
    packets.insert(
        PacketId::new(3, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::FocusUpdate,
    );
    packets.insert(
        PacketId::new(4, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::PublicChat,
    );
    packets.insert(
        PacketId::new(16, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::ThirdItemOption,
    );
    packets.insert(
        PacketId::new(17, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::ThirdNpcAction,
    );
    packets.insert(
        PacketId::new(18, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::FifthNpcAction,
    );
    packets.insert(
        PacketId::new(21, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::FourthNpcAction,
    );
    packets.insert(
        PacketId::new(39, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::FifthPlayerAction,
    );
    packets.insert(
        PacketId::new(40, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::DialogueContinue,
    );
    packets.insert(
        PacketId::new(41, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::SecondItemOption,
    );
    packets.insert(
        PacketId::new(43, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::ThirdItemAction,
    );
    packets.insert(
        PacketId::new(45, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::FlaggedMouseEvent,
    );
    packets.insert(
        PacketId::new(53, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::ItemOnItem,
    );
    packets.insert(
        PacketId::new(57, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::ItemOnNpc,
    );
    packets.insert(
        PacketId::new(70, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::ThirdObjectAction,
    );
    packets.insert(
        PacketId::new(72, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::SecondNpcAction,
    );
    packets.insert(
        PacketId::new(73, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::ThirdPlayerAction,
    );
    packets.insert(
        PacketId::new(74, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::RemoveIgnore,
    );
    packets.insert(
        PacketId::new(75, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::FourthItemOption,
    );
    packets.insert(
        PacketId::new(77, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::SpamPacket,
    );
    packets.insert(
        PacketId::new(78, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::SpamPacket,
    );
    packets.insert(
        PacketId::new(86, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::ArrowKey,
    );
    packets.insert(
        PacketId::new(87, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::FifthItemOption,
    );
    packets.insert(
        PacketId::new(95, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::PrivacyOption,
    );
    packets.insert(
        PacketId::new(98, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::Walk,
    );
    packets.insert(
        PacketId::new(101, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::PlayerDesign,
    );
    packets.insert(
        PacketId::new(103, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::Command,
    );
    packets.insert(
        PacketId::new(117, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::SecondItemAction,
    );
    packets.insert(
        PacketId::new(120, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::FlashingTabClicked,
    );
    packets.insert(
        PacketId::new(121, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::SpamPacket,
    );
    packets.insert(
        PacketId::new(122, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::FirstItemOption,
    );
    packets.insert(
        PacketId::new(126, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::PrivateChat,
    );
    packets.insert(
        PacketId::new(128, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::FirstPlayerAction,
    );
    packets.insert(
        PacketId::new(129, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::FourthItemAction,
    );
    packets.insert(
        PacketId::new(130, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::ClosedInterface,
    );
    packets.insert(
        PacketId::new(131, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::MagicOnNpc,
    );
    packets.insert(
        PacketId::new(132, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::FirstObjectAction,
    );
    packets.insert(
        PacketId::new(133, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::AddIgnore,
    );
    packets.insert(
        PacketId::new(135, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::FifthItemAction,
    );
    packets.insert(
        PacketId::new(139, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::FourthPlayerAction,
    );
    packets.insert(
        PacketId::new(145, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::FirstItemAction,
    );
    packets.insert(
        PacketId::new(153, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::SecondPlayerAction,
    );
    packets.insert(
        PacketId::new(155, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::FirstNpcAction,
    );
    packets.insert(
        PacketId::new(164, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::Walk,
    );
    packets.insert(
        PacketId::new(165, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::SpamPacket,
    );
    packets.insert(
        PacketId::new(185, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::Button,
    );
    packets.insert(
        PacketId::new(188, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::AddFriend,
    );
    packets.insert(
        PacketId::new(189, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::SpamPacket,
    );
    packets.insert(
        PacketId::new(192, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::ItemOnObject,
    );
    packets.insert(
        PacketId::new(208, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::EnteredAmount,
    );
    packets.insert(
        PacketId::new(210, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::SpamPacket,
    );
    packets.insert(
        PacketId::new(214, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::SwitchItem,
    );
    packets.insert(
        PacketId::new(215, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::RemoveFriend,
    );
    packets.insert(
        PacketId::new(218, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::ReportAbuse,
    );
    packets.insert(
        PacketId::new(226, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::SpamPacket,
    );
    packets.insert(
        PacketId::new(236, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::TakeTileItem,
    );
    packets.insert(
        PacketId::new(237, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::MagicOnItem,
    );
    packets.insert(
        PacketId::new(241, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::MouseClicked,
    );
    packets.insert(
        PacketId::new(248, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::WalkWithAnticheat,
    );
    packets.insert(
        PacketId::new(249, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::MagicOnPlayer,
    );
    packets.insert(
        PacketId::new(252, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::SecondObjectAction,
    );
    // endregion

    // region Gameplay clientbound packets
    packets.insert(
        PacketId::new(206, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::PrivacyOption,
    );
    packets.insert(
        PacketId::new(249, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::IdAssignment,
    );
    //endregion

    packets
});

static PACKET_TYPE_MAP: Lazy<AHashMap<PacketType, PacketId>> = Lazy::new(|| {
    let mut packets = AHashMap::new();
    for (key, value) in PACKET_ID_MAP.clone() {
        packets.insert(value, key);
    }
    packets
});

impl PacketType {
    pub fn get_from_id(packet_id: PacketId) -> Option<PacketType> {
        PACKET_ID_MAP.get(&packet_id).copied()
    }

    pub fn get_id(&self) -> PacketId {
        *PACKET_TYPE_MAP
            .get(&self)
            .unwrap_or_else(|| panic!("cannot find ID for packet type {:?}", self))
    }

    pub fn create(&self) -> anyhow::Result<Box<dyn Packet>> {
        match crate::packets::PACKET_FACTORIES.get(&self) {
            None => anyhow::bail!("packet factory does not exist"),
            Some(factory) => Ok(factory.create()),
        }
    }

    pub fn is_variable_length(&self) -> bool {
        match self {
            PacketType::PublicChat => true,
            PacketType::FlaggedMouseEvent => true,
            PacketType::SpamPacket => true,
            PacketType::Walk => true,
            PacketType::Command => true,
            PacketType::PrivateChat => true,
            _ => false,
        }
    }
}

pub struct PacketFactory {
    init_fn: fn() -> Box<dyn Packet>,
}

impl PacketFactory {
    pub fn new(init_fn: fn() -> Box<dyn Packet>) -> PacketFactory {
        Self { init_fn }
    }

    pub fn create(&self) -> Box<dyn Packet> {
        (self.init_fn)()
    }
}

pub trait Packet: Send + Sync + IntoAny {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        unimplemented!()
    }

    fn try_write(&self, src: &mut BytesMut) -> anyhow::Result<()> {
        unimplemented!()
    }

    fn get_type(&self) -> PacketType;
}

pub fn cast_packet<P: Packet + 'static + Send>(packet: Box<dyn Packet>) -> P {
    *packet.into_any().downcast().unwrap()
}

pub trait IntoAny {
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T> IntoAny for T
where
    T: Any,
{
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}
