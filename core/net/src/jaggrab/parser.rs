use std::convert::TryFrom;

use nom::{
    bytes::complete::{tag, take_while1},
    character::{is_alphabetic, is_digit},
    sequence::{preceded, tuple},
    IResult,
};

use super::JaggrabError;
use super::JaggrabFile;

pub fn parse_request(src: &[u8]) -> Result<JaggrabFile, JaggrabError> {
    if src.len() < 11 {
        return Err(JaggrabError::InvalidRequest);
    }

    let (_, file_name) = read_jaggrab_line(src)?;
    let file_name = String::from_utf8(file_name)?;
    JaggrabFile::try_from(file_name)
}

fn read_jaggrab_line(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let protocol = tag("JAGGRAB /");
    let file_name = take_while1(is_alphabetic);
    let expected_crc = take_while1(is_digit);
    let new_lines = tag("\n\n");
    let file_name = preceded(protocol, file_name);

    let (remaining, (file_name, _expected_crc, _line_breaks)) =
        tuple((file_name, expected_crc, new_lines))(input)?;
    Ok((remaining, file_name.to_vec()))
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use super::*;

    #[test]
    pub fn test_parse_request() {
        let mut buf = BytesMut::from(&b"JAGGRAB /title0\n\n"[..]);
        parse_request(&mut buf).expect("valid request");
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
                read_jaggrab_line(input.as_ref()).is_err(),
                "invalid input; \"{}\"",
                input
            );
        }
    }
}
