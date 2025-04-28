use bitflags::bitflags;
use bytes::{Buf, BufMut};

use crate::{BufExt, BufMutExt};

pub mod msg;
pub use msg::*;

bitflags! {
	/// Flags for the download capabilities
	#[derive(Debug, Clone, Copy)]
    pub struct DownloadCaps: u32 {
        const ASSETS_PALACE = 1; /// Assets from Palace
        const ASSETS_FTP = 2; /// Assets from FTP
        const ASSETS_HTTP = 4; /// Assets from HTTP
        const ASSETS_OTHER = 8; /// Assets from other
        const FILES_PALACE = 16; /// Files from Palace
        const FILES_FTP = 32; /// Files from FTP
        const FILES_HTTP = 64; /// Files from HTTP
        const FILES_OTHER = 128; /// Files from other
        const FILES_HTTPSRVR = 256;
        const EXTEND_PKT = 512; /// Extended packet
    }

	/// 2D engine capabilities
	#[derive(Debug, Clone, Copy)]
    pub struct Engine2DCaps: u32 {
        const PALACE = 1; /// Palace
        const DOUBLEBYTE = 2; /// Double byte
    }

	/// Flags for the 3D engine capabilities
	#[derive(Debug, Clone, Copy)]
    pub struct Engine3DCaps: u32 {
        const VRML1 = 1; /// VRML1
        const VRML2 = 2; /// VRML2
    }

	/// Flags for the extended info packet
	#[derive(Debug, Clone, Copy)]
    pub struct ExtendedInfoFlags: u32 {
        const AVATAR_URL = 1; /// Avatar URL
        const SERVER_VERSION = 2; /// Server version
        const SERVER_TYPE = 4; /// Server type
        const SERVER_FLAGS = 8; /// Server flags
        const NUM_USERS = 16; /// Number of users
        const SERVER_NAME = 32; /// Server name
        const HTTP_URL = 64; /// HTTP URL
    }

	/// Flags for the graphics 2D capabilities
	#[derive(Debug, Clone, Copy)]
    pub struct Graphics2DCaps: u32 {
        const GIF87 = 1;
        const GIF89A = 2;
        const JPG = 4;
        const TIFF = 8;
        const TARGA = 16;
        const BMP = 32; /// Bitmap
        const PCT = 64;
    }

	#[derive(Debug, Clone, Copy)]
    pub struct RegistrationFlags: u32 {
        const UNKNOWN_MACH = 0; /// Unknown machine	
        const MAC68K = 1; /// Mac 68K
        const MACPPC = 2; /// Mac PPC
        const WIN16 = 3; /// Windows 16-bit
        const WIN32 = 4; /// Windows 32-bit
        const JAVA = 5; /// Java
        const OS_MASK = 15; /// OS mask
        const AUTH = 0x80000000; /// Authenticated
    }

    /// Flags for rooms
	#[derive(Debug, Clone, Copy)]
    pub struct RoomFlags: u16 {
        const AUTHOR_LOCKED = 1; /// Author locked
        const PRIVATE = 2; /// Private
        const NO_PAINT = 4; /// No paint
        const CLOSED = 8; /// Closed
        const NO_SCRIPT = 16; /// No scripts
        const HIDDEN = 32; /// Hidden
        const NO_GUESTS = 64; /// No guests
        const WIZARDS_ONLY = 128; /// Wizards only
        const DROP_ZONE = 256; /// Drop zone
        const NO_LPROPS = 512; /// No loose props
    }

	/// Script events
	#[derive(Debug, Clone, Copy)]
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

    /// Flags for the server
	#[derive(Debug, Clone, Copy)]
    pub struct ServerFlags: u16 {
        const DIRECTPLAY = 1; /// DirectPlay
        const CLOSED = 2; /// Closed
        const GUESTS_ARE_MEMBERS = 4; /// Guests are members
        const INSTANTPALACE = 16; /// Instant Palace
        const PALACEPRESENTS = 32; /// Palace Presents
    }

	/// Upload capabilities
	#[derive(Debug, Clone, Copy)]
    pub struct UploadCaps: u32 {
        const ASSETS_PALACE = 1; /// Assets from Palace
        const ASSETS_FTP = 2; /// Assets from FTP
        const ASSETS_HTTP = 4; /// Assets from HTTP
        const ASSETS_OTHER = 8; /// Assets from other
        const FILES_PALACE = 16; /// Files from Palace
        const FILES_FTP = 32; /// Files from FTP
        const FILES_HTTP = 64; /// Files from HTTP
        const FILES_OTHER = 128; /// Files from other
        const EXTEND_PKT = 256; /// Extended packet
    }

	/// Flags for the user
	#[derive(Debug, Clone, Copy)]
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
        const REJECT_WHISPER = 2048; /// Reject whispers
        const PROP_GAG = 4096; /// Prop gag
    }
}


#[derive(Debug, Clone)]
pub struct Draw {
    cmd: DrawCmd,
    data: Vec<u8>,
}

/// This is used to identify the type of drawing command
/// that is being sent in the draw packet.
/// The command is a 16-bit unsigned integer, and the
/// values are defined in the Palace protocol documentation.
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum DrawCmd {
    Path = 0,
    Shape,
    Text,
    Detonate,
    Delete,
    Ellipse,
}

/// This is used to identify the type of extended info
/// that is being sent in the extended info packet.
/// The ID is a 32-bit unsigned integer, and the values
/// are defined in the Palace protocol documentation.
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum ExtendedInfoID {
    AuthNeeded = 0x41555448,
    AvatarURL = 0x4155524C,
    Flags = 0x464C4147,
    HttpURL = 0x4855524C,
    Name = 0x4E414D45,
    NumUsers = 0x4E555352, /// Number of users
    Password = 0x50415353,
    Type = 0x54595045,
    Unknown = 0x554E4B4E,
    Version = 0x56455253,
}

#[derive(Debug, Clone)]
pub struct FileDescriptor {
    num_blocks: u16,
    size: u32,
    name: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum HotspotState {
    Unlock = 0,
    Lock,
}

/// This is used to identify the type of hotspot
/// that is being sent in the hotspot packet.
/// The type is a 16-bit unsigned integer, and the
/// values are defined in the Palace protocol documentation.
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum HotspotType {
    Normal = 0, /// Normal hotspot
    Door, /// Door hotspot
    ShutableDoor, /// Shutable door hotspot
    LockableDoor, /// Lockable door hotspot
    Bolt, /// Bolt hotspot
    NavArea, /// Nav area hotspot
}

#[derive(Debug, Clone, Copy)]
pub struct LProp {
    spec: AssetSpec,
    flags: PropFlags,
    refnum: i32,
    loc: Point,
}

/// This is used to identify the type of navigation error
/// that is being sent in the navigation error packet.
/// The error is a 32-bit unsigned integer, and the values
/// are defined in the Palace protocol documentation.
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum NavError {
    Internal = 0,
    RoomUnknown,
    RoomFull,
    RoomClosed,
    CantAuthor,
    PalaceFull,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Platform {
    Mac = 0,
    Win95,
    WinNT,
    Unix,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Copy)]
pub struct State {
    pic_id: u16,
    pic_loc: Point,
}
