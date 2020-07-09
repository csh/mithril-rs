use crate::packets::PacketEvent;
use ahash::AHashMap;
use bytes::BytesMut;
use once_cell::sync::Lazy;

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

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
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
    SpamPacket(PacketLength),
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
    ConfigByte,
    ConfigInt,
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

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum PacketLength {
    Fixed(usize),
    VariableByte,
    VariableShort,
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
        PacketType::SpamPacket(PacketLength::VariableByte),
    );
    packets.insert(
        PacketId::new(78, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::SpamPacket(PacketLength::Fixed(0)),
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
        PacketType::SpamPacket(PacketLength::Fixed(0)),
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
        PacketType::SpamPacket(PacketLength::VariableByte),
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
        PacketType::SpamPacket(PacketLength::Fixed(1)),
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
        PacketType::SpamPacket(PacketLength::Fixed(4)),
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
        PacketType::SpamPacket(PacketLength::VariableByte),
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
        PacketId::new(8, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::SetWidgetModel,
    );
    packets.insert(
        PacketId::new(27, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::EnterAmount,
    );
    packets.insert(
        PacketId::new(44, PacketDirection::Serverbound, PacketStage::Gameplay),
        PacketType::AddTileItem,
    );
    packets.insert(
        PacketId::new(60, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::GroupedRegionUpdate,
    );
    packets.insert(
        PacketId::new(61, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::DisplayCrossbones,
    );
    packets.insert(
        PacketId::new(64, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::ClearRegion,
    );
    packets.insert(
        PacketId::new(65, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::NpcSynchronization,
    );
    packets.insert(
        PacketId::new(71, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::SwitchTabInterface,
    );
    packets.insert(
        PacketId::new(73, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::RegionChange,
    );
    packets.insert(
        PacketId::new(75, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::SetWidgetNpcModel,
    );
    packets.insert(
        PacketId::new(81, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::PlayerSynchronization,
    );
    packets.insert(
        PacketId::new(84, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::UpdateTileItem,
    );
    packets.insert(
        PacketId::new(97, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::OpenInterface,
    );
    packets.insert(
        PacketId::new(101, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::RemoveObject,
    );
    packets.insert(
        PacketId::new(104, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::SetPlayerAction,
    );
    packets.insert(
        PacketId::new(106, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::DisplayTabInterface,
    );
    packets.insert(
        PacketId::new(109, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::Logout,
    );
    packets.insert(
        PacketId::new(110, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::UpdateRunEnergy,
    );
    packets.insert(
        PacketId::new(126, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::SetWidgetText,
    );
    packets.insert(
        PacketId::new(134, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::UpdateSkill,
    );
    packets.insert(
        PacketId::new(151, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::SendObject,
    );
    packets.insert(
        PacketId::new(156, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::RemoveTileItem,
    );
    packets.insert(
        PacketId::new(164, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::OpenDialogueInterface,
    );
    packets.insert(
        PacketId::new(171, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::SetWidgetVisibility,
    );
    packets.insert(
        PacketId::new(185, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::SetWidgetPlayerModel,
    );
    packets.insert(
        PacketId::new(200, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::SetWidgetModelAnimation,
    );
    packets.insert(
        PacketId::new(206, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::PrivacyOption,
    );
    packets.insert(
        PacketId::new(215, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::AddGlobalTileItem,
    );
    packets.insert(
        PacketId::new(219, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::CloseInterface,
    );
    packets.insert(
        PacketId::new(240, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::UpdateWeight,
    );
    packets.insert(
        PacketId::new(246, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::SetWidgetItemModel,
    );
    packets.insert(
        PacketId::new(248, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::OpenInterfaceSidebar,
    );
    packets.insert(
        PacketId::new(249, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::IdAssignment,
    );
    packets.insert(
        PacketId::new(253, PacketDirection::Clientbound, PacketStage::Gameplay),
        PacketType::ServerMessage,
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
    pub fn iter() -> impl Iterator<Item = &'static PacketType> {
        PACKET_TYPE_MAP.keys()
    }

    pub fn get_from_id(packet_id: PacketId) -> Option<PacketType> {
        PACKET_ID_MAP.get(&packet_id).copied()
    }

    pub fn get_id(&self) -> PacketId {
        *PACKET_TYPE_MAP
            .get(&self)
            .unwrap_or_else(|| panic!("cannot find ID for packet type {:?}", self))
    }

    pub fn create(&self) -> anyhow::Result<PacketEvent> {
        match crate::packets::PACKET_FACTORIES.get(&self) {
            None => anyhow::bail!("packet factory does not exist for {:?}", &self),
            Some(factory) => Ok(factory.create()),
        }
    }

    pub fn packet_length(&self) -> Option<PacketLength> {
        match self {
            PacketType::KeepAlive => Some(PacketLength::Fixed(0)),
            PacketType::FocusUpdate => Some(PacketLength::Fixed(1)),
            PacketType::ThirdItemOption => Some(PacketLength::Fixed(6)),
            PacketType::ThirdNpcAction => Some(PacketLength::Fixed(2)),
            PacketType::FifthNpcAction => Some(PacketLength::Fixed(2)),
            PacketType::FourthNpcAction => Some(PacketLength::Fixed(2)),
            PacketType::FifthPlayerAction => Some(PacketLength::Fixed(2)),
            PacketType::DialogueContinue => Some(PacketLength::Fixed(2)),
            PacketType::SecondItemOption => Some(PacketLength::Fixed(6)),
            PacketType::ThirdItemAction => Some(PacketLength::Fixed(6)),
            PacketType::ItemOnItem => Some(PacketLength::Fixed(12)),
            PacketType::ItemOnNpc => Some(PacketLength::Fixed(8)),
            PacketType::ThirdObjectAction => Some(PacketLength::Fixed(6)),
            PacketType::SecondNpcAction => Some(PacketLength::Fixed(2)),
            PacketType::ThirdPlayerAction => Some(PacketLength::Fixed(2)),
            PacketType::RemoveIgnore => Some(PacketLength::Fixed(8)),
            PacketType::FourthItemOption => Some(PacketLength::Fixed(6)),
            PacketType::SpamPacket(len) => Some(*len),
            PacketType::ArrowKey => Some(PacketLength::Fixed(4)),
            PacketType::FifthItemOption => Some(PacketLength::Fixed(6)),
            PacketType::PrivacyOption => Some(PacketLength::Fixed(3)),
            PacketType::PlayerDesign => Some(PacketLength::Fixed(13)),
            PacketType::SecondItemAction => Some(PacketLength::Fixed(6)),
            PacketType::FlashingTabClicked => Some(PacketLength::Fixed(1)),
            PacketType::FirstItemOption => Some(PacketLength::Fixed(6)),
            PacketType::FirstPlayerAction => Some(PacketLength::Fixed(2)),
            PacketType::FourthItemAction => Some(PacketLength::Fixed(6)),
            PacketType::ClosedInterface => Some(PacketLength::Fixed(0)),
            PacketType::MagicOnNpc => Some(PacketLength::Fixed(4)),
            PacketType::FirstObjectAction => Some(PacketLength::Fixed(6)),
            PacketType::AddIgnore => Some(PacketLength::Fixed(8)),
            PacketType::FifthItemAction => Some(PacketLength::Fixed(6)),
            PacketType::FourthPlayerAction => Some(PacketLength::Fixed(2)),
            PacketType::FirstItemAction => Some(PacketLength::Fixed(6)),
            PacketType::SecondPlayerAction => Some(PacketLength::Fixed(2)),
            PacketType::FirstNpcAction => Some(PacketLength::Fixed(2)),
            PacketType::Button => Some(PacketLength::Fixed(2)),
            PacketType::AddFriend => Some(PacketLength::Fixed(8)),
            PacketType::ItemOnObject => Some(PacketLength::Fixed(12)),
            PacketType::EnteredAmount => Some(PacketLength::Fixed(4)),
            PacketType::SwitchItem => Some(PacketLength::Fixed(7)),
            PacketType::RemoveFriend => Some(PacketLength::Fixed(8)),
            PacketType::ReportAbuse => Some(PacketLength::Fixed(10)),
            PacketType::TakeTileItem => Some(PacketLength::Fixed(6)),
            PacketType::MagicOnItem => Some(PacketLength::Fixed(8)),
            PacketType::MouseClicked => Some(PacketLength::Fixed(4)),
            PacketType::MagicOnPlayer => Some(PacketLength::Fixed(4)),
            PacketType::SecondObjectAction => Some(PacketLength::Fixed(6)),
            PacketType::PublicChat => Some(PacketLength::VariableByte),
            PacketType::FlaggedMouseEvent => Some(PacketLength::VariableByte),
            PacketType::Walk => Some(PacketLength::VariableByte),
            PacketType::Command => Some(PacketLength::VariableByte),
            PacketType::PrivateChat => Some(PacketLength::VariableByte),
            PacketType::ServerMessage => Some(PacketLength::VariableByte),
            PacketType::SetPlayerAction => Some(PacketLength::VariableByte),
            PacketType::ForwardPrivateChat => Some(PacketLength::VariableByte),
            PacketType::GroupedRegionUpdate => Some(PacketLength::VariableShort),
            PacketType::IgnoreList => Some(PacketLength::VariableShort),
            PacketType::NpcSynchronization => Some(PacketLength::VariableShort),
            PacketType::PlayerSynchronization => Some(PacketLength::VariableShort),
            PacketType::SetWidgetText => Some(PacketLength::VariableShort),
            PacketType::UpdateItems => Some(PacketLength::VariableShort),
            PacketType::UpdateSlottedItems => Some(PacketLength::VariableShort),
            _ => None,
        }
    }
}

pub struct PacketFactory {
    init_fn: fn() -> PacketEvent,
}

impl PacketFactory {
    pub fn new(init_fn: fn() -> PacketEvent) -> PacketFactory {
        Self { init_fn }
    }

    pub fn create(&self) -> PacketEvent {
        (self.init_fn)()
    }
}

pub trait Packet: Send + Sync {
    fn try_read(&mut self, _src: &mut BytesMut) -> anyhow::Result<()> {
        unimplemented!()
    }

    fn try_write(&self, _dst: &mut BytesMut) -> anyhow::Result<()> {
        unimplemented!()
    }

    fn get_type(&self) -> PacketType;
}
