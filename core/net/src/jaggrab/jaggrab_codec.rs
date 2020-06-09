use std::convert::TryFrom;

use bytes::{BufMut, Bytes, BytesMut};
use nom::{
    bytes::complete::{tag, take_while1},
    character::{is_alphabetic, is_digit},
    sequence::{preceded, tuple},
};
use tokio_util::codec::{Decoder, Encoder};

use super::JaggrabError;
use super::JaggrabFile;

pub struct JaggrabCodec;

impl Encoder<Bytes> for JaggrabCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.reserve(item.len());
        dst.put(item);
        Ok(())
    }
}

impl Decoder for JaggrabCodec {
    type Item = JaggrabFile;
    type Error = JaggrabError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 11 {
            return Ok(None);
        }

        let file_requested = match read_jaggrab_line(src) {
            Ok((remaining, file_bytes)) => String::from_utf8(file_bytes),
            Err(_parse_error) => return Err(JaggrabError::InvalidRequest),
        };

        let jaggrab_file = match file_requested {
            Ok(file_name) => JaggrabFile::try_from(file_name),
            Err(error) => return Err(JaggrabError::InvalidFileName(error)),
        };

        jaggrab_file.map(|f| Some(f))
    }
}

fn read_jaggrab_line(input: &[u8]) -> nom::IResult<&[u8], Vec<u8>> {
    let protocol = tag("JAGGRAB /");
    let file_name = take_while1(is_alphabetic);
    let expected_crc = take_while1(is_digit);
    let new_lines = tag("\n\n");
    let file_name = preceded(protocol, file_name);

    let (remaining, (file_name, expected_crc, _line_breaks)) =
        tuple((file_name, expected_crc, new_lines))(input)?;
    Ok((remaining, file_name.to_vec()))
}

#[cfg(test)]
mod tests {
    use bytes::{BufMut, BytesMut};
    use tokio_util::codec::Decoder;

    use super::{read_jaggrab_line, JaggrabCodec, JaggrabFile};

    #[test]
    pub fn test_valid_request() {
        let (input, file_name) =
            read_jaggrab_line(b"JAGGRAB /title0\n\n").expect("decode_jaggrab_line");
        assert!(input.len() == 0, "request should be read fully");
        match String::from_utf8(file_name) {
            Ok(file_name) => {
                assert_eq!(
                    String::from("title"),
                    file_name,
                    "expected 'title' to be read"
                );
            }
            _ => unreachable!("test case should pass"),
        }
    }

    #[test]
    pub fn test_invalid_request() {
        let invalid_inputs = [
            "JAGGRAB /\u{FEFF}0\n\n",
            "JAGGRAB /0\n\n",
            "JAGGRAB /\n\n",
            "title",
        ];

        for input in &invalid_inputs {
            assert!(
                dbg!(read_jaggrab_line(input.as_ref())).is_err(),
                "invalid input; \"{}\"",
                input
            );
        }
    }

    #[test]
    pub fn test_decoder() {
        let mut input = BytesMut::new();
        input.put(&b"JAGGRAB /title0\n\n"[..]);

        let mut decoder = JaggrabCodec;
        let file_opt = decoder.decode(&mut input).expect("input is valid");
        match file_opt.expect("input is valid") {
            JaggrabFile::Title => {}
            _ => panic!("test should not fail"),
        };
    }
}
