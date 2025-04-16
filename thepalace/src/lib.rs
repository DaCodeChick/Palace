const LUT: &[u8] = include_bytes!("../encode.tbl");

/// A two-dimensional point on screen
#[derive(Debug, Default)]
pub struct Point {
    pub v: i16,
    pub h: i16,
}

/// Computes the CRC32 checksum of given input data
pub fn crc32(input: &[u8]) -> u32 {
    let mut crc = 0xD9216290u32;

    input.iter().for_each(|b| {
        crc = crc.wrapping_shl(1) | (if crc & 0x80000000 == 0 { 0 } else { 1 }) ^ (*b as u32)
    });

    crc
}

/// Encrypts or decrypts the given input
pub fn crypt(input: &[u8], decrypting: bool) -> Vec<u8> {
    let mut output = vec![0u8; input.len()];
    let mut rc = 0usize;
    let mut last = 0u8;

    input.iter().zip(output.iter_mut()).for_each(|(&i, o)| {
        *o = i ^ LUT[rc] ^ last;
        rc += 1;
        last = if decrypting { i } else { *o } ^ LUT[rc];
        rc += 1;
    });

    output
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
        let enc = crypt(&data[..], false);
        let dec = crypt(&enc[..], true);
        assert_eq!(&data[..], &dec[..]);
    }
}
