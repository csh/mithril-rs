use std::io::prelude::*;
use std::net::TcpStream;

use mithril_core::{fs::CacheFileSystem, net::jaggrab::parse_request};

#[cfg(feature = "amethyst")]
mod amethyst;

#[cfg(feature = "amethyst")]
pub use crate::amethyst::*;

#[cfg(feature = "standalone")]
mod standalone;
#[cfg(feature = "standalone")]
pub use standalone::*;

pub(crate) fn serve_request(mut stream: TcpStream, cache: &CacheFileSystem) -> anyhow::Result<()> {
    let mut buf = [0; 32];
    let read = stream.read(&mut buf)?;
    let file = parse_request(&buf[..read])?;
    log::trace!("{} requested {:?}", stream.peer_addr()?, file);
    let data = cache.get_file(0, file as usize)?;
    stream.write_all(&data[..])?;
    Ok(())
}
