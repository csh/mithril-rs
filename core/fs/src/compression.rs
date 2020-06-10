use std::io::Read;

use bytes::{Bytes, BytesMut};
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;

pub(crate) fn decompress_bzip2(compressed: Bytes) -> crate::Result<Bytes> {
    debug_assert!(!compressed.is_empty(), "compressed buffer is empty");
    let compressed_with_header = {
        let mut packed = BytesMut::with_capacity(compressed.len() + 4);
        packed.extend_from_slice(&b"BZh1"[..]);
        packed.extend_from_slice(&compressed[..]);
        packed
    };
    let decoder = BzDecoder::new(&compressed_with_header[..]);
    decompress(decoder)
}

pub(crate) fn decompress_gzip(compressed: Bytes) -> crate::Result<Bytes> {
    debug_assert!(!compressed.is_empty(), "compressed buffer is empty");
    let decoder = GzDecoder::new(&compressed[..]);
    decompress(decoder)
}

fn decompress<R: Read>(mut decompressor: R) -> crate::Result<Bytes> {
    let mut decompressed = BytesMut::new();
    loop {
        let mut buf = [0u8; 1024];
        match decompressor.read(&mut buf)? {
            0 => return Ok(decompressed.freeze()),
            read => decompressed.extend_from_slice(&buf[..read]),
        }
    }
}
