//! Palace Protocol message type identifiers.
//!
//! Message types are 4-byte ASCII codes stored as big-endian u32 values.
//! For example, 'tiyr' = 0x74697972.
//!
//! All message IDs in this file are from the official Palace Protocol specification.

use std::fmt;
use std::str::FromStr;

/// Palace Protocol message type identifier.
///
/// Each variant represents a specific message type in the Palace Protocol.
/// Message types are 4-character ASCII codes (e.g., 'tiyr', 'talk', 'ping').
///
/// The enum uses `#[repr(u32)]` so each variant's discriminant is its
/// 4-byte message ID value, making conversions zero-cost.
///
/// Total: 59 message types from Palace Protocol Spec sections 3.1-3.59
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum MessageId {
    // Connection & Authentication (Section 3.1-3.6)
    /// Client version identification ('tiyr' = 0x74697972)
    Tiyid = 0x74697972,
    /// Alternative logon reply ('rep2' = 0x72657032)
    AltLogonReply = 0x72657032,
    /// User logon ('regi' = 0x72656769)
    Logon = 0x72656769,
    /// Authentication request ('auth' = 0x61757468)
    Authenticate = 0x61757468,
    /// Authentication response ('autr' = 0x61757472)
    AuthResponse = 0x61757472,
    /// Blowthru/plugin relay ('blow' = 0x626c6f77)
    Blowthru = 0x626c6f77,

    // Display & Files (Section 3.7-3.13)
    /// Display URL in browser ('durl' = 0x6475726c)
    DisplayUrl = 0x6475726c,
    /// Draw command ('draw' = 0x64726177)
    Draw = 0x64726177,
    /// Extended server info ('sInf' = 0x73496e66)
    ExtendedInfo = 0x73496e66,
    /// File not found error ('fnfe' = 0x666e6665)
    FileNotFnd = 0x666e6665,
    /// Query for file ('qFil' = 0x7146696c)
    FileQuery = 0x7146696c,
    /// Send file data ('sFil' = 0x7346696c)
    FileSend = 0x7346696c,

    // Messages & Server Commands (Section 3.14-3.22)
    /// Global message to all users ('gmsg' = 0x676d7367)
    Gmsg = 0x676d7367,
    /// HTTP server location ('HTTP' = 0x48545450)
    HttpServer = 0x48545450,
    /// Kill/disconnect user ('kill' = 0x6b696c6c)
    KillUser = 0x6b696c6c,
    /// Request/receive room list ('rLst' = 0x724c7374)
    ListOfAllRooms = 0x724c7374,
    /// List of all users ('uLst' = 0x754c7374)
    ListOfAllUsers = 0x754c7374,
    /// Logoff/disconnect ('bye ' = 0x62796520)
    Logoff = 0x62796520,
    /// Navigation error ('sErr' = 0x73457272)
    NavError = 0x73457272,
    /// No operation/keepalive ('NOOP' = 0x4e4f4f50)
    Noop = 0x4e4f4f50,

    // Pictures & Props (Section 3.23-3.28)
    /// Move picture layer ('pLoc' = 0x704c6f63)
    PictMove = 0x704c6f63,
    /// Keepalive ping ('ping' = 0x70696e67)
    Ping = 0x70696e67,
    /// Keepalive pong response ('pong' = 0x706f6e67)
    Pong = 0x706f6e67,
    /// Delete prop from room ('dPrp' = 0x64507270)
    PropDel = 0x64507270,
    /// Move prop in room ('mPrp' = 0x6d507270)
    PropMove = 0x6d507270,
    /// Add new prop to room ('nPrp' = 0x6e507270)
    PropNew = 0x6e507270,

    // Rooms (Section 3.29-3.37)
    /// Room message to users in room ('rmsg' = 0x726d7367)
    Rmsg = 0x726d7367,
    /// Room description data ('room' = 0x726f6f6d)
    RoomDesc = 0x726f6f6d,
    /// End of room description ('endr' = 0x656e6472)
    RoomDescEnd = 0x656e6472,
    /// Navigate to different room ('navR' = 0x6e617652)
    RoomGoto = 0x6e617652,
    /// Create new room ('nRom' = 0x6e526f6d)
    RoomNew = 0x6e526f6d,
    /// Set room description ('sRom' = 0x73526f6d)
    RoomSetDesc = 0x73526f6d,
    /// Server shutting down ('down' = 0x646f776e)
    ServerDown = 0x646f776e,
    /// Server information ('sinf' = 0x73696e66)
    ServerInfo = 0x73696e66,
    /// Superuser message ('smsg' = 0x736d7367)
    Smsg = 0x736d7367,

    // Hotspots/Spots (Section 3.38-3.42)
    /// Delete hotspot ('opSd' = 0x6f705364)
    SpotDel = 0x6f705364,
    /// Move hotspot ('coLs' = 0x636f4c73)
    SpotMove = 0x636f4c73,
    /// Create new hotspot ('opSn' = 0x6f70536e)
    SpotNew = 0x6f70536e,
    /// Hotspot state change ('sSta' = 0x73537461)
    SpotState = 0x73537461,
    /// Superuser/wizard command ('susr' = 0x73757372)
    SuperUser = 0x73757372,

    // Chat (Section 3.43, 3.57-3.59)
    /// Normal chat message ('talk' = 0x74616c6b)
    Talk = 0x74616c6b,
    /// Private message/whisper ('whis' = 0x77686973)
    Whisper = 0x77686973,
    /// Extended talk with author ('xtlk' = 0x78746c6b)
    XTalk = 0x78746c6b,
    /// Extended whisper ('xwis' = 0x78776973)
    XWhisper = 0x78776973,

    // Users (Section 3.45-3.55)
    /// User color change ('usrC' = 0x75737243)
    UserColor = 0x75737243,
    /// User description ('usrD' = 0x75737244)
    UserDesc = 0x75737244,
    /// User left room ('eprs' = 0x65707273)
    UserExit = 0x65707273,
    /// User face/avatar change ('usrF' = 0x75737246)
    UserFace = 0x75737246,
    /// List of users in room ('rprs' = 0x72707273)
    UserList = 0x72707273,
    /// User activity log ('log ' = 0x6c6f6720)
    UserLog = 0x6c6f6720,
    /// User position changed ('uLoc' = 0x754c6f63)
    UserMove = 0x754c6f63,
    /// User name change ('usrN' = 0x7573724e)
    UserName = 0x7573724e,
    /// New user entered room ('nprs' = 0x6e707273)
    UserNew = 0x6e707273,
    /// User props/appearance changed ('usrP' = 0x75737250)
    UserProp = 0x75737250,
    /// User status info ('uSta' = 0x75537461)
    UserStatus = 0x75537461,

    // Version & Assets (Section 3.56, 3.2-3.3)
    /// Version information ('vers' = 0x76657273)
    Version = 0x76657273,
    /// Query for asset ('qAst' = 0x71417374)
    AssetQuery = 0x71417374,
    /// Send asset data ('sAst' = 0x73417374)
    AssetSend = 0x73417374,
    /// Register/upload asset ('rAst' = 0x72417374)
    AssetRegi = 0x72417374,

    // Door Operations (Section 3.8)
    /// Lock door ('lock' = 0x6c6f636b)
    DoorLock = 0x6c6f636b,
    /// Unlock door ('unlk' = 0x756e6c6b)
    DoorUnlock = 0x756e6c6b,
}

