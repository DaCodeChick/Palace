use bytes::{Buf, BufMut};

pub trait BufExt: Buf {
    /// Reads a null-terminated string from the buffer.
    /// The string is read until a null byte (0) is encountered.
    /// The null byte is not included in the returned string.
    /// The string is returned as a vector of bytes.
    /// The string is not guaranteed to be valid UTF-8.
    fn get_cstr(self) -> Vec<u8> {
        let mut s = vec![];

        loop {
            let c = self.get_u8();

            if c == 0 {
                break;
            } else {
                s.push(c);
            }
        }

        s
    }

    /// Reads a length-prefixed string from the buffer.
    /// The first byte of the string is the length of the string.
    /// The length is followed by the string data.
    /// The string is not null-terminated.
    /// The length of the string is limited to 255 bytes.
    /// If the string is longer than 255 bytes, it will be truncated.
    /// If the string is shorter than 255 bytes, the remaining bytes are filled with zeroes.
    /// The string is returned as a vector of bytes.
    /// The string is not guaranteed to be valid UTF-8.
    fn get_pstr(self) -> Vec<u8> {
        let len = self.get_u8() as usize;
        let data = self.take(len).into_inner().to_vec();

        data
    }

    /// Reads a length-prefixed string with a maximum length of 31 from the buffer.
    /// If the string is shorter than 31 bytes, the remaining bytes are filled with zeroes.
    /// If the string is longer than 31 bytes, it will be truncated.
    /// The string is returned as a vector of bytes.
    /// The string is not guaranteed to be valid UTF-8.
    fn get_str31(self) -> Vec<u8> {
        let len = self.get_u8() as usize;
        let data = self.take(len).into_inner().to_vec();

        if len < 31 {
            let _ = input.take(31 - len);
        }

        data
    }

    /// Reads a length-prefixed string with a maximum length of 63 from the buffer.
    /// If the string is shorter than 63 bytes, the remaining bytes are filled with zeroes.
    /// If the string is longer than 63 bytes, it will be truncated.
    /// The string is returned as a vector of bytes.
    /// The string is not guaranteed to be valid UTF-8.
    /// The string is not null-terminated.
    fn get_str63(self) -> Vec<u8> {
        let len = self.get_u8() as usize;
        let data = self.take(len).into_inner().to_vec();

        if len < 63 {
            let _ = self.take(63 - len);
        }

        data
    }
}

pub trait BufMutExt: BufMut {
    /// Writes a null-terminated string to the buffer.
    fn put_cstr(&mut self, s: &[u8]) {
        self.put(&s[..]);
        self.put_u8(0);
    }

    /// Writes a length-prefixed string to the buffer.
    /// The first byte of the string is the length of the string.
    /// The length is followed by the string data.
    /// The string is not null-terminated.
    /// The length of the string is limited to 255 bytes.
    /// If the string is longer than 255 bytes, it will be truncated.
    /// If the string is shorter than 255 bytes, the remaining bytes are filled with zeroes.
    fn put_pstr(&mut self, s: &[u8]) {
        self.put_u8(s.len() as u8);
        self.put(&s[..]);
    }

    /// Writes a length-prefixed string with a maximum length of 31 to the buffer.
    /// If the string is shorter than 31 bytes, the remaining bytes are filled with zeroes.
    /// If the string is longer than 31 bytes, it will be truncated.
    /// The string is not null-terminated.
    fn put_str31(&mut self, s: &[u8]) {
        self.put_pstr(s);

        if s.len() < 31 {
            self.put_bytes(0, 31 - s.len());
        }
    }

    /// Writes a length-prefixed string with a maximum length of 63 to the buffer.
    /// If the string is shorter than 63 bytes, the remaining bytes are filled with zeroes.
    /// If the string is longer than 63 bytes, it will be truncated.
    /// The string is not null-terminated.
    /// The string is not guaranteed to be valid UTF-8.
    fn put_str63(&mut self, s: &[u8]) {
        self.put_pstr(s);

        if s.len() < 63 {
            let _ = self.put_bytes(0, 63 - s.len());
        }
    }
}
