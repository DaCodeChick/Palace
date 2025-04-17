const LUT: &[u8] = include_bytes!("../encode.tbl");

#[derive(Debug)]
pub enum PalaceCryptError {
    Length(u8, usize), // threshold, got
}

/// Computes the CRC32 checksum of given input data and seed
pub fn crc32(input: &[u8], seed: u32) -> u32 {
    let mut crc = seed;

    input.iter().for_each(|b| {
        crc = crc.wrapping_shl(1) | (if crc & 0x80000000 == 0 { 0 } else { 1 }) ^ (*b as u32)
    });

    crc
}

/// Encrypts or decrypts the given input
pub fn crypt(input: &[u8], decrypting: bool) -> Result<Vec<u8>, PalaceCryptError> {
    if input.len() > 254 {
        return PalaceCryptError::Length(254, input.len());
    }

    let mut output = vec![0u8; input.len()];
    let mut rc = 0usize;
    let mut last = 0u8;

    input.iter().zip(output.iter_mut()).for_each(|(&i, o)| {
        *o = i ^ LUT[rc] ^ last;
        rc += 1;
        last = if decrypting { i } else { *o } ^ LUT[rc];
        rc += 1;
    });

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32() {
        let data = b"Hi there!";
        let crc = crc32(&data[..]);
        assert_eq!(crc, 0x42C57FF9);
    }

    #[test]
    fn test_crypt_roundtrip() {
        let data = b"Hi there!";
        let enc = crypt(&data[..], false).unwrap();
        let dec = crypt(&enc[..], true).unwrap();
        assert_eq!(&data[..], &dec[..]);
    }
}
