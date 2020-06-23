use amethyst::{
    core::{bundle::SystemBundle, Named, SystemDesc},
    ecs::{
        DispatcherBuilder, Entities, Entity, LazyUpdate, Read, ReadExpect, ReadStorage, System,
        SystemData, World, Write, WriteStorage,
    },
    network::simulation::{NetworkSimulationEvent, TransportResource},
    shrev::{EventChannel, ReaderId},
    Result,
};

use ahash::AHashMap;
use bytes::{Buf, BufMut, BytesMut};
use mithril_core::net::{
    self,
    packets::{HandshakeConnectResponse, HandshakeExchangeKey, LoginResponse},
};
use mithril_server_types::auth::Authenticator;
use mithril_server_types::{ConnectionIsaac, NetworkAddress, NewPlayer};
use std::collections::VecDeque;
use std::net::SocketAddr;

pub use mithril_core::net::packets::{GameplayEvent, HandshakeEvent, PacketEvent};
pub type EntityPacketEvent = (Entity, PacketEvent);
pub type PacketEventChannel = EventChannel<EntityPacketEvent>;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

#[derive(Debug)]
pub struct MithrilNetworkBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for MithrilNetworkBundle {
    fn build(self, world: &mut World, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        builder.add(
            MithrilEntityManagementSystemDesc::default().build(world),
            "entity_management_system",
            &["connection_listener"],
        );

        builder.add(
            MithrilDecodingSystemDesc::default().build(world),
            "decoding_system",
            &["entity_management_system"],
        );

        builder.add(
            MithrilEncodingSystemDesc::default().build(world),
            "encoding_system",
            &["entity_management_system"],
        );

        builder.add(
            MithrilHandshakeSystem {
                reader: world.fetch_mut::<PacketEventChannel>().register_reader(),
            },
            "handshake_system",
            &[],
        );

        Ok(())
    }
}

#[derive(Default, Debug)]
struct PlayerEntitiesResource {
    entities: AHashMap<SocketAddr, Entity>,
}

#[derive(Default, Debug)]
struct MithrilEntityManagementSystemDesc;

impl<'a, 'b> SystemDesc<'a, 'b, MithrilEntityManagementSystem>
    for MithrilEntityManagementSystemDesc
{
    fn build(self, world: &mut World) -> MithrilEntityManagementSystem {
        <MithrilEntityManagementSystem as System<'_>>::SystemData::setup(world);
        let reader = world
            .fetch_mut::<EventChannel<NetworkSimulationEvent>>()
            .register_reader();
        MithrilEntityManagementSystem::new(reader)
    }
}

struct MithrilEntityManagementSystem {
    reader: ReaderId<NetworkSimulationEvent>,
}

impl MithrilEntityManagementSystem {
    pub fn new(reader: ReaderId<NetworkSimulationEvent>) -> Self {
        Self { reader }
    }
}

impl<'a> System<'a> for MithrilEntityManagementSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, EventChannel<NetworkSimulationEvent>>,
        Write<'a, PlayerEntitiesResource>,
        WriteStorage<'a, NetworkAddress>,
    );

    fn run(&mut self, (entities, net_events, mut players, mut network_address): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("entity management");
        for event in net_events.read(&mut self.reader) {
            match event {
                NetworkSimulationEvent::Connect(addr) => {
                    log::info!("New connection from: {}", addr);
                    players.entities.entry(*addr).or_insert_with(|| {
                        entities
                            .build_entity()
                            .with(NetworkAddress(*addr), &mut network_address)
                            .build()
                    });
                }
                NetworkSimulationEvent::Disconnect(addr) => {
                    if let Some(entity) = players.entities.remove(addr) {
                        let _ = entities.delete(entity);
                        log::info!("Disconnected: {}", addr);
                    }
                }
                NetworkSimulationEvent::RecvError(e)
                    if e.kind() != std::io::ErrorKind::ConnectionAborted =>
                {
                    log::error!("Recv Error: {:?}", e);
                }
                _ => {}
            }
        }
    }
}

#[derive(Default, Debug)]
struct MithrilEncodingSystemDesc;

impl<'a, 'b> SystemDesc<'a, 'b, MithrilEncodingSystem> for MithrilEncodingSystemDesc {
    fn build(self, world: &mut World) -> MithrilEncodingSystem {
        <MithrilEncodingSystem as System<'_>>::SystemData::setup(world);
        MithrilEncodingSystem::default()
    }
}

#[derive(Default, Debug)]
struct MithrilEncodingSystem;

impl<'a> System<'a> for MithrilEncodingSystem {
    type SystemData = (
        Write<'a, TransportResource>,
        Write<'a, MithrilTransportResource>,
        ReadStorage<'a, NetworkAddress>,
        WriteStorage<'a, ConnectionIsaac>,
    );

    fn run(&mut self, (mut transport, mut send_queue, address, mut rng): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("packet encoding");
        while let Some((player, packet)) = send_queue.events.pop_front() {
            let network_address = match address.get(player) {
                Some(address) => address,
                None => continue,
            };

            let mut encoded = bytes::BytesMut::new();
            let encode_result = match packet {
                PacketEvent::Handshake(_) => net::encode_packet(None, packet, &mut encoded),
                PacketEvent::Gameplay(_) => {
                    if let Some(isaac) = rng.get_mut(player) {
                        net::encode_packet(Some(&mut isaac.encoding), packet, &mut encoded)
                    } else {
                        Err(anyhow::anyhow!(
                            "Attempted to send Gameplay packet before initialising ISAAC"
                        ))
                    }
                }
            };

            match encode_result {
                Ok(_) => transport.send(network_address.0, &encoded),
                Err(cause) => log::error!("Failed to encode packet; {}", cause),
            }
        }
    }
}

