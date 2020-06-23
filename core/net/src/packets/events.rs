use super::*;
use crate::Packet;
use bytes::BytesMut;
use std::fmt::Debug;

#[derive(Debug)]
pub enum PacketEvent {
    Handshake(HandshakeEvent),
    Gameplay(GameplayEvent),
}

#[derive(Debug)]
pub enum HandshakeEvent {
    HandshakeHello(HandshakeHello),
    HandshakeExchangeKey(HandshakeExchangeKey),
    HandshakeAttemptConnect(HandshakeAttemptConnect),
    HandshakeConnectResponse(HandshakeConnectResponse),
}

#[derive(Debug)]
pub enum GameplayEvent {
    // region Gameplay - Serverbound
    KeepAlive(KeepAlive),
    FocusUpdate(FocusUpdate),
    PublicChat(PublicChat),
    ThirdItemOption(ItemOption),
    ThirdNpcAction(NpcAction),
    FifthNpcAction(NpcAction),
    FourthNpcAction(NpcAction),
    FifthPlayerAction(PlayerAction),
    DialogueContinue(DialogueContinue),
    SecondItemOption(ItemOption),
    ThirdItemAction(ItemAction),
    //    FlaggedMouseEvent(FlaggedMouseEvent),
    ItemOnItem(ItemOnItem),
    ItemOnNpc(ItemOnNpc),
    ThirdObjectAction(ObjectAction),
    SecondNpcAction(NpcAction),
    ThirdPlayerAction(PlayerAction),
    RemoveIgnore(RemoveIgnore),
    FourthItemOption(ItemOption),
    SpamPacket(SpamPacket),
    ArrowKey(ArrowKey),
    FifthItemOption(ItemOption),
    PrivacyOption(PrivacyOption),
    PlayerDesign(PlayerDesign),
    Command(Command),
    SecondItemAction(ItemAction),
    FlashingTabClicked(FlashingTabClicked),
    FirstItemOption(ItemOption),
    PrivateChat(PrivateChat),
    FirstPlayerAction(PlayerAction),
    FourthItemAction(ItemAction),
    ClosedInterface(ClosedInterface),
    MagicOnNpc(MagicOnNpc),
    FirstObjectAction(ObjectAction),
    AddIgnore(AddIgnore),
    FifthItemAction(ItemAction),
    FourthPlayerAction(PlayerAction),
    FirstItemAction(ItemAction),
    SecondPlayerAction(PlayerAction),
    FirstNpcAction(NpcAction),
    Button(Button),
    AddFriend(AddFriend),
    ItemOnObject(ItemOnObject),
    EnteredAmount(EnteredAmount),
    //    SwitchItem(SwitchItem),
    RemoveFriend(RemoveFriend),
    ReportAbuse(ReportAbuse),
    TakeTileItem(TakeTileItem),
    MagicOnItem(MagicOnItem),
    MouseClicked(MouseClicked),
    Walk(Walk),
    WalkWithAnticheat(Walk),
    MagicOnPlayer(MagicOnPlayer),
    SecondObjectAction(ObjectAction),
    // endregion

