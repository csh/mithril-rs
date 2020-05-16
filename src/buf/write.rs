use std::ops::Shr;

use bytes::BufMut;

use super::Transform;

/// A set of helper methods that extend the `BufMut` object with functionality required to fully
/// encode packets bound for the client.
pub trait GameBufMut: BufMut {
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