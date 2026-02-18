//! Buffer extension traits for reading and writing Palace Protocol data types.
//!
//! The Palace Protocol uses several custom string formats inherited from classic Mac OS:
//!
//! - **PString**: Pascal-style string with 1-byte length prefix (max 255 bytes)
//! - **CString**: C-style null-terminated string
//! - **Str31**: Pascal string with max length 31 (used for usernames, room names)
//! - **Str63**: Pascal string with max length 63 (used for various fields)
//!
//! All multi-byte integers in the Palace Protocol use **big-endian** byte order (network byte order),
//! as the protocol originated on classic Macintosh systems.

use bytes::{Buf, BufMut};
use std::io::{self, ErrorKind};

/// Extension trait for reading Palace Protocol data types from buffers.
pub trait BufExt: Buf {
    /// Read a Pascal-style string (PString) from the buffer.
    ///
    /// Format: 1 byte length prefix followed by that many bytes of string data.
    /// The length prefix does NOT include itself in the count.
    ///
    /// # Errors
    ///
    /// Returns `UnexpectedEof` if there aren't enough bytes in the buffer.
    fn get_pstring(&mut self) -> io::Result<String> {
        if !self.has_remaining() {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                "no length byte for PString",
            ));
        }

        let len = self.get_u8() as usize;

        if self.remaining() < len {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                format!(
                    "PString length {} but only {} bytes remain",
                    len,
                    self.remaining()
                ),
            ));
        }

        let mut bytes = vec![0u8; len];
        self.copy_to_slice(&mut bytes);

        // Palace Protocol uses MacRoman encoding - convert to UTF-8
        Ok(macroman_to_string(&bytes))
    }

    /// Read a Str31 (Pascal string with max length 31) from the buffer.
    ///
    /// Reads a fixed 32-byte field: 1 length byte + up to 31 chars + padding.
    /// Format is identical to PString but the length is guaranteed to be ≤ 31
    /// and the field is always 32 bytes.
    ///
    /// # Errors
    ///
    /// Returns `InvalidData` if the length prefix is > 31.
    /// Returns `UnexpectedEof` if there aren't enough bytes in the buffer.
    fn get_str31(&mut self) -> io::Result<String> {
        if self.remaining() < 32 {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                format!("Str31 needs 32 bytes, only {} remain", self.remaining()),
            ));
        }

        let len = self.get_u8();
        if len > 31 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Str31 length {} exceeds maximum of 31", len),
            ));
        }

        let len = len as usize;
        let mut bytes = vec![0u8; len];
        self.copy_to_slice(&mut bytes);

        // Skip padding to complete the 32-byte field
        let padding = 31 - len;
        self.advance(padding);

        Ok(macroman_to_string(&bytes))
    }

    /// Read a Str63 (Pascal string with max length 63) from the buffer.
    ///
    /// Reads a fixed 64-byte field: 1 length byte + up to 63 chars + padding.
    /// Format is identical to PString but the length is guaranteed to be ≤ 63
    /// and the field is always 64 bytes.
    ///
    /// # Errors
    ///
    /// Returns `InvalidData` if the length prefix is > 63.
    /// Returns `UnexpectedEof` if there aren't enough bytes in the buffer.
    fn get_str63(&mut self) -> io::Result<String> {
        if self.remaining() < 64 {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                format!("Str63 needs 64 bytes, only {} remain", self.remaining()),
            ));
        }

        let len = self.get_u8();
        if len > 63 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Str63 length {} exceeds maximum of 63", len),
            ));
        }

        let len = len as usize;
        let mut bytes = vec![0u8; len];
        self.copy_to_slice(&mut bytes);

        // Skip padding to complete the 64-byte field
        let padding = 63 - len;
        self.advance(padding);

        Ok(macroman_to_string(&bytes))
    }

    /// Read a C-style null-terminated string (CString) from the buffer.
    ///
    /// Reads bytes until a null byte (0x00) is encountered. The null terminator
    /// is consumed but not included in the returned string.
    ///
    /// # Errors
    ///
    /// Returns `UnexpectedEof` if no null terminator is found before the buffer ends.
    fn get_cstring(&mut self) -> io::Result<String> {
        let mut bytes = Vec::new();

        while self.has_remaining() {
            let byte = self.get_u8();
            if byte == 0 {
                // Found null terminator
                return Ok(macroman_to_string(&bytes));
            }
            bytes.push(byte);
        }

        Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "CString not null-terminated",
        ))
    }
}