    // region Gameplay - Clientbound
    //    ForwardPrivateChat(ForwardPrivateChat),
    //    OpenOverlay(OpenOverlay),
    SetWidgetItemModel(SetWidgetItemModel),
    //    SendObject(SendObject),
    ServerMessage(ServerMessage),
    //    GroupedRegionUpdate(GroupedRegionUpdate),
    //    RemoveObject(RemoveObject),
    //    SetUpdatedRegion(SetUpdatedRegion),
    //    RemoveTileItem(RemoveTileItem),
    Logout(Logout),
    OpenInterface(OpenInterface),
    //    SendFriend(SendFriend),
    //    ConfigByte(ConfigByte),
    //    ConfigInt(ConfigInt),
    UpdateRunEnergy(UpdateRunEnergy),
    ClearRegion(ClearRegion),
    SetWidgetModel(SetWidgetModel),
    NpcSynchronization(NpcSynchronization),
    SetPlayerAction(SetPlayerAction),
    SetWidgetVisibility(SetWidgetVisibility),
    //    AddGlobalTileItem(AddGlobalTileItem),
    DisplayTabInterface(DisplayTabInterface),
    CloseInterface(CloseInterface),
    SetWidgetPlayerModel(SetWidgetPlayerModel),
    //    PositionHintIcon(PositionHintIcon),
    RegionChange(RegionChange),
    EnterAmount(EnterAmount),
    //    UpdateSlottedItems(UpdateSlottedItems),
    SetWidgetText(SetWidgetText),
    //    UpdateTileItem(UpdateTileItem),
    IdAssignment(IdAssignment),
    OpenDialogueInterface(OpenDialogueInterface),
    //    UpdateItems(UpdateItems),
    //    IgnoreList(IgnoreList),
    SetWidgetNpcModel(SetWidgetNpcModel),
    //    FriendServerStatus(FriendServerStatus),
    //    AddTileItem(AddTileItem),
    DisplayCrossbones(DisplayCrossbones),
    PlayerSynchronization(PlayerSynchronization),
    SetWidgetModelAnimation(SetWidgetModelAnimation),
    OpenInterfaceSidebar(OpenInterfaceSidebar),
    //    FlashTabInterface(FlashTabInterface),
    UpdateSkill(UpdateSkill),
    UpdateWeight(UpdateWeight),
    //    MobHintIcon(MobHintIcon),
    SwitchTabInterface(SwitchTabInterface),
    //    OpenDialogueOverlay(OpenDialogueOverlay),
    //    OpenSidebar(OpenSidebar),
    // endregion
}

impl From<GameplayEvent> for PacketEvent {
    fn from(event: GameplayEvent) -> Self {
        PacketEvent::Gameplay(event)
    }
}

impl From<HandshakeEvent> for PacketEvent {
    fn from(event: HandshakeEvent) -> Self {
        PacketEvent::Handshake(event)
    }
}