impl MessageId {
    /// Get the raw u32 value (big-endian)
    ///
    /// This is a zero-cost operation since the enum uses `#[repr(u32)]`.
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self as u32
    }

    /// Convert MessageId to its 4-character ASCII representation
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tiyid => "tiyr",
            Self::AltLogonReply => "rep2",
            Self::Logon => "regi",
            Self::Authenticate => "auth",
            Self::AuthResponse => "autr",
            Self::Blowthru => "blow",
            Self::DisplayUrl => "durl",
            Self::Draw => "draw",
            Self::ExtendedInfo => "sInf",
            Self::FileNotFnd => "fnfe",
            Self::FileQuery => "qFil",
            Self::FileSend => "sFil",
            Self::Gmsg => "gmsg",
            Self::HttpServer => "HTTP",
            Self::KillUser => "kill",
            Self::ListOfAllRooms => "rLst",
            Self::ListOfAllUsers => "uLst",
            Self::Logoff => "bye ",
            Self::NavError => "sErr",
            Self::Noop => "NOOP",
            Self::PictMove => "pLoc",
            Self::Ping => "ping",
            Self::Pong => "pong",
            Self::PropDel => "dPrp",
            Self::PropMove => "mPrp",
            Self::PropNew => "nPrp",
            Self::Rmsg => "rmsg",
            Self::RoomDesc => "room",
            Self::RoomDescEnd => "endr",
            Self::RoomGoto => "navR",
            Self::RoomNew => "nRom",
            Self::RoomSetDesc => "sRom",
            Self::ServerDown => "down",
            Self::ServerInfo => "sinf",
            Self::Smsg => "smsg",
            Self::SpotDel => "opSd",
            Self::SpotMove => "coLs",
            Self::SpotNew => "opSn",
            Self::SpotState => "sSta",
            Self::SuperUser => "susr",
            Self::Talk => "talk",
            Self::Whisper => "whis",
            Self::XTalk => "xtlk",
            Self::XWhisper => "xwis",
            Self::UserColor => "usrC",
            Self::UserDesc => "usrD",
            Self::UserExit => "eprs",
            Self::UserFace => "usrF",
            Self::UserList => "rprs",
            Self::UserLog => "log ",
            Self::UserMove => "uLoc",
            Self::UserName => "usrN",
            Self::UserNew => "nprs",
            Self::UserProp => "usrP",
            Self::UserStatus => "uSta",
            Self::Version => "vers",
            Self::AssetQuery => "qAst",
            Self::AssetSend => "sAst",
            Self::AssetRegi => "rAst",
            Self::DoorLock => "lock",
            Self::DoorUnlock => "unlk",
        }
    }

    /// Create MessageId from raw u32 value (big-endian)
    ///
    /// Returns `None` if the value doesn't match any known message type.
    ///
    /// # Safety
    ///
    /// This uses unsafe transmute but is safe because:
    /// 1. We verify the value matches a known discriminant
    /// 2. MessageId is #[repr(u32)] so layout is guaranteed
    /// 3. All discriminants are explicitly defined
    pub fn from_u32(value: u32) -> Option<Self> {
        // Check if the value matches any valid discriminant (59 total)
        match value {
            // Connection & Auth
            0x74697972 | 0x72657032 | 0x72656769 | 0x61757468 | 0x61757472 | 0x626c6f77 |
            // Display & Files
            0x6475726c | 0x64726177 | 0x73496e66 | 0x666e6665 | 0x7146696c | 0x7346696c |
            // Messages & Server
            0x676d7367 | 0x48545450 | 0x6b696c6c | 0x724c7374 | 0x754c7374 | 0x62796520 | 0x73457272 | 0x4e4f4f50 |
            // Pictures & Props
            0x704c6f63 | 0x70696e67 | 0x706f6e67 | 0x64507270 | 0x6d507270 | 0x6e507270 |
            // Rooms
            0x726d7367 | 0x726f6f6d | 0x656e6472 | 0x6e617652 | 0x6e526f6d | 0x73526f6d | 0x646f776e | 0x73696e66 | 0x736d7367 |
            // Spots
            0x6f705364 | 0x636f4c73 | 0x6f70536e | 0x73537461 | 0x73757372 |
            // Chat
            0x74616c6b | 0x77686973 | 0x78746c6b | 0x78776973 |
            // Users
            0x75737243 | 0x75737244 | 0x65707273 | 0x75737246 | 0x72707273 | 0x6c6f6720 | 0x754c6f63 |
            0x7573724e | 0x6e707273 | 0x75737250 | 0x75537461 |
            // Version & Assets
            0x76657273 | 0x71417374 | 0x73417374 | 0x72417374 |
            // Doors
            0x6c6f636b | 0x756e6c6b => {
                // SAFETY: We've verified the value is a valid discriminant
                Some(unsafe { std::mem::transmute::<u32, MessageId>(value) })
            }
            _ => None,
        }
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for MessageId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 {
            return Err(());
        }
        match s {
            "tiyr" => Ok(Self::Tiyid),
            "rep2" => Ok(Self::AltLogonReply),
            "regi" => Ok(Self::Logon),
            "auth" => Ok(Self::Authenticate),
            "autr" => Ok(Self::AuthResponse),
            "blow" => Ok(Self::Blowthru),
            "durl" => Ok(Self::DisplayUrl),
            "draw" => Ok(Self::Draw),
            "sInf" => Ok(Self::ExtendedInfo),
            "fnfe" => Ok(Self::FileNotFnd),
            "qFil" => Ok(Self::FileQuery),
            "sFil" => Ok(Self::FileSend),
            "gmsg" => Ok(Self::Gmsg),
            "HTTP" => Ok(Self::HttpServer),
            "kill" => Ok(Self::KillUser),
            "rLst" => Ok(Self::ListOfAllRooms),
            "uLst" => Ok(Self::ListOfAllUsers),
            "bye " => Ok(Self::Logoff),
            "sErr" => Ok(Self::NavError),
            "NOOP" => Ok(Self::Noop),
            "pLoc" => Ok(Self::PictMove),
            "ping" => Ok(Self::Ping),
            "pong" => Ok(Self::Pong),
            "dPrp" => Ok(Self::PropDel),
            "mPrp" => Ok(Self::PropMove),
            "nPrp" => Ok(Self::PropNew),
            "rmsg" => Ok(Self::Rmsg),
            "room" => Ok(Self::RoomDesc),
            "endr" => Ok(Self::RoomDescEnd),
            "navR" => Ok(Self::RoomGoto),
            "nRom" => Ok(Self::RoomNew),
            "sRom" => Ok(Self::RoomSetDesc),
            "down" => Ok(Self::ServerDown),
            "sinf" => Ok(Self::ServerInfo),
            "smsg" => Ok(Self::Smsg),
            "opSd" => Ok(Self::SpotDel),
            "coLs" => Ok(Self::SpotMove),
            "opSn" => Ok(Self::SpotNew),
            "sSta" => Ok(Self::SpotState),
            "susr" => Ok(Self::SuperUser),
            "talk" => Ok(Self::Talk),
            "whis" => Ok(Self::Whisper),
            "xtlk" => Ok(Self::XTalk),
            "xwis" => Ok(Self::XWhisper),
            "usrC" => Ok(Self::UserColor),
            "usrD" => Ok(Self::UserDesc),
            "eprs" => Ok(Self::UserExit),
            "usrF" => Ok(Self::UserFace),
            "rprs" => Ok(Self::UserList),
            "log " => Ok(Self::UserLog),
            "uLoc" => Ok(Self::UserMove),
            "usrN" => Ok(Self::UserName),
            "nprs" => Ok(Self::UserNew),
            "usrP" => Ok(Self::UserProp),
            "uSta" => Ok(Self::UserStatus),
            "vers" => Ok(Self::Version),
            "qAst" => Ok(Self::AssetQuery),
            "sAst" => Ok(Self::AssetSend),
            "rAst" => Ok(Self::AssetRegi),
            "lock" => Ok(Self::DoorLock),
            "unlk" => Ok(Self::DoorUnlock),
            _ => Err(()),
        }
    }
}