/// Convert MacRoman encoded bytes to UTF-8 String.
///
/// MacRoman is the character encoding used by classic Mac OS (pre-OS X).
/// Codes 0-127 are identical to ASCII. Codes 128-255 map to various
/// special characters, accented letters, and symbols.
///
/// This function handles the upper 128 characters according to the MacRoman
/// character set specification.
fn macroman_to_string(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| macroman_to_char(b)).collect()
}

/// Convert a single MacRoman byte to a Unicode character.
///
/// Codes 0-127 are identical to ASCII.
/// Codes 128-255 use the MacRoman character mapping.
fn macroman_to_char(byte: u8) -> char {
    match byte {
        // 0-127: Standard ASCII
        0..=127 => byte as char,

        // 128-255: MacRoman specific mappings
        128 => 'Ä',
        129 => 'Å',
        130 => 'Ç',
        131 => 'É',
        132 => 'Ñ',
        133 => 'Ö',
        134 => 'Ü',
        135 => 'á',
        136 => 'à',
        137 => 'â',
        138 => 'ä',
        139 => 'ã',
        140 => 'å',
        141 => 'ç',
        142 => 'é',
        143 => 'è',
        144 => 'ê',
        145 => 'ë',
        146 => 'í',
        147 => 'ì',
        148 => 'î',
        149 => 'ï',
        150 => 'ñ',
        151 => 'ó',
        152 => 'ò',
        153 => 'ô',
        154 => 'ö',
        155 => 'õ',
        156 => 'ú',
        157 => 'ù',
        158 => 'û',
        159 => 'ü',
        160 => '†',
        161 => '°',
        162 => '¢',
        163 => '£',
        164 => '§',
        165 => '•',
        166 => '¶',
        167 => 'ß',
        168 => '®',
        169 => '©',
        170 => '™',
        171 => '´',
        172 => '¨',
        173 => '≠',
        174 => 'Æ',
        175 => 'Ø',
        176 => '∞',
        177 => '±',
        178 => '≤',
        179 => '≥',
        180 => '¥',
        181 => 'µ',
        182 => '∂',
        183 => '∑',
        184 => '∏',
        185 => 'π',
        186 => '∫',
        187 => 'ª',
        188 => 'º',
        189 => 'Ω',
        190 => 'æ',
        191 => 'ø',
        192 => '¿',
        193 => '¡',
        194 => '¬',
        195 => '√',
        196 => 'ƒ',
        197 => '≈',
        198 => '∆',
        199 => '«',
        200 => '»',
        201 => '…',
        202 => '\u{00A0}', // non-breaking space
        203 => 'À',
        204 => 'Ã',
        205 => 'Õ',
        206 => 'Œ',
        207 => 'œ',
        208 => '–',
        209 => '—',
        210 => '"',
        211 => '"',
        212 => '\'',
        213 => '\'',
        214 => '÷',
        215 => '◊',
        216 => 'ÿ',
        217 => 'Ÿ',
        218 => '⁄',
        219 => '€',
        220 => '‹',
        221 => '›',
        222 => 'ﬁ',
        223 => 'ﬂ',
        224 => '‡',
        225 => '·',
        226 => '‚',
        227 => '„',
        228 => '‰',
        229 => 'Â',
        230 => 'Ê',
        231 => 'Á',
        232 => 'Ë',
        233 => 'È',
        234 => 'Í',
        235 => 'Î',
        236 => 'Ï',
        237 => 'Ì',
        238 => 'Ó',
        239 => 'Ô',
        240 => '\u{F8FF}', // Apple logo (private use area)
        241 => 'Ò',
        242 => 'Ú',
        243 => 'Û',
        244 => 'Ù',
        245 => 'ı',
        246 => 'ˆ',
        247 => '˜',
        248 => '¯',
        249 => '˘',
        250 => '˙',
        251 => '˚',
        252 => '¸',
        253 => '˝',
        254 => '˛',
        255 => 'ˇ',
    }
}

