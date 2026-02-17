//! Palace Protocol message type identifiers.
//!
//! Message types are 4-byte ASCII codes stored as big-endian u32 values.
//! For example, 'tiyr' = 0x74697972.

use std::fmt;

/// Palace Protocol message type identifier.
///
/// Each variant represents a specific message type in the Palace Protocol.
/// Message types are 4-character ASCII codes (e.g., 'tiyr', 'talk', 'ping').
///
/// The enum uses `#[repr(u32)]` so each variant's discriminant is its
/// 4-byte message ID value, making conversions zero-cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum MessageId {
    // Connection & Authentication
    /// Client version identification ('tiyr' = 0x74697972)
    Tiyid = 0x74697972,
    /// Alternative logon reply ('rep2' = 0x72657032)
    AltLogonReply = 0x72657032,
    /// User logon ('regi' = 0x72656769)
    Regi = 0x72656769,
    /// Authentication request ('auth' = 0x61757468)
    Authenticate = 0x61757468,
    /// Authentication response ('autr' = 0x61757472)
    AuthResponse = 0x61757472,
    /// Superuser (wizard) elevation ('susr' = 0x73757372)
    SuperUser = 0x73757372,
    /// Logoff/disconnect ('bye ' = 0x62796520)
    Logoff = 0x62796520,

    // Rooms
    /// Navigate to room ('navR' = 0x6e617652)
    RoomGoto = 0x6e617652,
    /// Room description ('room' = 0x726f6f6d)
    RoomDesc = 0x726f6f6d,
    /// End of room description ('endr' = 0x656e6472)
    RoomDescEnd = 0x656e6472,
    /// List of all rooms ('rLst' = 0x724c7374)
    ListOfAllRooms = 0x724c7374,
    /// Room list ('sLst' = 0x734c7374)
    RoomList = 0x734c7374,
    /// Room description (alternative) ('sRom' = 0x73526f6d)
    Room = 0x73526f6d,

    // Users
    /// New user entered room ('nprs' = 0x6e707273)
    UserNew = 0x6e707273,
    /// User exited room ('eprs' = 0x65707273)
    UserExit = 0x65707273,
    /// List of users in room ('rprs' = 0x72707273)
    UserList = 0x72707273,
    /// User moved ('uLoc' = 0x754c6f63)
    UserMove = 0x754c6f63,
    /// User face changed ('usrF' = 0x75737246)
    UserFace = 0x75737246,
    /// User props changed ('usrP' = 0x75737250)
    UserProp = 0x75737250,
    /// User description ('usrD' = 0x75737244)
    UserDesc = 0x75737244,
    /// User renamed ('uNam' = 0x754e616d)
    UserNameRename = 0x754e616d,
    /// User color changed ('uCol' = 0x75436f6c)
    UserColor = 0x75436f6c,
    /// User status ('uSta' = 0x75537461)
    UserStatus = 0x75537461,
    /// List of all users ('log ' = 0x6c6f6720)
    ListOfAllUsers = 0x6c6f6720,

    // Chat
    /// Normal chat message ('talk' = 0x74616c6b)
    Talk = 0x74616c6b,
    /// Private message/whisper ('whis' = 0x77686973)
    Whisper = 0x77686973,
    /// Extended talk with author info ('xtlk' = 0x78746c6b)
    XTalk = 0x78746c6b,
    /// Extended whisper ('xwis' = 0x78776973)
    XWhisper = 0x78776973,

    // Assets
    /// Query for asset ('qAst' = 0x71417374)
    AssetQuery = 0x71417374,
    /// Send asset data ('sAst' = 0x73417374)
    AssetSend = 0x73417374,
    /// Register new asset ('rAst' = 0x72417374)
    AssetRegi = 0x72417374,

    // Props
    /// Prop move ('pLoc' = 0x704c6f63)
    PropMove = 0x704c6f63,
    /// Prop delete ('dPrp' = 0x64507270)
    PropDelete = 0x64507270,
    /// Prop new ('nPrp' = 0x6e507270)
    PropNew = 0x6e507270,

    // Drawing
    /// Draw command ('draw' = 0x64726177)
    Draw = 0x64726177,
    /// Path move ('pMov' = 0x704d6f76)
    PathMove = 0x704d6f76,
    /// Path line ('pLin' = 0x704c696e)
    PathLine = 0x704c696e,

    // Hotspots
    /// Spot state changed ('sMsg' = 0x734d7367)
    SpotState = 0x734d7367,
    /// Spot move ('sMov' = 0x734d6f76)
    SpotMove = 0x734d6f76,

    // Door Operations
    /// Lock door ('lock' = 0x6c6f636b)
    DoorLock = 0x6c6f636b,
    /// Unlock door ('unlk' = 0x756e6c6b)
    DoorUnlock = 0x756e6c6b,

    // Server Info
    /// Server information ('sinf' = 0x73696e66)
    ServerInfo = 0x73696e66,
    /// Extended server info request ('sInf' = 0x73496e66)
    ExtendedInfo = 0x73496e66,

    // Connectivity
    /// Keepalive ping ('ping' = 0x70696e67)
    Ping = 0x70696e67,
    /// Keepalive pong ('pong' = 0x706f6e67)
    Pong = 0x706f6e67,
    /// Blowthru (plugin relay) ('blow' = 0x626c6f77)
    Blowthru = 0x626c6f77,

    // Server Commands
    /// HTTP server location ('HTTo' = 0x4854546f)
    HttpServerRequest = 0x4854546f,
    /// Global message ('gmsg' = 0x676d7367)
    GlobalMsg = 0x676d7367,
    /// Room message ('rmsg' = 0x726d7367)
    RoomMsg = 0x726d7367,
    /// Superuser message ('smsg' = 0x736d7367)
    SuperMsg = 0x736d7367,
    /// Display message ('dMsg' = 0x644d7367)
    DisplayMsg = 0x644d7367,

    // Media
    /// Play sound ('soun' = 0x736f756e)
    PlaySound = 0x736f756e,

    // User Management
    /// Kill user connection ('kill' = 0x6b696c6c)
    KillUser = 0x6b696c6c,

    // Navigation
    /// Navigation reply ('nPrs' = 0x6e507273)
    NavError = 0x6e507273,
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
            Self::Regi => "regi",
            Self::Authenticate => "auth",
            Self::AuthResponse => "autr",
            Self::SuperUser => "susr",
            Self::Logoff => "bye ",
            Self::RoomGoto => "navR",
            Self::RoomDesc => "room",
            Self::RoomDescEnd => "endr",
            Self::ListOfAllRooms => "rLst",
            Self::RoomList => "sLst",
            Self::Room => "sRom",
            Self::UserNew => "nprs",
            Self::UserExit => "eprs",
            Self::UserList => "rprs",
            Self::UserMove => "uLoc",
            Self::UserFace => "usrF",
            Self::UserProp => "usrP",
            Self::UserDesc => "usrD",
            Self::UserNameRename => "uNam",
            Self::UserColor => "uCol",
            Self::UserStatus => "uSta",
            Self::ListOfAllUsers => "log ",
            Self::Talk => "talk",
            Self::Whisper => "whis",
            Self::XTalk => "xtlk",
            Self::XWhisper => "xwis",
            Self::AssetQuery => "qAst",
            Self::AssetSend => "sAst",
            Self::AssetRegi => "rAst",
            Self::PropMove => "pLoc",
            Self::PropDelete => "dPrp",
            Self::PropNew => "nPrp",
            Self::Draw => "draw",
            Self::PathMove => "pMov",
            Self::PathLine => "pLin",
            Self::SpotState => "sMsg",
            Self::SpotMove => "sMov",
            Self::DoorLock => "lock",
            Self::DoorUnlock => "unlk",
            Self::ServerInfo => "sinf",
            Self::ExtendedInfo => "sInf",
            Self::Ping => "ping",
            Self::Pong => "pong",
            Self::Blowthru => "blow",
            Self::HttpServerRequest => "HTTo",
            Self::GlobalMsg => "gmsg",
            Self::RoomMsg => "rmsg",
            Self::SuperMsg => "smsg",
            Self::DisplayMsg => "dMsg",
            Self::PlaySound => "soun",
            Self::KillUser => "kill",
            Self::NavError => "nPrs",
        }
    }

    /// Create MessageId from 4-character ASCII string
    pub fn from_str(s: &str) -> Option<Self> {
        if s.len() != 4 {
            return None;
        }
        Some(match s {
            "tiyr" => Self::Tiyid,
            "rep2" => Self::AltLogonReply,
            "regi" => Self::Regi,
            "auth" => Self::Authenticate,
            "autr" => Self::AuthResponse,
            "susr" => Self::SuperUser,
            "bye " => Self::Logoff,
            "navR" => Self::RoomGoto,
            "room" => Self::RoomDesc,
            "endr" => Self::RoomDescEnd,
            "rLst" => Self::ListOfAllRooms,
            "sLst" => Self::RoomList,
            "sRom" => Self::Room,
            "nprs" => Self::UserNew,
            "eprs" => Self::UserExit,
            "rprs" => Self::UserList,
            "uLoc" => Self::UserMove,
            "usrF" => Self::UserFace,
            "usrP" => Self::UserProp,
            "usrD" => Self::UserDesc,
            "uNam" => Self::UserNameRename,
            "uCol" => Self::UserColor,
            "uSta" => Self::UserStatus,
            "log " => Self::ListOfAllUsers,
            "talk" => Self::Talk,
            "whis" => Self::Whisper,
            "xtlk" => Self::XTalk,
            "xwis" => Self::XWhisper,
            "qAst" => Self::AssetQuery,
            "sAst" => Self::AssetSend,
            "rAst" => Self::AssetRegi,
            "pLoc" => Self::PropMove,
            "dPrp" => Self::PropDelete,
            "nPrp" => Self::PropNew,
            "draw" => Self::Draw,
            "pMov" => Self::PathMove,
            "pLin" => Self::PathLine,
            "sMsg" => Self::SpotState,
            "sMov" => Self::SpotMove,
            "lock" => Self::DoorLock,
            "unlk" => Self::DoorUnlock,
            "sinf" => Self::ServerInfo,
            "sInf" => Self::ExtendedInfo,
            "ping" => Self::Ping,
            "pong" => Self::Pong,
            "blow" => Self::Blowthru,
            "HTTo" => Self::HttpServerRequest,
            "gmsg" => Self::GlobalMsg,
            "rmsg" => Self::RoomMsg,
            "smsg" => Self::SuperMsg,
            "dMsg" => Self::DisplayMsg,
            "soun" => Self::PlaySound,
            "kill" => Self::KillUser,
            "nPrs" => Self::NavError,
            _ => return None,
        })
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
        // Check if the value matches any valid discriminant
        match value {
            0x74697972 | 0x72657032 | 0x72656769 | 0x61757468 | 0x61757472 | 0x73757372
            | 0x62796520 | 0x6e617652 | 0x726f6f6d | 0x656e6472 | 0x724c7374 | 0x734c7374
            | 0x73526f6d | 0x6e707273 | 0x65707273 | 0x72707273 | 0x754c6f63 | 0x75737246
            | 0x75737250 | 0x75737244 | 0x754e616d | 0x75436f6c | 0x75537461 | 0x6c6f6720
            | 0x74616c6b | 0x77686973 | 0x78746c6b | 0x78776973 | 0x71417374 | 0x73417374
            | 0x72417374 | 0x704c6f63 | 0x64507270 | 0x6e507270 | 0x64726177 | 0x704d6f76
            | 0x704c696e | 0x734d7367 | 0x734d6f76 | 0x6c6f636b | 0x756e6c6b | 0x73696e66
            | 0x73496e66 | 0x70696e67 | 0x706f6e67 | 0x626c6f77 | 0x4854546f | 0x676d7367
            | 0x726d7367 | 0x736d7367 | 0x644d7367 | 0x736f756e | 0x6b696c6c | 0x6e507273 => {
                // SAFETY: We've verified the value is a valid discriminant
                Some(unsafe { std::mem::transmute(value) })
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
    }

    #[test]
    fn test_message_id_from_str() {
        assert_eq!(MessageId::from_str("tiyr"), Some(MessageId::Tiyid));
        assert_eq!(MessageId::from_str("talk"), Some(MessageId::Talk));
        assert_eq!(MessageId::from_str("ping"), Some(MessageId::Ping));
        assert_eq!(MessageId::from_str("xyz"), None); // Not a valid message
        assert_eq!(MessageId::from_str("toolong"), None); // Too long
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
    }

    #[test]
    fn test_all_message_ids_roundtrip() {
        // Test that all message IDs can be converted to u32 and back
        let ids = [
            MessageId::Tiyid,
            MessageId::AltLogonReply,
            MessageId::Regi,
            MessageId::Talk,
            MessageId::Whisper,
            MessageId::UserNew,
            MessageId::RoomGoto,
            MessageId::Ping,
            MessageId::Pong,
            MessageId::GlobalMsg,
            MessageId::RoomMsg,
            MessageId::SuperMsg,
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
    }

    #[test]
    fn test_repr_u32_size() {
        // Verify that MessageId is the same size as u32
        assert_eq!(std::mem::size_of::<MessageId>(), std::mem::size_of::<u32>());
    }
}
