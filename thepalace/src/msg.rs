use bytes::{Buf, BufMut};

use crate::{AssetDescriptor, AssetSpec};

#[derive(Debug, Clone)]
pub struct AssetSend {
    spec: AssetSpec,
    block_offset: u32,
    block_num: u16,
    num_blocks: u16,
    desc: AssetDescriptor,
    data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct BlowThruToClient {
    tag: u32,
    data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct BlowThruToServer {
    num_users: u32,
    tag: u32,
    ids: Vec<u32>,
    data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct DoorLock {
    room: u16,
    door: u16,
}

/// Represents a message sent to the client
#[derive(Debug, Clone)]
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