/// Extension trait for writing Palace Protocol data types to buffers.
pub trait BufMutExt: BufMut {
    /// Write a Pascal-style string (PString) to the buffer.
    ///
    /// Format: 1 byte length prefix followed by the string bytes.
    ///
    /// Note: The input string should be UTF-8. For compatibility with classic
    /// Mac clients expecting MacRoman encoding, stick to ASCII subset or use
    /// characters that map identically in both encodings.
    ///
    /// # Errors
    ///
    /// Returns `InvalidInput` if the string is longer than 255 bytes (PString maximum).
    fn try_put_pstring(&mut self, s: &str) -> io::Result<()> {
        let bytes = s.as_bytes();
        if bytes.len() > 255 {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                format!("PString too long: {} bytes (max 255)", bytes.len()),
            ));
        }

        self.put_u8(bytes.len() as u8);
        self.put_slice(bytes);
        Ok(())
    }

    /// Write a Str31 (Pascal string with max length 31) to the buffer.
    ///
    /// Writes a fixed 32-byte field: 1 length byte + up to 31 chars + padding.
    ///
    /// Note: The input string should be UTF-8. For compatibility with classic
    /// Mac clients expecting MacRoman encoding, stick to ASCII subset.
    ///
    /// # Errors
    ///
    /// Returns `InvalidInput` if the string is longer than 31 bytes.
    fn try_put_str31(&mut self, s: &str) -> io::Result<()> {
        let bytes = s.as_bytes();
        if bytes.len() > 31 {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                format!("Str31 too long: {} bytes (max 31)", bytes.len()),
            ));
        }

        self.put_u8(bytes.len() as u8);
        self.put_slice(bytes);
        // Pad to 32 bytes total (1 len + 31 max chars)
        let padding = 31 - bytes.len();
        for _ in 0..padding {
            self.put_u8(0);
        }
        Ok(())
    }

    /// Write a Str63 (Pascal string with max length 63) to the buffer.
    ///
    /// Writes a fixed 64-byte field: 1 length byte + up to 63 chars + padding.
    ///
    /// # Errors
    ///
    /// Returns `InvalidInput` if the string is longer than 63 bytes.
    fn try_put_str63(&mut self, s: &str) -> io::Result<()> {
        let bytes = s.as_bytes();
        if bytes.len() > 63 {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                format!("Str63 too long: {} bytes (max 63)", bytes.len()),
            ));
        }

        self.put_u8(bytes.len() as u8);
        self.put_slice(bytes);
        // Pad to 64 bytes total (1 len + 63 max chars)
        let padding = 63 - bytes.len();
        for _ in 0..padding {
            self.put_u8(0);
        }
        Ok(())
    }

    /// Write a C-style null-terminated string (CString) to the buffer.
    ///
    /// Writes the string bytes followed by a null terminator (0x00).
    ///
    /// # Errors
    ///
    /// Returns `InvalidInput` if the string contains null bytes.
    fn try_put_cstring(&mut self, s: &str) -> io::Result<()> {
        let bytes = s.as_bytes();
        if bytes.contains(&0) {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "CString cannot contain null bytes",
            ));
        }

        self.put_slice(bytes);
        self.put_u8(0);
        Ok(())
    }

    /// Write a Pascal-style string (PString) to the buffer.
    ///
    /// Format: 1 byte length prefix followed by the string bytes.
    ///
    /// # Panics
    ///
    /// Panics if the string is longer than 255 bytes (PString maximum).
    /// Panics if there isn't enough space in the buffer.
    ///
    /// # Deprecated
    ///
    /// Use `try_put_pstring` instead for explicit error handling.
    fn put_pstring(&mut self, s: &str) {
        self.try_put_pstring(s)
            .expect("put_pstring failed - use try_put_pstring for error handling")
    }

    /// Write a Str31 (Pascal string with max length 31) to the buffer.
    ///
    /// Writes a fixed 32-byte field: 1 length byte + up to 31 chars + padding.
    ///
    /// # Panics
    ///
    /// Panics if the string is longer than 31 bytes.
    /// Panics if there isn't enough space in the buffer.
    ///
    /// # Deprecated
    ///
    /// Use `try_put_str31` instead for explicit error handling.
    fn put_str31(&mut self, s: &str) {
        self.try_put_str31(s)
            .expect("put_str31 failed - use try_put_str31 for error handling")
    }

    /// Write a Str63 (Pascal string with max length 63) to the buffer.
    ///
    /// Writes a fixed 64-byte field: 1 length byte + up to 63 chars + padding.
    ///
    /// # Panics
    ///
    /// Panics if the string is longer than 63 bytes.
    /// Panics if there isn't enough space in the buffer.
    ///
    /// # Deprecated
    ///
    /// Use `try_put_str63` instead for explicit error handling.
    fn put_str63(&mut self, s: &str) {
        self.try_put_str63(s)
            .expect("put_str63 failed - use try_put_str63 for error handling")
    }

    /// Write a C-style null-terminated string (CString) to the buffer.
    ///
    /// Writes the string bytes followed by a null terminator (0x00).
    ///
    /// # Panics
    ///
    /// Panics if the string contains null bytes.
    /// Panics if there isn't enough space in the buffer.
    ///
    /// # Deprecated
    ///
    /// Use `try_put_cstring` instead for explicit error handling.
    fn put_cstring(&mut self, s: &str) {
        self.try_put_cstring(s)
            .expect("put_cstring failed - use try_put_cstring for error handling")
    }
}

