use std::io::Read;

use bytes::{BufMut, Bytes, BytesMut};
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;

pub(crate) fn decompress_bzip2(compressed: Bytes) -> anyhow::Result<Bytes> {
    anyhow::ensure!(!compressed.is_empty(), "compressed buffer is empty");
    let compressed_with_header = {
        let mut packed = BytesMut::with_capacity(compressed.len() + 4);
        packed.put(&b"BZh1"[..]);
        packed.put(&compressed[..]);
        packed
    };
    let decoder = BzDecoder::new(&compressed_with_header[..]);
    decompress(decoder)
}

pub(crate) fn decompress_gzip(compressed: Bytes) -> anyhow::Result<Bytes> {
    let decoder = GzDecoder::new(&compressed[..]);
    decompress(decoder)
}

fn decompress<R: Read>(mut decompressor: R) -> anyhow::Result<Bytes> {
    let mut decompressed = BytesMut::new();
    loop {
        let mut buf = [0u8; 1024];
        match decompressor.read(&mut buf)? {
            0 => return Ok(decompressed.freeze()),
            read => decompressed.put(&buf[..read]),
        }
    }
}
