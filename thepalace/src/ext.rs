use bytes::{Buf, BufMut};

/// Extensions to `bytes::Buf` for the Palace protocol
pub trait BufExt: Buf {
    /// Retrieves a C string from the buffer
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

    /// Retrieves a Pascal string from the buffer
    fn get_pstr(self) -> Vec<u8> {
        let len = self.get_u8() as usize;
        let data = self.take(len).into_inner().to_vec();

        data
    }

    /// Retrieves a 31-byte Pascal string from the buffer
    fn get_str31(self) -> Vec<u8> {
        let len = self.get_u8() as usize;
        let data = self.take(len).into_inner().to_vec();

        if len < 31 {
            let _ = input.take(31 - len);
        }

        data
    }

    /// Retrieves a 63-byte Pascal string from the buffer
    fn get_str63(self) -> Vec<u8> {
        let len = self.get_u8() as usize;
        let data = self.take(len).into_inner().to_vec();

        if len < 63 {
            let _ = self.take(63 - len);
        }

        data
    }
}

/// Extensions to `bytes::BufMut` for the Palace protocol
pub trait BufMutExt: BufMut {
    /// Writes a C string to the buffer
    fn put_cstr(&mut self, s: &[u8]) {
        self.put(&s[..]);
        self.put_u8(0);
    }

    /// Writes a Pascal string to the buffer
    fn put_pstr(&mut self, s: &[u8]) {
        self.put_u8(s.len() as u8);
        self.put(&s[..]);
    }

    /// Writes a 31-byte Pascal string to the buffer
    fn put_str31(&mut self, s: &[u8]) {
        self.put_pstr(s);

        if s.len() < 31 {
            self.put_bytes(0, 31 - s.len());
        }
    }

    /// Writes a 63-byte Pascal string to the buffer
    fn put_str63(&mut self, s: &[u8]) {
        self.put_pstr(s);

        if s.len() < 63 {
            let _ = self.put_bytes(0, 63 - s.len());
        }
    }
}
