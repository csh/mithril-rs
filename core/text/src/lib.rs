const FREQUENCY_ORDERED_CHARS: [char; 61] = [
    ' ', 'e', 't', 'a', 'o', 'i', 'h', 'n', 's', 'r', 'd', 'l', 'u', 'm', 'w', 'c', 'y', 'f', 'g',
    'p', 'b', 'v', 'k', 'x', 'j', 'q', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ' ',
    '!', '?', '.', ',', ':', ';', '(', ')', '-', '&', '*', '\\', '\'', '@', '#', '+', '=',
    '\u{0243}', '$', '%', '"', '[', ']',
];

const VALID_NAME_CHARS: [char; 37] = [
    '_', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
    's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];

pub fn encode_base37<T: AsRef<str>>(input: T) -> u64 {
    let mut combined = input.as_ref().chars().fold(0u64, |acc, c| {
        let acc = acc.wrapping_mul(37);
        match c {
            'A'..='Z' => acc + (c as u64 - 65) + 1,
            'a'..='z' => acc + (c as u64 - 97) + 1,
            '0'..='9' => acc + (c as u64 - 48) + 27,
            _ => acc,
        }
    });

    while combined % 37 == 0 && combined != 0 {
        combined = combined.wrapping_div(37);
    }
    combined
}

pub fn decode_base37(mut input: u64) -> anyhow::Result<String> {
    if input == 0 || input >= 0x5B5B57F8A98A5DD1 || input % 37 == 0 {
        return Err(anyhow::anyhow!("invalid name"));
    }

    let mut result = ['\0'; 12];
    let mut index = 0;
    while input != 0 {
        let local_input = input;
        input /= 37;
        index += 1;
        result[11 - index] = VALID_NAME_CHARS[(local_input - input * 37) as usize];
    }
    Ok(result.iter().filter(|c| **c != '\0').collect::<String>())
}

pub fn decompress(input: &[u8], len: usize) -> String {
    let mut out: Vec<char> = Vec::with_capacity(len as usize * 2);
    let mut position: usize = 0;
    let mut carry = -1i32;
    for index in 0..len * 2 {
        let table_pos = input[index / 2] >> 4 - 4 * (index % 2) as u8 & 0xF;
        if carry == -1 {
            if table_pos < 13 {
                out.insert(position, FREQUENCY_ORDERED_CHARS[table_pos as usize]);
                position += 1;
            } else {
                carry = table_pos as i32;
            }
        } else {
            out.insert(
                position,
                FREQUENCY_ORDERED_CHARS[((carry << 4) as usize + table_pos as usize - 195)],
            );
            position += 1;
            carry = -1;
        }
    }
    out.iter().collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_decompress() {
        const HELLO_WORLD: [u8; 8] = [0x61, 0xBB, 0x4E, 0xC0, 0xD1, 0x49, 0xBA, 0xE9];
        assert_eq!(decompress(&HELLO_WORLD, 8), "hello, world!");
    }

    #[test]
    pub fn test_encode_base37() {
        assert_eq!(encode_base37("smrkn"), 36292611);
        assert_eq!(encode_base37("csh"), 4818);
    }

    #[test]
    pub fn test_decode_base37() {
        assert_eq!(decode_base37(36292611).unwrap(), String::from("smrkn"));
        assert_eq!(decode_base37(4818).unwrap(), String::from("csh"));
    }
}
