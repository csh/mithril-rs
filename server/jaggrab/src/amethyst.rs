use std::net::{TcpListener, TcpStream};
use std::collections::VecDeque;
use std::io;
use std::ops::DerefMut;

use amethyst::{
    core::SystemBundle,
    ecs::{System, World, DispatcherBuilder, ReadExpect, Write},
    Result
};
use mithril_core::fs::CacheFileSystem;

#[derive(Debug)]
pub struct JaggrabServerBundle {
    listener: Option<TcpListener>
}

impl JaggrabServerBundle {
    pub fn new(listener: Option<TcpListener>) -> Self {
        Self {
            listener
        }
    }
}

impl<'a, 'b> SystemBundle<'a, 'b> for JaggrabServerBundle {
    fn build(self, world: &mut World, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        let server_resource = JaggrabServerResource::new(self.listener);
        world.insert(server_resource);

        builder.add(JaggrabConnectionSystem::default(), "jaggrab_connect", &[]);
        builder.add(JaggrabSendingSystem::default(), "jaggrab", &["jaggrab_connect"]);

        log::info!("Listening for JAGGRAB requests");
        Ok(())
    }
}

#[derive(Default)]
struct JaggrabServerResource {
    listener: Option<TcpListener>,
    streams: VecDeque<TcpStream>
}

impl JaggrabServerResource {
    pub fn new(listener: Option<TcpListener>) -> Self {
        Self {
            listener,
            streams: VecDeque::with_capacity(64)
        }
    }
}

#[derive(Default, Debug)]
struct JaggrabConnectionSystem;

impl<'a> System<'a> for JaggrabConnectionSystem {
    type SystemData = Write<'a, JaggrabServerResource>;

    fn run(&mut self, mut net: Self::SystemData) {
        let net = net.deref_mut();
        if let Some(ref listener) = net.listener {
            loop {
                match listener.accept() {
                    Ok((stream, addr)) => {
                        log::info!("New JAGGRAB connection from {}", addr);
                        net.streams.push_back(stream);
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        break;
                    }
                    Err(cause) => {
                        log::error!("Error accepting JAGGRAB connection {}", cause);
                        break;
                    }
                }
            }
        }
    }    
}

#[derive(Default, Debug)]
struct JaggrabSendingSystem;

impl<'a> System<'a> for JaggrabSendingSystem {
    type SystemData = (
        Write<'a, JaggrabServerResource>,
        ReadExpect<'a, CacheFileSystem>
    );
    
    fn run(&mut self, (mut net, cache): Self::SystemData) {
        while let Some(stream) = net.streams.pop_front() {
            if let Err(cause) = crate::serve_request(stream, &cache) {
                log::error!("Processing JAGGRAB request failed; {}", cause);
            }
        }
    }
}