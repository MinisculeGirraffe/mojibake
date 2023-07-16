#![warn(clippy::pedantic)]
mod lookups;
pub use lookups::{EMOJI_MAP, NUMBER_MAP, TAIL_MAP, TAIL_NUMBER_MAP};
use unicode_segmentation::UnicodeSegmentation;

pub fn encode(bytes: impl AsRef<[u8]>) -> String {
    let mut output = String::new();
    let mut stage = 0x0000u16;
    let mut remaining = 0;

    for byte in bytes.as_ref() {
        let byte = u16::from(*byte);
        let need = 11 - remaining;
        if need <= 8 {
            remaining = 8 - need;
            let index = (stage << need) | (byte >> remaining);
            output.push_str(
                EMOJI_MAP
                    .get(&index)
                    .expect("Somehow Unicode got rid of some emoji characters"),
            );

            stage = byte & ((1 << remaining) - 1);
        } else {
            stage = (stage << 8) | byte;
            remaining += 8;
        }
    }

    if remaining > 0 {
        if remaining <= 3 {
            output.push_str(
                TAIL_MAP
                    .get(&stage)
                    .expect("Somehow Unicode got rid of some emoji characters"),
            );
        } else {
            output.push_str(
                EMOJI_MAP
                    .get(&stage)
                    .expect("Somehow Unicode got rid of some emoji characters"),
            );
        }
    }

    output
}

pub fn decode(string: impl AsRef<str>) -> Option<Vec<u8>> {
    let mut ret = vec![];
    let mut remaining = 0u8;
    let mut stage = 0x00u32;
    let mut chars = string.as_ref().graphemes(true).peekable();
    let mut residue = 0;

    while let Some(c) = chars.next() {
        residue = (residue + 11) % 8;
        let (n_new_bits, new_bits) = match NUMBER_MAP.get(c) {
            Some(&bits) => {
                if chars.peek().is_none() {
                    (11 - residue, bits)
                } else {
                    (11, bits)
                }
            }
            None => match TAIL_NUMBER_MAP.get(c) {
                Some(index) => {
                    let need = 8 - remaining;
                    if *index < (1 << need) {
                        (need, *index as u16)
                    } else {
                        return None;
                    }
                }
                None => return None,
            },
        };

        remaining += n_new_bits;
        stage = (stage << n_new_bits) | u32::from(new_bits);
        while remaining >= 8 {
            remaining -= 8;
            ret.push((stage >> remaining) as u8);
            stage &= (1 << remaining) - 1;
        }
    }

    if remaining > 0 {
        ret.push((stage >> (8 - remaining)) as u8);
    }

    Some(ret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_decode_invalid_input() {
        let invalid_encoded_data = "Invalid data";
        let decoded = decode(invalid_encoded_data);
        assert_eq!(decoded, None);
    }

    #[test]
    fn test_decode_empty_input() {
        let empty_encoded_data = "";
        let decoded = decode(empty_encoded_data);

        assert_eq!(decoded, Some(vec![]));
    }

    proptest! {
        #[test]
        fn test_encode_decode(bytes in proptest::collection::vec(0u8..=255u8, 0..100)) {
            let encoded = encode(&bytes);
            let decoded = decode(&encoded);

            assert_eq!(decoded, Some(bytes));
        }
    }
}
