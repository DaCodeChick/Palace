use bitflags::bitflags;
use bytes::{Buf, BufMut};

use crate::{BufExt, BufMutExt};

pub mod msg;
pub use msg::*;

bitflags! {
    pub struct DownloadCaps: u32 {
        const ASSETS_PALACE = 1;
        const ASSETS_FTP = 2;
        const ASSETS_HTTP = 4;
        const ASSETS_OTHER = 8;
        const FILES_PALACE = 16;
        const FILES_FTP = 32;
        const FILES_HTTP = 64;
        const FILES_OTHER = 128;
        const FILES_HTTPSRVR = 256;
        const EXTEND_PKT = 512;
    }

    pub struct Engine2DCaps: u32 {
        const PALACE = 1;
        const DOUBLEBYTE = 2;
    }

    pub struct Engine3DCaps: u32 {
        const VRML1 = 1;
        const VRML2 = 2;
    }

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

    pub struct Graphics2DCaps: u32 {
        const GIF87 = 1;
        const GIF89A = 2;
        const JPG = 4;
        const TIFF = 8;
        const TARGA = 16;
        const BMP = 32;
        const PCT = 64;
    }

    /// User's machine attributes
    pub struct RegistrationFlags: u32 {
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

    pub struct ScriptEvent: u32 {
        const SELECT = 1;
        const LOCK = 2;
        const UNLOCK = 4;
        const HIDE = 8;
        const SHOW = 16;
        const STARTUP = 32;
        const ALARM = 64;
        const CUSTOM = 128;
        const IN_CHAT = 256;
        const PROP_CHANGE = 512;
        const ENTER = 1024;
        const LEAVE = 2048;
        const OUT_CHAT = 4096;
        const SIGN_ON = 8192;
        const SIGN_OFF = 16384;
        const MACRO0 = 32768;
        const MACRO1 = 65536;
        const MACRO2 = 0x20000;
        const MACRO3 = 0x40000;
        const MACRO4 = 0x80000;
        const MACRO5 = 0x100000;
        const MACRO6 = 0x200000;
        const MACRO7 = 0x400000;
        const MACRO8 = 0x800000;
        const MACRO9 = 0x1000000;
    }

    /// Server info flags
    pub struct ServerFlags: u16 {
        const DIRECTPLAY = 1;
        const CLOSED = 2;
        const GUESTS_ARE_MEMBERS = 4;
        const INSTANTPALACE = 16;
        const PALACEPRESENTS = 32;
    }

    pub struct UploadCaps: u32 {
        const ASSETS_PALACE = 1;
        const ASSETS_FTP = 2;
        const ASSETS_HTTP = 4;
        const ASSETS_OTHER = 8;
        const FILES_PALACE = 16;
        const FILES_FTP = 32;
        const FILES_HTTP = 64;
        const FILES_OTHER = 128;
        const EXTEND_PKT = 256;
    }

    pub struct UserFlags: u16 {
        const SUPERUSER = 1;
        const GOD = 2;
        const KILL = 4;
        const GUEST = 8;
		#[deprecated(note = "Redundant with KILL, shouldn't be used")]
        const BANISHED = 16;
		#[deprecated(note = "Historical artifact, shouldn't be used")]
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

/// Draw data
#[derive(Debug)]
pub struct Draw {
    cmd: DrawCmd,
    data: Vec<u8>,
}

/// Draw command
#[derive(Debug)]
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
#[derive(Debug)]
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

#[derive(Debug)]
#[repr(u16)]
pub enum HotspotState {
    Unlock = 0,
    Lock,
}

#[derive(Debug)]
#[repr(u16)]
pub enum HotspotType {
    Normal = 0,
    Door,
    ShutableDoor,
    LockableDoor,
    Bolt,
    NavArea,
}

#[derive(Debug)]
pub struct LProp {
    spec: AssetSpec,
    flags: PropFlags,
    refnum: i32,
    loc: Point,
}

#[derive(Debug)]
#[repr(u32)]
pub enum NavError {
    Internal = 0,
    RoomUnknown,
    RoomFull,
    RoomClosed,
    CantAuthor,
    PalaceFull,
}

#[derive(Debug)]
#[repr(u8)]
pub enum Platform {
    Mac = 0,
    Win95,
    WinNT,
    Unix,
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

#[derive(Debug)]
pub struct State {
    pic_id: u16,
    pic_loc: Point,
}
