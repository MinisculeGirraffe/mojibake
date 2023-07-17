use std::io::{self, Read, Write};

use crate::{EMOJI_MAP, TAIL_MAP};

#[inline]
fn bytes_to_emojis<'a>(stage: &mut u16, remaining: &mut u8, byte: u8) -> Option<&'a str> {
    let byte = u16::from(byte);
    let need = 11 - *remaining;
    if need <= 8 {
        *remaining = 8 - need;
        let index = (*stage << need) | (byte >> *remaining);
        let emoji = EMOJI_MAP
            .get(&index)
            .expect("Somehow Unicode got rid of some emoji characters");

        *stage = byte & ((1 << *remaining) - 1);
        Some(emoji)
    } else {
        *stage = (*stage << 8) | byte;
        *remaining += 8;
        None
    }
}

#[inline]
fn handle_remaining_bits<'a>(stage: u16, remaining: u8) -> Option<&'a str> {
    if remaining == 0 {
        return None;
    }
    let emoji = {
        if remaining <= 3 {
            TAIL_MAP
                .get(&stage)
                .expect("Somehow Unicode got rid of some emoji characters")
        } else {
            EMOJI_MAP
                .get(&stage)
                .expect("Somehow Unicode got rid of some emoji characters")
        }
    };
    Some(emoji)
}

#[inline]
fn push_str(source: Option<&str>, dst: &mut String) {
    if let Some(str) = source {
        dst.push_str(str);
    }
}

#[inline]
fn write_str(source: Option<&str>, dst: &mut impl Write) -> io::Result<()> {
    if let Some(str) = source {
        dst.write_all(str.as_bytes())?;
    }
    Ok(())
}

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
    let bytes = bytes.as_ref();
    let mut output = String::new();
    let mut stage = 0x0000u16;
    let mut remaining = 0;

    for byte in bytes {
        push_str(
            bytes_to_emojis(&mut stage, &mut remaining, *byte),
            &mut output,
        );
    }
    push_str(handle_remaining_bits(stage, remaining), &mut output);

    output
}

/// Encodes a byte stream into a string representation using a defined emoji map.
///
/// This function is designed to convert bytes into a specific set of emoji
/// characters, providing a fun and unique way of representing data.
///
/// It accepts any type that implements `Read` (for reading bytes)
/// and `Write` (for writing the resulting emojis),
/// and returns a `Result<(), io::Error>`. Each input byte is mapped
/// to a specific emoji character, with the help of two maps: `EMOJI_MAP` and `TAIL_MAP`.
///
/// # Arguments
///
/// * `reader` - An object implementing `Read` to read bytes from.
/// * `writer` - An object implementing `Write` to write emojis to.
///
/// # Returns
///
/// An `io::Result<()>`. If it is `Err`, an error occurred while reading or writing.
///
/// # Panics
///
/// This function will panic if any emoji character needed for encoding is not
/// present in `EMOJI_MAP` or `TAIL_MAP`. This is not expected to ever happen.
///
/// # Example
/// ```rust
/// use mojibake::encode_stream;
/// use std::io::Cursor;
///
/// let input = vec![0x31, 0x32, 0x33];
/// let reader = Cursor::new(input);
/// let mut writer = Cursor::new(Vec::new());
///
/// encode_stream(reader, &mut writer).expect("encoding failed");
/// println!("{}", String::from_utf8(writer.into_inner()).unwrap());
/// ```
#[allow(clippy::module_name_repetitions)]
pub fn encode_stream<R: Read, W: Write>(reader: &mut R, mut writer: &mut W) -> io::Result<()> {
    let mut buffer = [0; 2]; // read two bytes at a time
    let mut stage = 0x0000u16;
    let mut remaining = 0;

    while let Ok(n) = reader.read(&mut buffer) {
        if n == 0 {
            break;
        }
        for byte in buffer.iter().take(n) {
            write_str(
                bytes_to_emojis(&mut stage, &mut remaining, *byte),
                &mut writer,
            )?;
        }
    }

    write_str(handle_remaining_bits(stage, remaining), &mut writer)?;

    Ok(())
}
