use crate::{NUMBER_MAP, TAIL_NUMBER_MAP};
use std::io::{self, Read, Write};
use unicode_segmentation::UnicodeSegmentation;

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
            let byte = u8::try_from(stage >> remaining).expect("Decoding byte was higher than 255");
            ret.push(byte);
            stage &= (1 << remaining) - 1;
        }
    }

    if remaining > 0 {
        let byte =
            u8::try_from(stage >> (8 - remaining)).expect("Decoding byte was higher than 255");
        ret.push(byte);
    }

    Some(ret)
}

struct GraphemeReader<'a, R: Read> {
    reader: &'a mut R,
    buffer: Vec<u8>,
}

impl<'a, R: Read> GraphemeReader<'a, R> {
    pub fn new(reader: &'a mut R) -> Self {
        Self {
            reader,
            buffer: Vec::new(),
        }
    }
    pub fn read_next_grapheme(&mut self) -> io::Result<Option<String>> {
        let mut chunk = [0; 4];

        loop {
            if let Some(grapheme) = self.get_grapheme() {
                let len = grapheme.len();
                self.buffer = self.buffer.split_off(len);
                return Ok(Some(grapheme));
            }
            let chunk_size = self.reader.read(&mut chunk)?;
            if chunk_size == 0 {
                break;
            }
            self.buffer.extend(&chunk[0..chunk_size]);
        }

        if self.buffer.is_empty() {
            return Ok(None);
        }

        let str = std::str::from_utf8(&self.buffer)
            .map(|i| Some(i.to_string()))
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid input data"));

        self.buffer.clear();
        str
    }

    fn get_grapheme(&self) -> Option<String> {
        let Ok(s) = std::str::from_utf8(&self.buffer) else {
            return None
        };
        let mut iter = s.graphemes(true).peekable();
        let grapheme = iter.next()?.to_string();
        iter.peek()?;
        Some(grapheme)
    }
}

impl<'a, R: Read> Iterator for GraphemeReader<'a, R> {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.read_next_grapheme().transpose()
    }
}

/// This function decodes a stream of data using a custom encoding scheme.
///
/// `decode_stream` takes a reader and writer object, and decodes the input read from the reader using the predefined
/// `NUMBER_MAP` and `TAIL_NUMBER_MAP`. The decoded output is then written to the writer. The function operates on the
/// data in chunks and maintains a decoding state internally.
///
/// If it encounters an invalid input during the decoding, it returns an `io::Error` of kind `InvalidData`.
/// # Example
/// ```rust
/// use std::io::Cursor;
/// use mojibake::{encode,decode_stream};
/// // assuming `encode` is a function that encodes your data
/// let original_data = "Hello, World!".as_bytes();
/// let encoded_data = encode(original_data);
///
/// let mut reader = Cursor::new(encoded_data);
/// let mut writer = Cursor::new(Vec::new());
///
/// // `decode_stream` may return an error if the encoded data is invalid
/// match decode_stream(&mut reader, &mut writer) {
///     Ok(()) => println!("Decoding succeeded"),
///     Err(e) => println!("Decoding failed: {}", e),
/// }
///
/// let decoded_data = writer.into_inner();
/// assert_eq!(original_data, decoded_data);
/// ```
#[allow(clippy::module_name_repetitions)]
pub fn decode_stream<R: Read, W: Write>(reader: &mut R, writer: &mut W) -> io::Result<()> {
    let mut remaining = 0u8;
    let mut stage = 0x00u32;
    let mut chars = GraphemeReader::new(reader).peekable();
    let mut residue = 0;

    while let Some(c) = chars.next() {
        let c = c?;
        residue = (residue + 11) % 8;
        let (n_new_bits, new_bits) = match NUMBER_MAP.get(&c) {
            Some(&bits) => {
                if chars.peek().is_none() {
                    (11 - residue, bits)
                } else {
                    (11, bits)
                }
            }
            None => match TAIL_NUMBER_MAP.get(&c) {
                Some(index) => {
                    let need = 8 - remaining;
                    if *index < (1 << need) {
                        (need, *index)
                    } else {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Invalid input data",
                        ));
                    }
                }
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid input data",
                    ))
                }
            },
        };
        remaining += n_new_bits;
        stage = (stage << n_new_bits) | u32::from(new_bits);
        while remaining >= 8 {
            remaining -= 8;
            let byte = u8::try_from(stage >> remaining).expect("LMAO this would be bad");
            writer.write_all(&[byte])?;
            stage &= (1 << remaining) - 1;
        }
    }

    if remaining > 0 {
        let byte =
            u8::try_from(stage >> (8 - remaining)).expect("Decoding byte was higher than 255");
        writer.write_all(&[byte])?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EMOJI_MAP;
    use std::io::Cursor;
    #[test]
    fn test_read_next_grapheme() {
        let data = EMOJI_MAP.values().copied().collect::<Vec<&str>>().join("");
        let mut cursor = Cursor::new(data.as_bytes());
        let reader = GraphemeReader::new(&mut cursor);

        for (left, right) in reader.zip(data.graphemes(true)) {
            assert_eq!(left.unwrap().as_str(), right);
        }
    }
}