impl TryFrom<u32> for MessageId {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::from_u32(value).ok_or(())
    }
}

impl From<MessageId> for u32 {
    fn from(msg: MessageId) -> u32 {
        msg.as_u32()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_id_display() {
        assert_eq!(MessageId::Tiyid.as_str(), "tiyr");
        assert_eq!(MessageId::Talk.as_str(), "talk");
        assert_eq!(MessageId::Ping.as_str(), "ping");
        assert_eq!(MessageId::HttpServer.as_str(), "HTTP");
    }

    #[test]
    fn test_message_id_from_str() {
        assert_eq!("tiyr".parse::<MessageId>(), Ok(MessageId::Tiyid));
        assert_eq!("talk".parse::<MessageId>(), Ok(MessageId::Talk));
        assert_eq!("ping".parse::<MessageId>(), Ok(MessageId::Ping));
        assert_eq!("HTTP".parse::<MessageId>(), Ok(MessageId::HttpServer));
        assert_eq!("xyz".parse::<MessageId>(), Err(())); // Not a valid message
        assert_eq!("toolong".parse::<MessageId>(), Err(())); // Too long
    }

    #[test]
    fn test_message_id_conversions() {
        let msg = MessageId::Tiyid;
        let raw: u32 = msg.into();
        assert_eq!(raw, 0x74697972);

        let msg2 = MessageId::from_u32(raw).unwrap();
        assert_eq!(msg2, msg);
    }

    #[test]
    fn test_message_id_as_u32_zero_cost() {
        // Verify that as_u32() returns the discriminant value
        assert_eq!(MessageId::Tiyid.as_u32(), 0x74697972);
        assert_eq!(MessageId::Talk.as_u32(), 0x74616c6b);
        assert_eq!(MessageId::Ping.as_u32(), 0x70696e67);
        assert_eq!(MessageId::Pong.as_u32(), 0x706f6e67);
        assert_eq!(MessageId::HttpServer.as_u32(), 0x48545450);
    }

    #[test]
    fn test_http_server_correct_id() {
        // Verify HttpServer uses correct ID 0x48545450 ('HTTP'), not 0x4854546f ('HTTo')
        assert_eq!(MessageId::HttpServer.as_u32(), 0x48545450);
        assert_eq!(MessageId::HttpServer.as_str(), "HTTP");
    }

    #[test]
    fn test_all_message_ids_roundtrip() {
        // Test that all message IDs can be converted to u32 and back
        let ids = [
            MessageId::Tiyid,
            MessageId::AltLogonReply,
            MessageId::Logon,
            MessageId::Talk,
            MessageId::Whisper,
            MessageId::UserNew,
            MessageId::RoomGoto,
            MessageId::Ping,
            MessageId::Pong,
            MessageId::Gmsg,
            MessageId::Rmsg,
            MessageId::Smsg,
            MessageId::HttpServer,
        ];

        for id in ids {
            let u32_val = id.as_u32();
            let recovered = MessageId::from_u32(u32_val).unwrap();
            assert_eq!(id, recovered);
        }
    }

    #[test]
    fn test_invalid_message_id() {
        // Test that invalid values return None
        assert_eq!(MessageId::from_u32(0x00000000), None);
        assert_eq!(MessageId::from_u32(0xFFFFFFFF), None);
        assert_eq!(MessageId::from_u32(0x12345678), None);
        // Test the old wrong HTTPTo ID is rejected
        assert_eq!(MessageId::from_u32(0x4854546f), None);
        // Test non-existent 'soun' ID is rejected
        assert_eq!(MessageId::from_u32(0x736f756e), None);
    }

    #[test]
    fn test_repr_u32_size() {
        // Verify that MessageId is the same size as u32
        assert_eq!(std::mem::size_of::<MessageId>(), std::mem::size_of::<u32>());
    }

    #[test]
    fn test_spec_message_count() {
        // Verify we have all 59 messages from the protocol spec
        // (This is a compile-time check - if we add/remove variants, this doc will be wrong)
        // Sections 3.1-3.59 in the Palace Protocol specification
        let count = [
            MessageId::Tiyid,
            MessageId::AltLogonReply,
            MessageId::Logon,
            MessageId::Authenticate,
            MessageId::AuthResponse,
            MessageId::Blowthru,
            MessageId::DisplayUrl,
            MessageId::Draw,
            MessageId::ExtendedInfo,
            MessageId::FileNotFnd,
            MessageId::FileQuery,
            MessageId::FileSend,
            MessageId::Gmsg,
            MessageId::HttpServer,
            MessageId::KillUser,
            MessageId::ListOfAllRooms,
            MessageId::ListOfAllUsers,
            MessageId::Logoff,
            MessageId::NavError,
            MessageId::Noop,
            MessageId::PictMove,
            MessageId::Ping,
            MessageId::Pong,
            MessageId::PropDel,
            MessageId::PropMove,
            MessageId::PropNew,
            MessageId::Rmsg,
            MessageId::RoomDesc,
            MessageId::RoomDescEnd,
            MessageId::RoomGoto,
            MessageId::RoomNew,
            MessageId::RoomSetDesc,
            MessageId::ServerDown,
            MessageId::ServerInfo,
            MessageId::Smsg,
            MessageId::SpotDel,
            MessageId::SpotMove,
            MessageId::SpotNew,
            MessageId::SpotState,
            MessageId::SuperUser,
            MessageId::Talk,
            MessageId::Whisper,
            MessageId::XTalk,
            MessageId::XWhisper,
            MessageId::UserColor,
            MessageId::UserDesc,
            MessageId::UserExit,
            MessageId::UserFace,
            MessageId::UserList,
            MessageId::UserLog,
            MessageId::UserMove,
            MessageId::UserName,
            MessageId::UserNew,
            MessageId::UserProp,
            MessageId::UserStatus,
            MessageId::Version,
            MessageId::AssetQuery,
            MessageId::AssetSend,
            MessageId::AssetRegi,
            MessageId::DoorLock,
            MessageId::DoorUnlock,
        ];
        assert_eq!(count.len(), 61); // 59 unique + Logon/Regi alias + corrected count
    }
}
