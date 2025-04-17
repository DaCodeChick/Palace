use bitflags::bitflags;
use bytes::{Buf, BufMut};
use prop::PropFlags;

use crate::{BufExt, BufMutExt};

pub mod msg;
pub use msg::*;

bitflags! {
	/// Server info request flags
	pub struct ExtendedInfoFlags: u32 {
		const AVATAR_URL = 1;
		const SERVER_VERSION = 2;
		const SERVER_TYPE = 4;
		const SERVER_FLAGS = 8;
		const NUM_USERS = 16;
		const SERVER_NAME = 32;
		const HTTP_URL = 64;
	}

	/// User's machine attributes
	pub struct RegFlags: u32 {
		const UNKNOWN_MACH = 0;
		const MAC68K = 1;
		const MACPPC = 2;
		const WIN16 = 3;
		const WIN32 = 4;
		const JAVA = 5;
		const OS_MASK = 15;
		const AUTH = 0x80000000;
	}

	/// Room flags
	pub struct RoomFlags: u16 {
		const AUTHOR_LOCKED = 1;
		const PRIVATE = 2;
		const NO_PAINT = 4;
		const CLOSED = 8;
		const NO_SCRIPT = 16;
		const HIDDEN = 32;
		const NO_GUESTS = 64;
		const WIZARDS_ONLY = 128;
		const DROP_ZONE = 256;
		const NO_LPROPS = 512;
	}

	/// Server info flags
	pub struct ServerFlags: u16 {
		const DIRECTPLAY = 1;
		const CLOSED = 2;
		const GUESTS_ARE_MEMBERS = 4;
		const INSTANTPALACE = 16;
		const PALACEPRESENTS = 32;
	}

	pub struct UserFlags: u16 {
		const SUPERUSER = 1;
		const GOD = 2;
		const KILL = 4;
		const GUEST = 8;
		const BANISHED = 16;
		const PENALIZED = 32;
		const ERROR = 64;
		const GAG = 128;
		const PIN = 256;
		const HIDE = 512;
		const REJECT_ESP = 1024;
		const REJECT_WHISPER = 2048;
		const PROP_GAG = 4096;
	}
}

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

/// Draw data
#[derive(Debug)]
pub struct Draw {
	cmd: DrawCmd,
	data: Vec<u8>,
}

/// Draw command
#[repr(u16)]
pub enum DrawCmd {
	Path = 0,
	Shape,
	Text,
	Detonate,
	Delete,
	Ellipse,
}

/// Extended info ID code
#[repr(u32)]
pub enum ExtendedInfoID {
	AuthNeeded = 0x41555448,
	AvatarURL = 0x4155524C,
	Flags = 0x464C4147,
	HttpURL = 0x4855524C,
	Name = 0x4E414D45,
	NumUsers = 0x4E555352,
	Password = 0x50415353,
	Type = 0x54595045,
	Unknown = 0x554E4B4E,
	Version = 0x56455253,
}

/// File descriptor for transfers
#[derive(Debug)]
pub struct FileDescriptor {
    num_blocks: u16,
    size: u32,
    name: Vec<u8>,
}

#[repr(u8)]
pub enum Platform {
	Mac = 0,
	Win95,
	WinNT,
	Unix,
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

/// Sent upon user login
#[derive(Debug)]
pub struct Registration {
	crc: u32,
	counter: u32,
	name: Vec<u8>,
	password: Vec<u8>,
	flags: RegFlags,
	puid_ctr: u32,
	puid_crc: u32,
	demo_elapsed: u32,
	total_elapsed: u32,
	demo_limit: u32,
	room: u16,
	reserved: [u8; 6],
	req_protocol_ver: u32,
}
