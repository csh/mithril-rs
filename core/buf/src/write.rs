use std::ops::Shr;

use bytes::{BufMut, BytesMut};
use once_cell::sync::Lazy;

use super::Transform;

static BIT_MASKS: Lazy<[u32; 32]> = Lazy::new(|| {
    let mut masks = [0; 32];
    for i in 0..32 {
        masks[i] = (1 << i as u32) - 1;
    }
    masks
});

/// Helper struct for giving more fine-grained control over data being written to a buffer.
#[derive(Debug)]
pub struct BitWriter {
    inner: BytesMut,
    buffer: u32,
    index: u32,
}

impl BitWriter {
    fn new() -> Self {
        let mut inner = BytesMut::with_capacity(32);
        inner.put_slice(&[0u8; 32]);
        BitWriter {
            inner,
            buffer: 0,
            index: 0,
        }
    }

    /// Attempts to encode the specified number of bits and write them to the inner buffer.
    pub fn put_bits(&mut self, mut count: u32, value: u32) {
        assert!(count <= 32);
        let mut read_index = self.index as usize >> 3;
        let mut bit_offset = 8 - (self.index & 7);
        self.index += count;

        while count > bit_offset {
            self.buffer = self.inner[read_index] as u32;
            self.buffer &= BIT_MASKS[bit_offset as usize].wrapping_neg();
            self.buffer |= value >> count - bit_offset & BIT_MASKS[bit_offset as usize];
            self.inner[read_index] = self.buffer as u8;
            read_index += 1;
            count -= bit_offset;
            bit_offset = 8;
        }

        self.buffer = self.inner[read_index] as u32;
        if count == bit_offset {
            self.buffer &= BIT_MASKS[bit_offset as usize].wrapping_neg();
            self.buffer |= value & BIT_MASKS[bit_offset as usize];
        } else {
            self.buffer &= (BIT_MASKS[count as usize] << bit_offset - count).wrapping_neg();
            self.buffer |= (value & BIT_MASKS[count as usize]) << bit_offset - count;
        }
        self.inner[read_index] = self.buffer as u8;
    }
}

/// A set of helper methods that extend the `BufMut` object with functionality required to fully
/// encode packets bound for the client.
pub trait GameBufMut: BufMut {
    /// Accepts a closure with a [BitWrite](struct.BitWrite.html) argument that gives fine-grained control over
    /// the data written to self.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bytes::{BufMut, BytesMut};
    /// use mithril_net::buf::GameBufMut;
    ///
    /// let mut buf = BytesMut::new();
    /// buf.put_bits(|mut writer| {
    ///     writer.put_bits(8, 1234);
    ///     for _ in 0..5 {
    ///         writer.put_bits(1, 1);
    ///         writer.put_bits(2, 3);
    ///     }
    ///     writer
    /// });
    /// ```
    fn put_bits<B>(&mut self, write_fn: B)
        where B: FnOnce(BitWriter) -> BitWriter
    {
        let writer = write_fn(BitWriter::new());
        let written = (writer.index + 7) / 8;
        let mut buf = writer.inner;
        self.put_slice(&buf.split_to(written as usize));
    }

    /// Writes a `String` to self, terminating with the line feed (`\n`) character.
    fn put_rs_string(&mut self, value: String) {
        value.chars().for_each(|c| self.put_u8(c as u8));
        self.put_u8(10);
    }

    /// Writes a big endian `u8` to the `Buf` whilst applying a transformation.
    fn put_u8t(&mut self, value: u8, transform: Transform) {
        match transform {
            Transform::Add => self.put_u8(value.wrapping_add(128)),
            Transform::Subtract => self.put_u8(128u8.wrapping_sub(value)),
            Transform::Negate => self.put_u8((-(value as i8)) as u8),
        }
    }

    /// Writes a big endian `u16` to the `Buf` whilst applying a transformation.
    fn put_u16t(&mut self, value: u16, transform: Transform) {
        self.put_u8(value.shr(8) as u8);
        self.put_u8t(value as u8, transform);
    }

    /// Writes a little endian `u16` to the `Buf` whilst applying a transformation.
    fn put_u16t_le(&mut self, value: u16, transform: Transform) {
        self.put_u8t(value as u8, transform);
        self.put_u8(value.shr(8) as u8);
    }

    /// Writes a "middle" endian `u32` to the `Buf`.
    fn put_u32_me(&mut self, value: u32) {
        self.put_u8(value.shr(8) as u8);
        self.put_u8(value as u8);
        self.put_u8(value.shr(24) as u8);
        self.put_u8(value.shr(16) as u8);
    }

    /// Writes an "inverse middle" endian `u32` to the `Buf`.
    fn put_u32_inv_me(&mut self, value: u32) {
        self.put_u8(value.shr(16) as u8);
        self.put_u8(value.shr(24) as u8);
        self.put_u8(value as u8);
        self.put_u8(value.shr(8) as u8);
    }
}

impl<B: BufMut> GameBufMut for B {}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use super::*;

    #[test]
    pub fn test_put_bits() {
        let mut buf = BytesMut::new();
        buf.put_bits(|mut writer| {
            writer.put_bits(8, 1234);
            for _ in 0..5 {
                writer.put_bits(1, 1);
                writer.put_bits(2, 3);
            }
            writer
        });
        assert_eq!(&buf, &[0xD2, 0xFF, 0xFE][..]);
    }

    #[test]
    pub fn test_put_u8t() {
        let mut buf = BytesMut::with_capacity(3);
        buf.put_u8t(10, Transform::Add);
        buf.put_u8t(10, Transform::Negate);
        buf.put_u8t(10, Transform::Subtract);
        assert_eq!(138, buf[0]);
        assert_eq!(246, buf[1]);
        assert_eq!(118, buf[2]);
    }

    #[test]
    pub fn test_put_u16t() {
        let mut buf = BytesMut::with_capacity(6);
        buf.put_u16t(10, Transform::Add);
        buf.put_u16t(10, Transform::Negate);
        buf.put_u16t(10, Transform::Subtract);
        assert_eq!(buf[0], 0);
        assert_eq!(buf[1], 138);
        assert_eq!(buf[2], 0);
        assert_eq!(buf[3], 246);
        assert_eq!(buf[4], 0);
        assert_eq!(buf[5], 118);
    }

    #[test]
    pub fn test_put_u16t_le() {
        let mut buf = BytesMut::with_capacity(6);
        buf.put_u16t_le(10, Transform::Add);
        assert_eq!(buf[0], 138);
        assert_eq!(buf[1], 0);
    }

    #[test]
    pub fn test_put_u32_me() {
        let mut buf = BytesMut::with_capacity(6);
        buf.put_u32_me(12345678);
        assert_eq!(buf[0], 97);
        assert_eq!(buf[1], 78);
        assert_eq!(buf[2], 0);
        assert_eq!(buf[3], 188);
    }

    #[test]
    pub fn test_put_u32_inv_me() {
        let mut buf = BytesMut::with_capacity(6);
        buf.put_u32_inv_me(12345678);
        assert_eq!(buf[0], 188);
        assert_eq!(buf[1], 0);
        assert_eq!(buf[2], 78);
        assert_eq!(buf[3], 97);
    }
}
