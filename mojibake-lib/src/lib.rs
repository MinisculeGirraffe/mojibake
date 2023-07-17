#![warn(clippy::pedantic)]
mod decode;
mod encode;
mod lookups;
pub use decode::decode;
pub use encode::{encode, encode_stream};
pub use lookups::{EMOJI_MAP, NUMBER_MAP, TAIL_MAP, TAIL_NUMBER_MAP};

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
        fn test_encode_decode(bytes in proptest::collection::vec(0u8..=255u8, 0..1000)) {
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
