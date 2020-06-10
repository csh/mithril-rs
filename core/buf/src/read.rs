use std::ops::Shl;

use bytes::Buf;

use super::Transform;

/// A set of helper methods that extend the Buf object with functionality required to fully
/// decode packets sent by the client.
pub trait GameBuf: Buf {
    /// Attempts to read a String from self, this method will read until a line feed (`\n`) character.
    fn get_rs_string(&mut self) -> String {
        let mut result = String::default();
        loop {
            match self.get_u8() {
                10 => break,
                c => result.push(char::from(c)),
            }
        }
        result
    }

    /// Attempts to read a `u8` from self, if the the value of the byte > 127 then read another byte
    /// to compose a `u16`.
    fn get_smart(&mut self) -> u16 {
        let first = self.get_u8() as u16;
        if first > 127 {
            (first << 8 | self.get_u8() as u16) - 32768
        } else {
            first
        }
    }

    /// Reads a `u8` from the `Buf` whilst applying a transformation.
    fn get_u8t(&mut self, transform: Transform) -> u8 {
        match transform {
            Transform::Add => self.get_u8().wrapping_sub(128),
            Transform::Subtract => 128u8.wrapping_sub(self.get_u8()),
            Transform::Negate => (-self.get_i8()) as u8,
        }
    }

    /// Reads a big endian `u16` from the `Buf` whilst applying a transformation.
    fn get_u16t(&mut self, transform: Transform) -> u16 {
        let left = (self.get_u8() as u16).shl(8);
        let right = self.get_u8t(transform) as u16;
        left | right
    }

    /// Reads a little endian `u16` from the `Buf` whilst applying a transformation.
    fn get_u16t_le(&mut self, transform: Transform) -> u16 {
        let right = self.get_u8t(transform) as u16;
        let left = (self.get_u8() as u16).shl(8);
        left | right
    }

    /// Reads a `[u8]` in reverse from the `Buf` whilst applying a transformation.
    fn get_reverse(&mut self, dst: &mut [u8], transform: Transform) {
        let len = dst.len();
        for i in (0..len).rev() {
            dst[i] = self.get_u8t(transform);
        }
    }
}

impl<B: Buf> GameBuf for B {}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::*;

    #[test]
    pub fn test_get_u8t() {
        let mut buf = Bytes::from_static(&[17u8, 17u8, 17u8]);
        assert_eq!(buf.get_u8t(Transform::Subtract), 111);
        assert_eq!(buf.get_u8t(Transform::Add), 145);
        assert_eq!(buf.get_u8t(Transform::Negate), 239);
    }

    #[test]
    pub fn test_get_u16t() {
        let mut buf = Bytes::from_static(&[17u8, 20u8, 17u8, 20u8, 17u8, 20u8]);
        assert_eq!(buf.get_u16t(Transform::Subtract), 4460);
        assert_eq!(buf.get_u16t(Transform::Negate), 4588);
        assert_eq!(buf.get_u16t(Transform::Add), 4500);
    }

    #[test]
    pub fn test_get_u16t_le() {
        let mut buf = Bytes::from_static(&[17u8, 20u8, 17u8, 20u8, 17u8, 20u8]);
        assert_eq!(buf.get_u16t_le(Transform::Subtract), 5231);
        assert_eq!(buf.get_u16t_le(Transform::Negate), 5359);
        assert_eq!(buf.get_u16t_le(Transform::Add), 5265);
    }

    #[test]
    pub fn test_get_reverse() {
        let mut buf = Bytes::from_static(&[17u8, 20u8, 25u8]);
        let mut out = [0u8; 3];
        buf.get_reverse(&mut out, Transform::Add);
        assert_eq!(out[0], 153);
        assert_eq!(out[1], 148);
        assert_eq!(out[2], 145);
    }
}