#[derive(Default, Debug)]
struct MithrilDecodingSystemDesc;

impl<'a, 'b> SystemDesc<'a, 'b, MithrilDecodingSystem> for MithrilDecodingSystemDesc {
    fn build(self, world: &mut World) -> MithrilDecodingSystem {
        <MithrilDecodingSystem as System<'_>>::SystemData::setup(world);
        let reader = world
            .fetch_mut::<EventChannel<NetworkSimulationEvent>>()
            .register_reader();
        MithrilDecodingSystem::new(reader)
    }
}

struct MithrilDecodingSystem {
    reader: ReaderId<NetworkSimulationEvent>,
}

impl MithrilDecodingSystem {
    pub fn new(reader: ReaderId<NetworkSimulationEvent>) -> Self {
        Self { reader }
    }
}

impl<'a> System<'a> for MithrilDecodingSystem {
    type SystemData = (
        Read<'a, EventChannel<NetworkSimulationEvent>>,
        Read<'a, PlayerEntitiesResource>,
        Write<'a, PacketEventChannel>,
        WriteStorage<'a, ConnectionIsaac>,
    );

    fn run(&mut self, (net_events, players, mut incoming, mut rng): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("packet decoding");
        for event in net_events.read(&mut self.reader) {
            if let NetworkSimulationEvent::Message(addr, payload) = event {
                let entity = match players.entities.get(addr) {
                    Some(entity) => *entity,
                    None => continue,
                };

                log::info!("{}: {:?}", addr, payload);

                let mut payload = {
                    let mut buf = bytes::BytesMut::with_capacity(payload.len());
                    buf.extend_from_slice(payload);
                    buf
                };

                /*
                 * Multiple packets may arrive in a single payload due to the timing of flushes.
                 * Tokio previously made this behaviour transparent, here we have a while loop to
                 * restore the effect.
                 *
                 * An example of packets that may arrive together are MouseClicked and PrivacyOption
                 */
                while payload.has_remaining() {
                    let packet = match rng.get_mut(entity) {
                        Some(isaac) => net::decode_packet(Some(&mut isaac.decoding), &mut payload),
                        None => net::decode_packet(None, &mut payload),
                    };

                    match packet {
                        Ok(packet) => incoming.single_write((entity, packet)),
                        Err(cause) => {
                            log::error!("Failed to decode packet; {}", cause);
                            break;
                        }
                    }
                }
            }
        }
    }
}

struct MithrilHandshakeSystem {
    reader: ReaderId<EntityPacketEvent>,
}

impl<'a> System<'a> for MithrilHandshakeSystem {
    type SystemData = (
        Read<'a, PacketEventChannel>,
        ReadExpect<'a, Authenticator>,
        Write<'a, MithrilTransportResource>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (channel, auth, mut net, lazy): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("handshake");
        for (player, event) in channel.read(&mut self.reader) {
            if !event.is_handshake() {
                continue;
            }

            let player = *player;

            if let PacketEvent::Handshake(HandshakeEvent::HandshakeHello(_)) = event {
                log::info!("First handshake packet received");
                net.send_raw(player, HandshakeExchangeKey::default());
            } else if let PacketEvent::Handshake(HandshakeEvent::HandshakeAttemptConnect(attempt)) =
                event
            {
                log::info!("Second handshake packet received");

                let authenticated = match auth
                    .authenticate(attempt.username.clone(), attempt.password.clone())
                {
                    Ok(result) => result,
                    Err(cause) => {
                        log::error!("'{}' authentication failed; {}", attempt.username, cause);
                        net.send_raw(player, HandshakeConnectResponse(LoginResponse::SessionBad));
                        continue;
                    }
                };

                if !authenticated {
                    net.send_raw(
                        player,
                        HandshakeConnectResponse(LoginResponse::InvalidCredentials),
                    );
                    continue;
                }

                net.send_raw(player, HandshakeConnectResponse(LoginResponse::Success));

                let decoding_seed =
                    prepare_isaac_seed(attempt.client_isaac_key, attempt.server_isaac_key, 0);
                let encoding_seed =
                    prepare_isaac_seed(attempt.client_isaac_key, attempt.server_isaac_key, 50);
                lazy.insert(player, ConnectionIsaac::new(decoding_seed, encoding_seed));
                lazy.insert(player, Named::new(attempt.username.clone()));
                lazy.insert(player, NewPlayer);
            }
        }
    }
}

#[derive(Default)]
pub struct MithrilTransportResource {
    events: VecDeque<EntityPacketEvent>,
}

impl MithrilTransportResource {
    pub fn send<I: Into<GameplayEvent>>(&mut self, player: Entity, packet: I) {
        let packet = packet.into();
        self.events.push_back((player, packet.into()));
    }

    pub fn send_raw<I: Into<HandshakeEvent>>(&mut self, player: Entity, packet: I) {
        let packet = packet.into();
        self.events.push_back((player, packet.into()));
    }
}

fn prepare_isaac_seed(client_key: u64, server_key: u64, increment: u32) -> [u8; 32] {
    let mut seed = BytesMut::with_capacity(32);
    seed.put_u32_le((client_key >> 32) as u32 + increment);
    seed.put_u32_le(client_key as u32 + increment);
    seed.put_u32_le((server_key >> 32) as u32 + increment);
    seed.put_u32_le(server_key as u32 + increment);
    seed.put(&[0u8; 16][..]);

    let mut actual_seed = [0u8; 32];
    actual_seed.copy_from_slice(&mut seed);
    actual_seed
}
