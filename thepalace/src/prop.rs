use bitflags::bitflags;
use bytes::{Buf, BufMut};
use cfg_if::cfg_if;

use crate::{BufEx, BufMutEx, crc32};

const PROP: u32 = 0x50726F70;

bitflags! {
    /// Characterizes how a prop behaves. The server does not use these.
    pub struct PropFlags: u16 {
        const FORMAT_8BIT = 0;
        const HEAD = 2;
        const GHOST = 4;
        const RARE = 8;
        const ANIMATE = 16;
        const BOUNCE = 32;
        const FORMAT_20BIT = 64;
        const FORMAT_32BIT = 256;
        const FORMAT_S20BIT = 512;
        const FORMAT_MASK = FORMAT_20BIT | FORMAT_32BIT | FORMAT_S20BIT;
        const LEGACY = 0xFFC1;
    }
}

#[derive(Debug)]
pub struct AssetDescriptor {
    flags: PropFlags,
    size: u32,
    name: Vec<u8>,
}

impl AssetDescriptor {
    pub fn from_bytes(input: &[u8], swap: bool) -> Self {
        let desc: Self;

        if swap {
            desc = Self {
                flags: PropFlags::from_bits_retain(input.get_u32_ne().swap_bytes()),
                size: input.get_u32_ne().swap_bytes(),
                name: input.get_str31(),
            };
        } else {
            desc = Self {
                flags: PropFlags::from_bits_retain(input.get_u32_ne()),
                size: input.get_u32_ne(),
                name: input.get_str31(),
            };
        }

        desc
    }

    pub fn to_bytes(&self, swap: bool) -> Vec<u8> {
        let mut buf = vec![];

        if swap {
            buf.put_u32_ne(self.flags.bits().swap_bytes());
            buf.put_u32_ne(self.size.swap_bytes());
        } else {
            buf.put_u32_ne(self.flags.bits());
            buf.put_u32_ne(self.size);
        }

        buf.put_str31(&self.name[..]);

        buf
    }
}

#[derive(Debug)]
pub struct AssetSpec {
    pub id: i32,
    pub crc: u32,
}

impl AssetSpec {
    pub fn from_bytes(input: &[u8], swap: bool) -> Self {
        let spec: Self;

        if swap {
            spec = Self {
                id: input.get_i32_ne().swap_bytes(),
                crc: input.get_u32_ne().swap_bytes(),
            };
        } else {
            spec = Self {
                id: input.get_i32_ne(),
                crc: input.get_u32_ne(),
            };
        }

        spec
    }

    pub fn to_bytes(&self, swap: bool) -> Vec<u8> {
        let mut buf = vec![];

        if swap {
            buf.put_i32_ne(self.id.swap_bytes());
            buf.put_u32_ne(self.crc.swap_bytes());
        } else {
            buf.put_i32_ne(self.id);
            buf.put_u32_ne(self.crc);
        }

        buf
    }
}

#[inline]
pub fn prop_crc32(input: &[u8]) -> u32 {
    crc32(input, 0xD9216290)
}

cfg_if! {
    if #[cfg(feature = "prop")] {
        use flate2::read::ZlibDecoder;
        use flate2::write::ZlibEncoder;
        use png::{Decoder, Encoder};
        use std::io;

        const BIT16: f64 = 0.12156862745098039;
        const DITHER_20BIT: f64 = 4.0476190476190474;
        const DITHER_S20BIT: f64 = 8.225806451612904;

        #[derive(Debug)]
        pub struct Prop {
            spec: AssetSpec,
            desc: AssetDescriptor,
            w: i16,
            h: i16,
            offset: Point,
            img: Vec<u8>,
        }

        impl Prop {
            pub fn decode32(input: &[u8], swap: bool) -> io::Result<Self> {

            }

            pub fn to_bytes(&self, swap: bool) -> Vec<u8> {
                let mut buf = vec![];

                if swap {
                    buf.put_u32_ne(PROP.swap_bytes());
                    buf.put(self.spec.to_bytes(true)[..]);
                    buf.put_u32_ne(self.desc.size.swap_bytes());
                    buf.put_u32_ne(0); // block offset, always 0
                    buf.put_u16_ne(0); // block number, always 0
                    buf.put_u16_ne(1u16.swap_bytes()); // number of blocks, always 1
                    buf.put(self.desc.to_bytes(true)[..]);
                    buf.put_i16_ne(self.w.swap_bytes());
                    buf.put_i16_ne(self.h.swap_bytes());

                    if self.w > 44 || self.h > 44 {
                        buf.put_u32_ne(0); // 2 i16s of 0
                    } else {
                        buf.put(self.offset.to_bytes(true)[..]);
                    }

                    buf.put_u16_ne(0); // script offset?
                    buf.put_u16_ne(self.flags.bits().swap_bytes());
                } else {
                    buf.put_u32_ne(PROP);
                    buf.put(self.spec.to_bytes(false)[..]);
                    buf.put_u32_ne(self.desc.size);
                    buf.put_u32_ne(0); // block offset, always 0
                    buf.put_u16_ne(0); // block number, always 0
                    buf.put_u16_ne(1); // number of blocks, always 1
                    buf.put(self.desc.to_bytes(false)[..]);
                    buf.put_i16_ne(self.w);
                    buf.put_i16_ne(self.h);

                    if self.w > 44 || self.h > 44 {
                        buf.put_u32_ne(0); // 2 i16s of 0
                    } else {
                        buf.put(self.offset.to_bytes(false)[..]);
                    }

                    buf.put_u16_ne(0); // script offset?
                    buf.put_u16_ne(self.flags.bits());
                }

                buf.put(&self.img[..]);

                buf
            }
        }
    }
}
