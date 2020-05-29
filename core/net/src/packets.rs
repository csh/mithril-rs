use ahash::AHashMap;
use once_cell::sync::Lazy;

pub use game::*;
pub use handshake::*;

use crate::packet::PacketFactory;
use crate::{PacketType, PacketLength};

mod game;
mod handshake;

mod prelude {
    pub use bytes::{Buf, BufMut, BytesMut};

    pub use mithril_buf::*;
    pub use crate::{Packet, PacketType};
}

/// Register a PacketFactory that uses Default to construct the packet.
macro_rules! default_factory {
    ($map:ident, $ty:ident) => {
        $map.insert(
            PacketType::$ty,
            PacketFactory::new(|| Box::new($ty::default())),
        )
    };
}

macro_rules! item_option_factory {
    ($map:ident, $opt:ident) => {
        $map.insert(
            PacketType::$opt,
            PacketFactory::new(|| {
                Box::new(game::ItemOption {
                    packet_type: PacketType::$opt,
                    interface_id: 0,
                    item_id: 0,
                    slot: 0,
                })
            }),
        );
    };
}

macro_rules! npc_option_factory {
    ($map:ident, $opt:ident) => {
        $map.insert(
            PacketType::$opt,
            PacketFactory::new(|| {
                Box::new(game::NpcAction {
                    packet_type: PacketType::$opt,
                    index: 0,
                })
            }),
        )
    };
}

macro_rules! player_action_factory {
    ($map:ident, $opt:ident) => {
        $map.insert(
            PacketType::$opt,
            PacketFactory::new(|| {
                Box::new(game::PlayerAction {
                    packet_type: PacketType::$opt,
                    index: 0,
                })
            }),
        )
    };
}

pub(crate) static PACKET_FACTORIES: Lazy<AHashMap<PacketType, PacketFactory>> = Lazy::new(|| {
    let mut factories = AHashMap::new();

    default_factory!(factories, HandshakeHello);
    default_factory!(factories, HandshakeAttemptConnect);
    default_factory!(factories, KeepAlive);
    default_factory!(factories, FocusUpdate);
    default_factory!(factories, PublicChat);
    default_factory!(factories, PrivateChat);
    default_factory!(factories, AddFriend);
    default_factory!(factories, AddIgnore);
    default_factory!(factories, RemoveFriend);
    default_factory!(factories, RemoveIgnore);
    default_factory!(factories, Button);
    default_factory!(factories, DialogueContinue);
    default_factory!(factories, ItemOnItem);
    default_factory!(factories, ItemOnObject);
    default_factory!(factories, ItemOnNpc);
    default_factory!(factories, PrivacyOption);
    default_factory!(factories, Command);
    default_factory!(factories, FlashingTabClicked);
    default_factory!(factories, ClosedInterface);
    default_factory!(factories, MagicOnNpc);
    default_factory!(factories, MagicOnItem);
    default_factory!(factories, MagicOnPlayer);
    default_factory!(factories, ArrowKey);
    default_factory!(factories, EnteredAmount);
    default_factory!(factories, ReportAbuse);
    default_factory!(factories, TakeTileItem);
    default_factory!(factories, MouseClicked);
    default_factory!(factories, PlayerDesign);

    item_option_factory!(factories, FirstItemOption);
    item_option_factory!(factories, SecondItemOption);
    item_option_factory!(factories, ThirdItemOption);
    item_option_factory!(factories, FourthItemOption);
    item_option_factory!(factories, FifthItemOption);

    npc_option_factory!(factories, FirstNpcAction);
    npc_option_factory!(factories, SecondNpcAction);
    npc_option_factory!(factories, ThirdNpcAction);
    npc_option_factory!(factories, FourthNpcAction);
    npc_option_factory!(factories, FifthNpcAction);

    player_action_factory!(factories, FirstPlayerAction);
    player_action_factory!(factories, SecondPlayerAction);
    player_action_factory!(factories, ThirdPlayerAction);
    player_action_factory!(factories, FourthPlayerAction);
    player_action_factory!(factories, FifthPlayerAction);

    factories.insert(
        PacketType::Walk,
        PacketFactory::new(|| Box::new(Walk {
            packet_type: PacketType::Walk,
            path: Vec::default(),
            running: false
        }))
    );
    factories.insert(
        PacketType::WalkWithAnticheat,
        PacketFactory::new(|| Box::new(Walk {
            packet_type: PacketType::WalkWithAnticheat,
            path: Vec::default(),
            running: false
        }))
    );

    factories.insert(
        PacketType::SpamPacket(PacketLength::VariableByte),
        PacketFactory::new(|| Box::new(SpamPacket(PacketLength::VariableByte)))
    );
    factories.insert(
        PacketType::SpamPacket(PacketLength::Fixed(0)),
        PacketFactory::new(|| Box::new(SpamPacket(PacketLength::Fixed(0))))
    );
    factories.insert(
        PacketType::SpamPacket(PacketLength::Fixed(1)),
        PacketFactory::new(|| Box::new(SpamPacket(PacketLength::Fixed(1))))
    );
    factories.insert(
        PacketType::SpamPacket(PacketLength::Fixed(4)),
        PacketFactory::new(|| Box::new(SpamPacket(PacketLength::Fixed(4))))
    );

    factories
});
