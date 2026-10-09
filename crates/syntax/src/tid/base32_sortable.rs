use super::errors::DecodeError;

cfg_select! {
    // TODO: for now this part is not working, I'll like to have a proper constant-time
    // implementation
    // feature = "constant-time" => {
    //     use std::ops::RangeInclusive;
    //
    //     const DECODE_RANGES: [(RangeInclusive<u8>, i16); 2] = [(b'2'..=b'7', -49), (b'a'..=b'z', -102)];
    //
    //     fn decode(encoded: &str) -> Result<u64, DecodeError> {
    //         let encoded: &[u8] = encoded.as_ref();
    //         if encoded.is_empty() {
    //             return Err(());
    //         }
    //
    //         let mut decoded = [0u8; 8];
    //
    //         let mut encoded_chunks = encoded.chunks_exact(8);
    //         let mut decoded_chunks = decoded.chunks_exact_mut(5);
    //         let mut err = 0u8;
    //
    //         for (src, dst) in (&mut encoded_chunks).zip(&mut decoded_chunks) {
    //             let c0 = decode_5bits(src[0]);
    //             let c1 = decode_5bits(src[1]);
    //             let c2 = decode_5bits(src[2]);
    //             let c3 = decode_5bits(src[3]);
    //             let c4 = decode_5bits(src[4]);
    //             let c5 = decode_5bits(src[5]);
    //             let c6 = decode_5bits(src[6]);
    //             let c7 = decode_5bits(src[7]);
    //
    //             dst[0] = (((c0 << 3) | (c1 >> 2)) & 0xff) as u8;
    //             dst[1] = (((c1 << 6) | (c2 << 1) | (c3 >> 4)) & 0xff) as u8;
    //             dst[2] = (((c3 << 4) | (c4 >> 1)) & 0xff) as u8;
    //             dst[3] = (((c4 << 7) | (c5 << 2) | (c6 >> 3)) & 0xff) as u8;
    //             dst[4] = (((c6 << 5) | c7) & 0xff) as u8;
    //
    //             err |= ((c0 | c1 | c2 | c3 | c4 | c5 | c6 | c7) >> 8) as u8;
    //         }
    //
    //         let encoded_remainder = encoded_chunks.remainder();
    //         let decoded_remainder = decoded_chunks.into_remainder();
    //         let mut c = [0i16; 7];
    //
    //         if encoded_remainder.len() >= 1 {
    //             c[0] = decode_5bits(encoded_remainder[0]);
    //         }
    //         if encoded_remainder.len() >= 2 {
    //             c[1] = decode_5bits(encoded_remainder[1]);
    //         }
    //         if encoded_remainder.len() >= 3 {
    //             c[2] = decode_5bits(encoded_remainder[2]);
    //         }
    //         if encoded_remainder.len() >= 4 {
    //             c[3] = decode_5bits(encoded_remainder[3]);
    //         }
    //         if encoded_remainder.len() >= 5 {
    //             c[4] = decode_5bits(encoded_remainder[4]);
    //         }
    //         if encoded_remainder.len() >= 6 {
    //             c[5] = decode_5bits(encoded_remainder[5]);
    //         }
    //         if encoded_remainder.len() >= 7 {
    //             c[6] = decode_5bits(encoded_remainder[6]);
    //         }
    //
    //         err |= (encoded_remainder.len() == 1) as u8;
    //         err |= (encoded_remainder.len() == 3) as u8;
    //         err |= (encoded_remainder.len() == 6) as u8;
    //
    //         if encoded_remainder.len() >= 2 {
    //             decoded_remainder[0] = (((c[0] << 3) | (c[1] >> 2)) & 0xff) as u8;
    //         }
    //         if encoded_remainder.len() >= 4 {
    //             decoded_remainder[1] = (((c[1] << 6) | (c[2] << 1) | (c[3] >> 4)) & 0xff) as u8;
    //         }
    //         if encoded_remainder.len() >= 5 {
    //             decoded_remainder[2] = (((c[3] << 4) | (c[4] >> 1)) & 0xff) as u8;
    //         }
    //         if encoded_remainder.len() >= 7 {
    //             decoded_remainder[3] = (((c[4] << 7) | (c[5] << 2) | (c[6] >> 3)) & 0xff) as u8;
    //         }
    //
    //         err |= ((c[0] | c[1] | c[2] | c[3] | c[4] | c[5] | c[6]) >> 8) as u8;
    //
    //         if err == 0 {
    //             Ok(u64::from_le_bytes(decoded))
    //         } else {
    //             Err(())
    //         }
    //     }
    //
    //     fn decode_5bits(byte: u8) -> i16 {
    //         let byte = byte as i16;
    //         let mut decoded = -1;
    //
    //         for (range, offset) in &DECODE_RANGES {
    //             let start = *range.start() as i16 - 1;
    //             let end = *range.end() as i16 + 1;
    //
    //             decoded += (((start - byte) & (byte - end)) >> 8) & (byte + *offset);
    //         }
    //
    //         decoded
    //     }
    // }
    _ => {
        const ALPHABET: &[u8; 32] = b"234567abcdefghijklmnopqrstuvwxyz";

        pub(super) fn decode(encoded: impl AsRef<[u8]>) -> Result<u64, DecodeError> {
            let encoded = encoded.as_ref();
            if encoded.is_empty() {
                return Err(DecodeError::EmptyString);
            }

            let decoded = encoded
                .iter()
                .map(|byte| ALPHABET.iter().position(|symb| *byte == *symb).map(|chunk| chunk as u8))
                .collect::<Vec<_>>();

            if !decoded.contains(&None) {
                Ok(decoded
                    .iter()
                    .flatten()
                    .fold(0, |decoded, chunk| (decoded << 5) + *chunk as u64))
            } else {
                Err(
                    decoded.iter()
                        .enumerate()
                        .filter(|(_, chunk)| chunk.is_none())
                        .map(|(position, _)| DecodeError::InvalidSymbol { symbol: encoded[position].into(), position })
                        .next()
                        .expect("unable to retrieve decode error")
                )
            }
        }

        pub(super) fn encode(value: u64) -> String {
            let mut decoded = value;
            let mut encoded = (0..13).map(|_| {
                let chunk = decoded & 0x1f;
                decoded >>= 5;

                ALPHABET[chunk as usize]
            }).collect::<Vec<_>>();
            encoded.reverse();

            // SAFETY: all encoded chunk of 5 bits are encoded into ASCII characters
            unsafe { String::from_utf8_unchecked(encoded) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::assert_matches;

    #[test]
    fn it_encodes_u64_values() {
        assert_eq!(encode(0), "2222222222222".to_owned());
        assert_eq!(encode(0x5294a5294a5294a5), "7777777777777".to_owned());
        assert_eq!(encode(0x1fffffffffffffff), "3zzzzzzzzzzzz".to_owned());
        assert_eq!(encode(0x17fd6873eaf07c06), "3jzfcijpj2z2a".to_owned());
    }

    #[test]
    fn it_decodes_base32_sortable_values() {
        assert_matches!(decode("2222222222222"), Ok(0));
        assert_matches!(decode("7777777777777"), Ok(0x5294a5294a5294a5));
        assert_matches!(decode("3zzzzzzzzzzzz"), Ok(0x1fffffffffffffff));
        assert_matches!(decode("3jzfcijpj2z2a"), Ok(0x17fd6873eaf07c06));
    }
}
