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

        // Palace Protocol uses MacRoman encoding, but for now we'll treat as UTF-8/ASCII
        // TODO: Implement proper MacRoman to UTF-8 conversion
        String::from_utf8(bytes).map_err(|e| {
            io::Error::new(
                ErrorKind::InvalidData,
                format!("invalid UTF-8 in PString: {}", e),
            )
        })
    }

    /// Read a Str31 (Pascal string with max length 31) from the buffer.
    ///
    /// This is commonly used for usernames and room names in the Palace Protocol.
    /// Format is identical to PString but the length is guaranteed to be ≤ 31.
    ///
    /// # Errors
    ///
    /// Returns `InvalidData` if the length prefix is > 31.
    /// Returns `UnexpectedEof` if there aren't enough bytes in the buffer.
    fn get_str31(&mut self) -> io::Result<String> {
        if !self.has_remaining() {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                "no length byte for Str31",
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
        if self.remaining() < len {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                format!(
                    "Str31 length {} but only {} bytes remain",
                    len,
                    self.remaining()
                ),
            ));
        }

        let mut bytes = vec![0u8; len];
        self.copy_to_slice(&mut bytes);

        String::from_utf8(bytes).map_err(|e| {
            io::Error::new(
                ErrorKind::InvalidData,
                format!("invalid UTF-8 in Str31: {}", e),
            )
        })
    }

    /// Read a Str63 (Pascal string with max length 63) from the buffer.
    ///
    /// Format is identical to PString but the length is guaranteed to be ≤ 63.
    ///
    /// # Errors
    ///
    /// Returns `InvalidData` if the length prefix is > 63.
    /// Returns `UnexpectedEof` if there aren't enough bytes in the buffer.
    fn get_str63(&mut self) -> io::Result<String> {
        if !self.has_remaining() {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                "no length byte for Str63",
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
        if self.remaining() < len {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                format!(
                    "Str63 length {} but only {} bytes remain",
                    len,
                    self.remaining()
                ),
            ));
        }

        let mut bytes = vec![0u8; len];
        self.copy_to_slice(&mut bytes);

        String::from_utf8(bytes).map_err(|e| {
            io::Error::new(
                ErrorKind::InvalidData,
                format!("invalid UTF-8 in Str63: {}", e),
            )
        })
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
                return String::from_utf8(bytes).map_err(|e| {
                    io::Error::new(
                        ErrorKind::InvalidData,
                        format!("invalid UTF-8 in CString: {}", e),
                    )
                });
            }
            bytes.push(byte);
        }

        Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "CString not null-terminated",
        ))
    }
}

/// Extension trait for writing Palace Protocol data types to buffers.
pub trait BufMutExt: BufMut {
    /// Write a Pascal-style string (PString) to the buffer.
    ///
    /// Format: 1 byte length prefix followed by the string bytes.
    ///
    /// # Panics
    ///
    /// Panics if the string is longer than 255 bytes (PString maximum).
    /// Panics if there isn't enough space in the buffer.
    fn put_pstring(&mut self, s: &str) {
        let bytes = s.as_bytes();
        assert!(
            bytes.len() <= 255,
            "PString too long: {} bytes (max 255)",
            bytes.len()
        );

        self.put_u8(bytes.len() as u8);
        self.put_slice(bytes);
    }

    /// Write a Str31 (Pascal string with max length 31) to the buffer.
    ///
    /// # Panics
    ///
    /// Panics if the string is longer than 31 bytes.
    /// Panics if there isn't enough space in the buffer.
    fn put_str31(&mut self, s: &str) {
        let bytes = s.as_bytes();
        assert!(
            bytes.len() <= 31,
            "Str31 too long: {} bytes (max 31)",
            bytes.len()
        );

        self.put_u8(bytes.len() as u8);
        self.put_slice(bytes);
    }

    /// Write a Str63 (Pascal string with max length 63) to the buffer.
    ///
    /// # Panics
    ///
    /// Panics if the string is longer than 63 bytes.
    /// Panics if there isn't enough space in the buffer.
    fn put_str63(&mut self, s: &str) {
        let bytes = s.as_bytes();
        assert!(
            bytes.len() <= 63,
            "Str63 too long: {} bytes (max 63)",
            bytes.len()
        );

        self.put_u8(bytes.len() as u8);
        self.put_slice(bytes);
    }

    /// Write a C-style null-terminated string (CString) to the buffer.
    ///
    /// Writes the string bytes followed by a null terminator (0x00).
    ///
    /// # Panics
    ///
    /// Panics if the string contains null bytes.
    /// Panics if there isn't enough space in the buffer.
    fn put_cstring(&mut self, s: &str) {
        let bytes = s.as_bytes();
        assert!(!bytes.contains(&0), "CString cannot contain null bytes");

        self.put_slice(bytes);
        self.put_u8(0);
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
}