macro_rules! save_my_sanity {
    ($this:ident, $method:ident, $buf:ident) => {
        match $this {
            PacketEvent::Handshake(event) => match event {
                HandshakeEvent::HandshakeHello(packet) => packet.$method($buf),
                HandshakeEvent::HandshakeExchangeKey(packet) => packet.$method($buf),
                HandshakeEvent::HandshakeAttemptConnect(packet) => packet.$method($buf),
                HandshakeEvent::HandshakeConnectResponse(packet) => packet.$method($buf),
            },
            PacketEvent::Gameplay(event) => match event {
                GameplayEvent::KeepAlive(packet) => packet.$method($buf),
                GameplayEvent::FocusUpdate(packet) => packet.$method($buf),
                GameplayEvent::PublicChat(packet) => packet.$method($buf),
                GameplayEvent::ThirdItemOption(packet) => packet.$method($buf),
                GameplayEvent::ThirdNpcAction(packet) => packet.$method($buf),
                GameplayEvent::FifthNpcAction(packet) => packet.$method($buf),
                GameplayEvent::FourthNpcAction(packet) => packet.$method($buf),
                GameplayEvent::FifthPlayerAction(packet) => packet.$method($buf),
                GameplayEvent::DialogueContinue(packet) => packet.$method($buf),
                GameplayEvent::SecondItemOption(packet) => packet.$method($buf),
                GameplayEvent::ThirdItemAction(packet) => packet.$method($buf),
                GameplayEvent::ItemOnItem(packet) => packet.$method($buf),
                GameplayEvent::ItemOnNpc(packet) => packet.$method($buf),
                GameplayEvent::ThirdObjectAction(packet) => packet.$method($buf),
                GameplayEvent::SecondNpcAction(packet) => packet.$method($buf),
                GameplayEvent::ThirdPlayerAction(packet) => packet.$method($buf),
                GameplayEvent::RemoveIgnore(packet) => packet.$method($buf),
                GameplayEvent::FourthItemOption(packet) => packet.$method($buf),
                GameplayEvent::SpamPacket(packet) => packet.$method($buf),
                GameplayEvent::ArrowKey(packet) => packet.$method($buf),
                GameplayEvent::FifthItemOption(packet) => packet.$method($buf),
                GameplayEvent::PrivacyOption(packet) => packet.$method($buf),
                GameplayEvent::PlayerDesign(packet) => packet.$method($buf),
                GameplayEvent::Command(packet) => packet.$method($buf),
                GameplayEvent::SecondItemAction(packet) => packet.$method($buf),
                GameplayEvent::FlashingTabClicked(packet) => packet.$method($buf),
                GameplayEvent::FirstItemOption(packet) => packet.$method($buf),
                GameplayEvent::PrivateChat(packet) => packet.$method($buf),
                GameplayEvent::FirstPlayerAction(packet) => packet.$method($buf),
                GameplayEvent::FourthItemAction(packet) => packet.$method($buf),
                GameplayEvent::ClosedInterface(packet) => packet.$method($buf),
                GameplayEvent::MagicOnNpc(packet) => packet.$method($buf),
                GameplayEvent::FirstObjectAction(packet) => packet.$method($buf),
                GameplayEvent::AddIgnore(packet) => packet.$method($buf),
                GameplayEvent::FifthItemAction(packet) => packet.$method($buf),
                GameplayEvent::FourthPlayerAction(packet) => packet.$method($buf),
                GameplayEvent::FirstItemAction(packet) => packet.$method($buf),
                GameplayEvent::SecondPlayerAction(packet) => packet.$method($buf),
                GameplayEvent::FirstNpcAction(packet) => packet.$method($buf),
                GameplayEvent::Button(packet) => packet.$method($buf),
                GameplayEvent::AddFriend(packet) => packet.$method($buf),
                GameplayEvent::ItemOnObject(packet) => packet.$method($buf),
                GameplayEvent::EnteredAmount(packet) => packet.$method($buf),
                GameplayEvent::RemoveFriend(packet) => packet.$method($buf),
                GameplayEvent::ReportAbuse(packet) => packet.$method($buf),
                GameplayEvent::TakeTileItem(packet) => packet.$method($buf),
                GameplayEvent::MagicOnItem(packet) => packet.$method($buf),
                GameplayEvent::MouseClicked(packet) => packet.$method($buf),
                GameplayEvent::Walk(packet) => packet.$method($buf),
                GameplayEvent::WalkWithAnticheat(packet) => packet.$method($buf),
                GameplayEvent::MagicOnPlayer(packet) => packet.$method($buf),
                GameplayEvent::SecondObjectAction(packet) => packet.$method($buf),
                GameplayEvent::SetWidgetItemModel(packet) => packet.$method($buf),
                GameplayEvent::ServerMessage(packet) => packet.$method($buf),
                GameplayEvent::Logout(packet) => packet.$method($buf),
                GameplayEvent::OpenInterface(packet) => packet.$method($buf),
                GameplayEvent::UpdateRunEnergy(packet) => packet.$method($buf),
                GameplayEvent::ClearRegion(packet) => packet.$method($buf),
                GameplayEvent::SetWidgetModel(packet) => packet.$method($buf),
                GameplayEvent::NpcSynchronization(packet) => packet.$method($buf),
                GameplayEvent::SetPlayerAction(packet) => packet.$method($buf),
                GameplayEvent::SetWidgetVisibility(packet) => packet.$method($buf),
                GameplayEvent::DisplayTabInterface(packet) => packet.$method($buf),
                GameplayEvent::CloseInterface(packet) => packet.$method($buf),
                GameplayEvent::SetWidgetPlayerModel(packet) => packet.$method($buf),
                GameplayEvent::RegionChange(packet) => packet.$method($buf),
                GameplayEvent::EnterAmount(packet) => packet.$method($buf),
                GameplayEvent::SetWidgetText(packet) => packet.$method($buf),
                GameplayEvent::IdAssignment(packet) => packet.$method($buf),
                GameplayEvent::OpenDialogueInterface(packet) => packet.$method($buf),
                GameplayEvent::SetWidgetNpcModel(packet) => packet.$method($buf),
                GameplayEvent::DisplayCrossbones(packet) => packet.$method($buf),
                GameplayEvent::PlayerSynchronization(packet) => packet.$method($buf),
                GameplayEvent::SetWidgetModelAnimation(packet) => packet.$method($buf),
                GameplayEvent::OpenInterfaceSidebar(packet) => packet.$method($buf),
                GameplayEvent::UpdateSkill(packet) => packet.$method($buf),
                GameplayEvent::UpdateWeight(packet) => packet.$method($buf),
                GameplayEvent::SwitchTabInterface(packet) => packet.$method($buf),
            },
        }
    };
}