// Implement the traits for all types that implement Buf and BufMut
impl<T: Buf> BufExt for T {}
impl<T: BufMut> BufMutExt for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{Bytes, BytesMut};

    #[test]
    fn test_pstring_roundtrip() {
        let mut buf = BytesMut::new();
        buf.put_pstring("Hello");

        assert_eq!(buf.len(), 6); // 1 byte length + 5 bytes string
        assert_eq!(buf[0], 5); // Length prefix

        let mut reader = buf.freeze();
        let result = reader.get_pstring().unwrap();
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_pstring_empty() {
        let mut buf = BytesMut::new();
        buf.put_pstring("");

        assert_eq!(buf.len(), 1); // Just the length byte
        assert_eq!(buf[0], 0);

        let mut reader = buf.freeze();
        let result = reader.get_pstring().unwrap();
        assert_eq!(result, "");
    }

    #[test]
    #[should_panic(expected = "PString too long")]
    fn test_pstring_too_long() {
        let mut buf = BytesMut::new();
        let long_string = "a".repeat(256);
        buf.put_pstring(&long_string);
    }

    #[test]
    fn test_str31_roundtrip() {
        let mut buf = BytesMut::new();
        buf.put_str31("User");

        let mut reader = buf.freeze();
        let result = reader.get_str31().unwrap();
        assert_eq!(result, "User");
    }

    #[test]
    #[should_panic(expected = "Str31 too long")]
    fn test_str31_too_long() {
        let mut buf = BytesMut::new();
        let long_string = "a".repeat(32);
        buf.put_str31(&long_string);
    }

    #[test]
    fn test_str63_roundtrip() {
        let mut buf = BytesMut::new();
        buf.put_str63("This is a medium length string");

        let mut reader = buf.freeze();
        let result = reader.get_str63().unwrap();
        assert_eq!(result, "This is a medium length string");
    }

    #[test]
    #[should_panic(expected = "Str63 too long")]
    fn test_str63_too_long() {
        let mut buf = BytesMut::new();
        let long_string = "a".repeat(64);
        buf.put_str63(&long_string);
    }

    #[test]
    fn test_cstring_roundtrip() {
        let mut buf = BytesMut::new();
        buf.put_cstring("Hello World");

        assert_eq!(buf.len(), 12); // 11 bytes string + 1 null terminator
        assert_eq!(buf[11], 0); // Null terminator

        let mut reader = buf.freeze();
        let result = reader.get_cstring().unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_cstring_empty() {
        let mut buf = BytesMut::new();
        buf.put_cstring("");

        assert_eq!(buf.len(), 1); // Just the null terminator
        assert_eq!(buf[0], 0);

        let mut reader = buf.freeze();
        let result = reader.get_cstring().unwrap();
        assert_eq!(result, "");
    }

    #[test]
    #[should_panic(expected = "CString cannot contain null bytes")]
    fn test_cstring_with_null() {
        let mut buf = BytesMut::new();
        buf.put_cstring("Hello\0World");
    }

    #[test]
    fn test_get_pstring_insufficient_data() {
        let data = vec![5u8, b'H', b'i']; // Says 5 bytes but only has 2
        let mut buf = Bytes::from(data);
        let result = buf.get_pstring();
        assert!(result.is_err());
    }

    #[test]
    fn test_get_cstring_not_terminated() {
        let data = vec![b'H', b'e', b'l', b'l', b'o']; // No null terminator
        let mut buf = Bytes::from(data);
        let result = buf.get_cstring();
        assert!(result.is_err());
    }

    // Tests for new try_put_* methods with explicit error handling

    #[test]
    fn test_try_put_pstring_success() {
        let mut buf = BytesMut::new();
        let result = buf.try_put_pstring("Hello");
        assert!(result.is_ok());

        let mut reader = buf.freeze();
        let decoded = reader.get_pstring().unwrap();
        assert_eq!(decoded, "Hello");
    }

    #[test]
    fn test_try_put_pstring_too_long() {
        let mut buf = BytesMut::new();
        let long_string = "a".repeat(256);
        let result = buf.try_put_pstring(&long_string);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn test_try_put_str31_success() {
        let mut buf = BytesMut::new();
        let result = buf.try_put_str31("User");
        assert!(result.is_ok());

        let mut reader = buf.freeze();
        let decoded = reader.get_str31().unwrap();
        assert_eq!(decoded, "User");
    }

    #[test]
    fn test_try_put_str31_too_long() {
        let mut buf = BytesMut::new();
        let long_string = "a".repeat(32);
        let result = buf.try_put_str31(&long_string);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn test_try_put_str63_success() {
        let mut buf = BytesMut::new();
        let result = buf.try_put_str63("Medium string");
        assert!(result.is_ok());

        let mut reader = buf.freeze();
        let decoded = reader.get_str63().unwrap();
        assert_eq!(decoded, "Medium string");
    }

    #[test]
    fn test_try_put_str63_too_long() {
        let mut buf = BytesMut::new();
        let long_string = "a".repeat(64);
        let result = buf.try_put_str63(&long_string);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn test_try_put_cstring_success() {
        let mut buf = BytesMut::new();
        let result = buf.try_put_cstring("Hello World");
        assert!(result.is_ok());

        let mut reader = buf.freeze();
        let decoded = reader.get_cstring().unwrap();
        assert_eq!(decoded, "Hello World");
    }

    #[test]
    fn test_try_put_cstring_with_null() {
        let mut buf = BytesMut::new();
        let result = buf.try_put_cstring("Hello\0World");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidInput);
    }
}
