use unicode_segmentation::UnicodeSegmentation;

use crate::{NUMBER_MAP, TAIL_NUMBER_MAP};
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
