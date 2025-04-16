use bytes::{Buf, BufMut};

/// Message to be passed between client and server
#[derive(Debug, Default)]
pub struct Message {
    pub event: u32,
    pub relay: i32,
    pub data: Vec<u8>,
}

impl Message {
    pub fn from_bytes(input: &[u8]) -> Self {
        let event = input.get_u32_ne();
        let size = input.get_u32_ne() as usize;
        let relay = input.get_i32_ne();
        let mut data = vec![];

        Self {
            event: event,
            relay: relay,
            data: input.take(size).into_inner().to_vec(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![];

        buf.put_u32_ne(self.event);
        buf.put_u32_ne(self.data.len() as u32);
        buf.put_i32_ne(self.relay);
        buf.put(&self.data[..]);

        buf
    }
}