impl PacketEvent {
    pub fn is_handshake(&self) -> bool {
        match self {
            PacketEvent::Handshake(_) => true,
            PacketEvent::Gameplay(_) => false,
        }
    }

    pub fn is_gameplay(&self) -> bool {
        match self {
            PacketEvent::Handshake(_) => false,
            PacketEvent::Gameplay(_) => true,
        }
    }
}

// TODO: Code generation pls.
// TODO: Return an error in the appropriate method if the packet cannot be read/write.
impl Packet for PacketEvent {
    fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
        save_my_sanity!(self, try_read, src)
    }

    fn try_write(&self, dst: &mut BytesMut) -> anyhow::Result<()> {
        save_my_sanity!(self, try_write, dst)
    }

    fn get_type(&self) -> PacketType {
        match self {
            PacketEvent::Handshake(event) => match event {
                HandshakeEvent::HandshakeHello(packet) => packet.get_type(),
                HandshakeEvent::HandshakeExchangeKey(packet) => packet.get_type(),
                HandshakeEvent::HandshakeAttemptConnect(packet) => packet.get_type(),
                HandshakeEvent::HandshakeConnectResponse(packet) => packet.get_type(),
            },
            PacketEvent::Gameplay(event) => match event {
                GameplayEvent::KeepAlive(packet) => packet.get_type(),
                GameplayEvent::FocusUpdate(packet) => packet.get_type(),
                GameplayEvent::PublicChat(packet) => packet.get_type(),
                GameplayEvent::ThirdItemOption(packet) => packet.get_type(),
                GameplayEvent::ThirdNpcAction(packet) => packet.get_type(),
                GameplayEvent::FifthNpcAction(packet) => packet.get_type(),
                GameplayEvent::FourthNpcAction(packet) => packet.get_type(),
                GameplayEvent::FifthPlayerAction(packet) => packet.get_type(),
                GameplayEvent::DialogueContinue(packet) => packet.get_type(),
                GameplayEvent::SecondItemOption(packet) => packet.get_type(),
                GameplayEvent::ThirdItemAction(packet) => packet.get_type(),
                GameplayEvent::ItemOnItem(packet) => packet.get_type(),
                GameplayEvent::ItemOnNpc(packet) => packet.get_type(),
                GameplayEvent::ThirdObjectAction(packet) => packet.get_type(),
                GameplayEvent::SecondNpcAction(packet) => packet.get_type(),
                GameplayEvent::ThirdPlayerAction(packet) => packet.get_type(),
                GameplayEvent::RemoveIgnore(packet) => packet.get_type(),
                GameplayEvent::FourthItemOption(packet) => packet.get_type(),
                GameplayEvent::SpamPacket(packet) => packet.get_type(),
                GameplayEvent::ArrowKey(packet) => packet.get_type(),
                GameplayEvent::FifthItemOption(packet) => packet.get_type(),
                GameplayEvent::PrivacyOption(packet) => packet.get_type(),
                GameplayEvent::PlayerDesign(packet) => packet.get_type(),
                GameplayEvent::Command(packet) => packet.get_type(),
                GameplayEvent::SecondItemAction(packet) => packet.get_type(),
                GameplayEvent::FlashingTabClicked(packet) => packet.get_type(),
                GameplayEvent::FirstItemOption(packet) => packet.get_type(),
                GameplayEvent::PrivateChat(packet) => packet.get_type(),
                GameplayEvent::FirstPlayerAction(packet) => packet.get_type(),
                GameplayEvent::FourthItemAction(packet) => packet.get_type(),
                GameplayEvent::ClosedInterface(packet) => packet.get_type(),
                GameplayEvent::MagicOnNpc(packet) => packet.get_type(),
                GameplayEvent::FirstObjectAction(packet) => packet.get_type(),
                GameplayEvent::AddIgnore(packet) => packet.get_type(),
                GameplayEvent::FifthItemAction(packet) => packet.get_type(),
                GameplayEvent::FourthPlayerAction(packet) => packet.get_type(),
                GameplayEvent::FirstItemAction(packet) => packet.get_type(),
                GameplayEvent::SecondPlayerAction(packet) => packet.get_type(),
                GameplayEvent::FirstNpcAction(packet) => packet.get_type(),
                GameplayEvent::Button(packet) => packet.get_type(),
                GameplayEvent::AddFriend(packet) => packet.get_type(),
                GameplayEvent::ItemOnObject(packet) => packet.get_type(),
                GameplayEvent::EnteredAmount(packet) => packet.get_type(),
                GameplayEvent::RemoveFriend(packet) => packet.get_type(),
                GameplayEvent::ReportAbuse(packet) => packet.get_type(),
                GameplayEvent::TakeTileItem(packet) => packet.get_type(),
                GameplayEvent::MagicOnItem(packet) => packet.get_type(),
                GameplayEvent::MouseClicked(packet) => packet.get_type(),
                GameplayEvent::Walk(packet) => packet.get_type(),
                GameplayEvent::WalkWithAnticheat(packet) => packet.get_type(),
                GameplayEvent::MagicOnPlayer(packet) => packet.get_type(),
                GameplayEvent::SecondObjectAction(packet) => packet.get_type(),
                GameplayEvent::SetWidgetItemModel(packet) => packet.get_type(),
                GameplayEvent::ServerMessage(packet) => packet.get_type(),
                GameplayEvent::Logout(packet) => packet.get_type(),
                GameplayEvent::OpenInterface(packet) => packet.get_type(),
                GameplayEvent::UpdateRunEnergy(packet) => packet.get_type(),
                GameplayEvent::ClearRegion(packet) => packet.get_type(),
                GameplayEvent::SetWidgetModel(packet) => packet.get_type(),
                GameplayEvent::NpcSynchronization(packet) => packet.get_type(),
                GameplayEvent::SetPlayerAction(packet) => packet.get_type(),
                GameplayEvent::SetWidgetVisibility(packet) => packet.get_type(),
                GameplayEvent::DisplayTabInterface(packet) => packet.get_type(),
                GameplayEvent::CloseInterface(packet) => packet.get_type(),
                GameplayEvent::SetWidgetPlayerModel(packet) => packet.get_type(),
                GameplayEvent::RegionChange(packet) => packet.get_type(),
                GameplayEvent::EnterAmount(packet) => packet.get_type(),
                GameplayEvent::SetWidgetText(packet) => packet.get_type(),
                GameplayEvent::IdAssignment(packet) => packet.get_type(),
                GameplayEvent::OpenDialogueInterface(packet) => packet.get_type(),
                GameplayEvent::SetWidgetNpcModel(packet) => packet.get_type(),
                GameplayEvent::DisplayCrossbones(packet) => packet.get_type(),
                GameplayEvent::PlayerSynchronization(packet) => packet.get_type(),
                GameplayEvent::SetWidgetModelAnimation(packet) => packet.get_type(),
                GameplayEvent::OpenInterfaceSidebar(packet) => packet.get_type(),
                GameplayEvent::UpdateSkill(packet) => packet.get_type(),
                GameplayEvent::UpdateWeight(packet) => packet.get_type(),
                GameplayEvent::SwitchTabInterface(packet) => packet.get_type(),
            },
        }
    }
}
