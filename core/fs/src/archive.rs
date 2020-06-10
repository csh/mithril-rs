use std::collections::HashMap;

use bytes::{Buf, Bytes};

use crate::ArchiveError;

#[derive(Debug)]
pub struct Archive(HashMap<i32, ArchiveEntry>);

#[derive(Debug)]
struct ArchiveHeader {
    name_hash: i32,
    extracted_size: usize,
    size: usize,
}

#[derive(Debug)]
pub struct ArchiveEntry {
    header: ArchiveHeader,
    contents: Bytes,
}

impl ArchiveEntry {
    pub fn contents(&self) -> Bytes {
        self.contents.clone()
    }

    pub fn size(&self) -> usize {
        self.header.extracted_size
    }
}

#[allow(clippy::len_without_is_empty)]
impl Archive {
    pub(crate) fn decode(mut buf: Bytes) -> crate::Result<Self> {
        let decompressed_size = buf.get_uint(3) as usize;
        let size = buf.get_uint(3) as usize;
        let is_extracted = if size != decompressed_size {
            let decompressed = crate::compression::decompress_bzip2(buf)?;
            if decompressed.len() != decompressed_size {
                return Err(ArchiveError::LengthMismatch {
                    expected: decompressed_size,
                    actual: decompressed.len(),
                }
                .into());
            }
            buf = decompressed;
            true
        } else {
            false
        };

        let headers = decode_headers(&mut buf)?;
        let mut entries = HashMap::with_capacity(headers.len());
        for header in headers {
            let name_hash = header.name_hash;
            let contents = if is_extracted {
                let contents = buf.slice(..header.extracted_size);
                buf.advance(header.size);
                if contents.len() != header.size {
                    return Err(ArchiveError::LengthMismatch {
                        expected: header.size,
                        actual: contents.len(),
                    }
                    .into());
                }
                contents
            } else {
                let compressed = buf.slice(..header.size);
                buf.advance(header.size);
                let decompressed = crate::compression::decompress_bzip2(compressed)?;
                if decompressed.len() != header.extracted_size {
                    return Err(ArchiveError::LengthMismatch {
                        expected: header.extracted_size,
                        actual: decompressed.len(),
                    }
                    .into());
                }
                decompressed
            };
            entries.insert(name_hash, ArchiveEntry { header, contents });
        }
        Ok(Self(entries))
    }

    pub fn get_entry(&self, name: &str) -> Option<&ArchiveEntry> {
        self.0.get(&hash_name(name))
    }

    pub fn len(&self) -> usize {
        self.0.capacity()
    }
}

fn decode_headers<B: Buf>(buf: &mut B) -> crate::Result<Vec<ArchiveHeader>> {
    let mut headers = Vec::with_capacity(buf.get_u16() as usize);
    for _ in 0..headers.capacity() {
        headers.push(ArchiveHeader {
            name_hash: buf.get_i32(),
            extracted_size: buf.get_uint(3) as usize,
            size: buf.get_uint(3) as usize,
        });
    }
    Ok(headers)
}

fn hash_name(name: &str) -> i32 {
    name.to_uppercase()
        .chars()
        .map(|point| point as i32)
        .fold(0, |accumulated, next| {
            accumulated
                .wrapping_mul(61)
                .wrapping_add(next)
                .wrapping_sub(32)
        })
}
