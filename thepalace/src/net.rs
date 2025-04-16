use bitflags::bitflags;
use bytes::{Buf, BufMut};
use prop::PropFlags;

use crate::{BufExt, BufMutExt};

pub mod msg;
pub use msg::*;

/// Prop descriptor for file transfers
#[derive(Debug)]
pub struct AssetDescriptor {
    flags: PropFlags,
    size: u32,
    name: Vec<u8>,
}

impl AssetDescriptor {
    pub fn from_bytes(input: &[u8]) -> Self {
        let flags = PropFlags::from_bits_retain(input.get_u32_ne());
        let size = input.get_u32_ne();
        let name = input.get_str31();

        Self {
            flags: flags,
            size: size,
            name: name,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![];

        buf.put_u32_ne(self.flags.bits());
        buf.put_u32_ne(self.size);
        buf.put_str31(&self.name[..]);

        buf
    }
}

/// Prop specification for network
#[derive(Debug)]
pub struct AssetSpec {
    pub id: u32,
    pub crc: u32,
}

impl AssetSpec {
    pub fn from_bytes(input: &[u8]) -> Self {
        Self {
            id: input.get_u32_ne(),
            crc: input.get_u32_ne(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![];

        buf.put_u32_ne(self.id);
        buf.put_u32_ne(self.crc);

        buf
    }
}

/// A two-dimensional point on screen
#[derive(Debug, Default)]
pub struct Point {
    pub v: i16,
    pub h: i16,
}

impl Point {
    pub fn from_bytes(input: &[u8]) -> Self {
        Self {
            v: input.get_i16_ne(),
            h: input.get_i16_ne(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![];

        buf.put_i16_ne(self.v);
        buf.put_i16_ne(self.h);

        buf
    }
}
