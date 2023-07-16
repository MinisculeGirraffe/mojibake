#![warn(clippy::pedantic)]
mod lookups;
pub use lookups::{EMOJI_MAP, NUMBER_MAP, TAIL_MAP, TAIL_NUMBER_MAP};
use unicode_segmentation::UnicodeSegmentation;

/// Encodes a byte array into a string representation using a defined emoji map.
///
/// This function is designed to convert bytes into a specific set of emoji
/// characters, providing a fun and unique way of representing data.
///
/// It accepts any type that implements `AsRef<[u8]>` (like `&[u8]` and `Vec<u8>`)
/// and returns the encoded data as a `String`. Each input byte is mapped
/// to a specific emoji character, with the help of two maps: `EMOJI_MAP` and `TAIL_MAP`.
///
/// # Arguments
///
/// * `bytes` - A byte array to be encoded into emojis.
///
/// # Returns
///
/// A `String` containing the emoji representation of the input byte array.
///
/// # Panics
///
/// This function will panic if any emoji character needed for encoding is not
/// present in `EMOJI_MAP` or `TAIL_MAP`. This is not expected to ever happen.
///
/// # Example
/// ```rust
/// use mojibake::encode;
///
/// let bytes = vec![0x31, 0x32, 0x33];
/// let encoded = encode(bytes);
/// println!("{}", encoded);  // Prints the emoji representation of the byte array.
/// ```
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
        println!("Stage: {stage}");
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

/// Decodes a string of emojis back into a byte array.
///
/// This function converts a string of emojis, that were encoded with the `encode` function,
/// back into the original byte array. The conversion uses two predefined maps: `NUMBER_MAP` and `TAIL_NUMBER_MAP`.
///
/// # Arguments
///
/// * `string` - A string reference containing the emojis to be decoded. This argument implements `AsRef<str>`,
/// meaning it can be anything that can be viewed as a string slice, like `&str` or `String`.
///
/// # Returns
///
/// An `Option<Vec<u8>>` which is `Some` with the decoded byte array when the operation is successful, and `None`
/// when an error occurs during decoding (e.g. if the string contains invalid characters or an unexpected sequence).
///
/// # Example
///
/// ```rust
/// use mojibake::decode;
///
/// let encoded = "💑🏼❣️🧝🏻😎🇹🇰🟰❇️🥷🇨🇾🙆🏼🙁😎🐽🥉❇️🥦2️⃣🏇🏿🙁🚞👷🏽👲🏻⚙️‼️🧑🏾";  // Emoji string obtained from the `encode` function.
/// let decoded = decode(encoded);
/// println!("{:?}", decoded)
/// ```
pub fn decode(string: impl AsRef<str>) -> Option<Vec<u8>> {
    let mut ret = vec![];
    let mut remaining = 0u8;
    let mut stage = 0x00u32;
    let mut chars = string.as_ref().graphemes(false).peekable();
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
                        (need, *index)
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

    #[test]
    fn test_lookup_map_codegen() {
        assert_eq!(EMOJI_MAP.len(), 2048);
        assert_eq!(EMOJI_MAP.len(), NUMBER_MAP.len());
        assert_eq!(TAIL_MAP.len(), TAIL_NUMBER_MAP.len());
        assert!(TAIL_MAP.len() > 8);

        for key in 0..EMOJI_MAP.len() {
            let key = key as u16;
            let value = EMOJI_MAP.get(&key);
            assert!(value.is_some());
            let value = *value.unwrap();
            let matching_num = NUMBER_MAP.get(value);
            assert!(matching_num.is_some());
            assert_eq!(key, *matching_num.unwrap())
        }
    }
}
