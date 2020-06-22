use ahash::AHashMap;
use once_cell::sync::Lazy;

pub use game::*;
pub use handshake::*;

use crate::packet::PacketFactory;
use crate::{PacketLength, PacketType};

mod game;
mod handshake;
mod events;

pub use events::*;

mod prelude {
    pub use bytes::{Buf, BufMut, BytesMut};

    pub use crate::{Packet, PacketType};
    pub use mithril_buf::*;
}

/// Register a PacketFactory that uses Default to construct the packet.
macro_rules! default_factory {
    ($map:ident, $event:ident, $ty:ident) => {
        $map.insert(
            PacketType::$ty,
            PacketFactory::new(|| $event::$ty($ty::default()).into()),
        )
    };
}

macro_rules! item_option_factory {
    ($map:ident, $opt:ident, $idx:literal) => {
        $map.insert(
            PacketType::$opt,
            PacketFactory::new(|| GameplayEvent::$opt(ItemOption {
                option_index: $idx,
                ..Default::default()
            }).into()),
        )
    };
}

macro_rules! npc_action_factory {
    ($map:ident, $opt:ident, $idx:literal) => {
        action_factory!($map, $opt, NpcAction, $idx)
    };
}

macro_rules! player_action_factory {
    ($map:ident, $opt:ident, $idx:literal) => {
        action_factory!($map, $opt, PlayerAction, $idx)
    };
}

macro_rules! object_action_factory {
    ($map:ident, $opt:ident, $idx:literal) => {
        action_factory!($map, $opt, ObjectAction, $idx)
    };
}

macro_rules! item_action_factory {
    ($map:ident, $opt:ident, $idx:literal) => {
        action_factory!($map, $opt, ItemAction, $idx)
    };
}

macro_rules! action_factory {
    ($map:ident, $opt:ident, $impl:ident, $idx:literal) => {
        $map.insert(
            PacketType::$opt,
            PacketFactory::new(|| GameplayEvent::$opt($impl {
                action_index: $idx,
                ..Default::default()
            }).into()),
        )
    }
}

pub(crate) static PACKET_FACTORIES: Lazy<AHashMap<PacketType, PacketFactory>> = Lazy::new(|| {
    let mut factories = AHashMap::new();

    default_factory!(factories, HandshakeEvent, HandshakeHello);
    default_factory!(factories, HandshakeEvent, HandshakeAttemptConnect);
    default_factory!(factories, GameplayEvent, KeepAlive);
    default_factory!(factories, GameplayEvent, FocusUpdate);
    default_factory!(factories, GameplayEvent, PublicChat);
    default_factory!(factories, GameplayEvent, PrivateChat);
    default_factory!(factories, GameplayEvent, AddFriend);
    default_factory!(factories, GameplayEvent, AddIgnore);
    default_factory!(factories, GameplayEvent, RemoveFriend);
    default_factory!(factories, GameplayEvent, RemoveIgnore);
    default_factory!(factories, GameplayEvent, Button);
    default_factory!(factories, GameplayEvent, DialogueContinue);
    default_factory!(factories, GameplayEvent, ItemOnItem);
    default_factory!(factories, GameplayEvent, ItemOnObject);
    default_factory!(factories, GameplayEvent, ItemOnNpc);
    default_factory!(factories, GameplayEvent, PrivacyOption);
    default_factory!(factories, GameplayEvent, Command);
    default_factory!(factories, GameplayEvent, FlashingTabClicked);
    default_factory!(factories, GameplayEvent, ClosedInterface);
    default_factory!(factories, GameplayEvent, MagicOnNpc);
    default_factory!(factories, GameplayEvent, MagicOnItem);
    default_factory!(factories, GameplayEvent, MagicOnPlayer);
    default_factory!(factories, GameplayEvent, ArrowKey);
    default_factory!(factories, GameplayEvent, EnteredAmount);
    default_factory!(factories, GameplayEvent, ReportAbuse);
    default_factory!(factories, GameplayEvent, TakeTileItem);
    default_factory!(factories, GameplayEvent, MouseClicked);
    default_factory!(factories, GameplayEvent, PlayerDesign);

    item_option_factory!(factories, FirstItemOption, 0);
    item_option_factory!(factories, SecondItemOption, 1);
    item_option_factory!(factories, ThirdItemOption, 2);
    item_option_factory!(factories, FourthItemOption, 3);
    item_option_factory!(factories, FifthItemOption, 4);

    item_action_factory!(factories, FirstItemAction, 0);
    item_action_factory!(factories, SecondItemAction, 1);
    item_action_factory!(factories, ThirdItemAction, 2);
    item_action_factory!(factories, FourthItemAction, 3);
    item_action_factory!(factories, FifthItemAction, 4);

    npc_action_factory!(factories, FirstNpcAction, 0);
    npc_action_factory!(factories, SecondNpcAction, 1);
    npc_action_factory!(factories, ThirdNpcAction, 2);
    npc_action_factory!(factories, FourthNpcAction, 3);
    npc_action_factory!(factories, FifthNpcAction, 4);

    player_action_factory!(factories, FirstPlayerAction, 0);
    player_action_factory!(factories, SecondPlayerAction, 1);
    player_action_factory!(factories, ThirdPlayerAction, 2);
    player_action_factory!(factories, FourthPlayerAction, 3);
    player_action_factory!(factories, FifthPlayerAction, 4);

    object_action_factory!(factories, FirstObjectAction, 0);
    object_action_factory!(factories, SecondObjectAction, 1);
    object_action_factory!(factories, ThirdObjectAction, 2);

    factories.insert(
        PacketType::Walk,
        PacketFactory::new(|| GameplayEvent::Walk(Walk {
            packet_type: PacketType::Walk,
            path: Vec::default(),
            running: false,
        }).into())
    );
    factories.insert(
        PacketType::WalkWithAnticheat,
        PacketFactory::new(|| GameplayEvent::Walk(Walk {
            packet_type: PacketType::WalkWithAnticheat,
            path: Vec::default(),
            running: false,
        }).into())
    );

    factories.insert(
        PacketType::SpamPacket(PacketLength::VariableByte),
        PacketFactory::new(|| GameplayEvent::SpamPacket(SpamPacket(PacketLength::VariableByte)).into()),
    );
    factories.insert(
        PacketType::SpamPacket(PacketLength::Fixed(0)),
        PacketFactory::new(|| GameplayEvent::SpamPacket(SpamPacket(PacketLength::Fixed(0))).into()),
    );
    factories.insert(
        PacketType::SpamPacket(PacketLength::Fixed(1)),
        PacketFactory::new(|| GameplayEvent::SpamPacket(SpamPacket(PacketLength::Fixed(1))).into()),
    );
    factories.insert(
        PacketType::SpamPacket(PacketLength::Fixed(4)),
        PacketFactory::new(|| GameplayEvent::SpamPacket(SpamPacket(PacketLength::Fixed(4))).into()),
    );

    factories
});
