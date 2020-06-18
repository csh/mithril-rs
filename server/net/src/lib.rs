use amethyst::{
    core::{bundle::SystemBundle, SystemDesc, Named},
    ecs::{
        DispatcherBuilder, Entities, Entity, Read, ReadExpect, ReadStorage, System, SystemData, World, Write,
        WriteStorage, LazyUpdate
    },
    network::simulation::{NetworkSimulationEvent, TransportResource},
    shrev::{EventChannel, ReaderId},
    Result,
};

use mithril_core::net::{
    Packet, self,
    packets::{
        HandshakeAttemptConnect, HandshakeConnectResponse, HandshakeExchangeKey, HandshakeHello,
        LoginResponse,
    }
};
use mithril_server_types::{ConnectionIsaac, NetworkAddress, NewPlayer};
use std::collections::VecDeque;
use std::net::SocketAddr;
use ahash::AHashMap;
use bytes::{Buf, BufMut, BytesMut};
use mithril_server_types::auth::Authenticator;

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
                reader: world
                    .fetch_mut::<EventChannel<PacketEvent>>()
                    .register_reader(),
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
                    if players.entities.remove(addr).is_some() {
                        log::info!("Disconnected: {}", addr);
                    }
                }
                NetworkSimulationEvent::RecvError(e) => {
                    log::error!("Recv Error: {:?}", e);
                }
                _ => {}
            }
        }
    }
}

pub enum PacketEvent {
    Handshake(Entity, Box<dyn Packet>),
    Gameplay(Entity, Box<dyn Packet>),
}

impl PacketEvent {
    fn entity(&self) -> Entity {
        match self {
            PacketEvent::Handshake(entity, _) => *entity,
            PacketEvent::Gameplay(entity, _) => *entity,
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
        while let Some(event) = send_queue.events.pop_front() {
            let network_address = match address.get(event.entity()) {
                Some(address) => address,
                None => continue,
            };

            let mut encoded = bytes::BytesMut::new();
            let encode_result = match event {
                PacketEvent::Handshake(_, packet) => {
                    net::encode_packet(None, packet, &mut encoded)
                }
                PacketEvent::Gameplay(entity, packet) => {
                    if let Some(isaac) = rng.get_mut(entity) {
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
        Write<'a, EventChannel<PacketEvent>>,
        WriteStorage<'a, ConnectionIsaac>,
    );

    fn run(&mut self, (net_events, players, mut incoming, mut rng): Self::SystemData) {
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
                    let (emit_gameplay, packet) = match rng.get_mut(entity) {
                        Some(isaac) => (
                            true,
                            net::decode_packet(Some(&mut isaac.decoding), &mut payload),
                        ),
                        None => (false, net::decode_packet(None, &mut payload)),
                    };

                    match packet {
                        Ok(packet) => {
                            let packet_event = if emit_gameplay {
                                PacketEvent::Gameplay(entity, packet)
                            } else {
                                PacketEvent::Handshake(entity, packet)
                            };
                            incoming.single_write(packet_event);
                        }
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
    reader: ReaderId<PacketEvent>,
}

impl<'a> System<'a> for MithrilHandshakeSystem {
    type SystemData = (
        Read<'a, EventChannel<PacketEvent>>,
        ReadExpect<'a, Authenticator>,
        Write<'a, MithrilTransportResource>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (channel, auth, mut net, lazy): Self::SystemData) {
        for event in channel.read(&mut self.reader) {
            let (entity, packet) = match event {
                PacketEvent::Handshake(entity, packet) => (*entity, packet),
                _ => continue,
            };

            if packet.is::<HandshakeHello>() {
                log::info!("First handshake packet received");
                net.send_raw(entity, HandshakeExchangeKey::default());
            } else if let Ok(attempt) = packet.downcast_ref::<HandshakeAttemptConnect>() {
                let authenticated = match auth.authenticate(attempt.username.clone(), attempt.password.clone()) {
                    Ok(result) => result,
                    Err(cause) => {
                        log::error!("'{}' authentication failed; {}", attempt.username, cause);
                        net.send_raw(entity, HandshakeConnectResponse(LoginResponse::SessionBad));
                        continue;
                    }
                };

                if !authenticated {
                    net.send_raw(entity, HandshakeConnectResponse(LoginResponse::InvalidCredentials));
                    continue;
                }

                net.send_raw(entity, HandshakeConnectResponse(LoginResponse::Success));

                let decoding_seed = prepare_isaac_seed(attempt.client_isaac_key, attempt.server_isaac_key, 0);
                let encoding_seed = prepare_isaac_seed(attempt.client_isaac_key, attempt.server_isaac_key, 50);
                lazy.insert(entity, ConnectionIsaac::new(decoding_seed, encoding_seed));
                lazy.insert(entity, Named::new(attempt.username.clone()));
                lazy.insert(entity, NewPlayer);
            }
        }
    }
}

#[derive(Default)]
pub struct MithrilTransportResource {
    events: VecDeque<PacketEvent>,
}

impl MithrilTransportResource {
    pub fn send<P: Packet>(&mut self, entity: Entity, packet: P) {
        self.events
            .push_back(PacketEvent::Gameplay(entity, Box::new(packet)))
    }

    pub fn send_raw<P: Packet>(&mut self, entity: Entity, packet: P) {
        self.events
            .push_back(PacketEvent::Handshake(entity, Box::new(packet)))
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