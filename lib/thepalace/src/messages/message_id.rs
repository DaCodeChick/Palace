//! Palace Protocol message type identifiers.
//!
//! Message types are 4-byte ASCII codes stored as big-endian u32 values.
//! For example, 'tiyr' = 0x74697972.

use std::fmt;

/// Palace Protocol message type identifier (4-character ASCII code).
///
/// Message types are represented as big-endian u32 values where each byte
/// corresponds to an ASCII character. For example, MSG_TIYID ('tiyr') is
/// stored as 0x74697972.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct MessageId(pub u32);

impl MessageId {
    // Connection & Authentication
    /// Client version identification ('tiyr' = 0x74697972)
    pub const TIYID: Self = Self(0x74697972);
    /// Alternative logon reply ('rep2' = 0x72657032)
    pub const ALTLOGONREPLY: Self = Self(0x72657032);
    /// User logon ('regi' = 0x72656769)
    pub const REGI: Self = Self(0x72656769);
    /// Authentication request ('auth' = 0x61757468)
    pub const AUTHENTICATE: Self = Self(0x61757468);
    /// Authentication response ('autr' = 0x61757472)
    pub const AUTHRESPONSE: Self = Self(0x61757472);
    /// Superuser (wizard) elevation ('susr' = 0x73757372)
    pub const SUPERUSER: Self = Self(0x73757372);
    /// Logoff/disconnect ('bye ' = 0x62796520)
    pub const LOGOFF: Self = Self(0x62796520);

    // Rooms
    /// Navigate to room ('navR' = 0x6e617652)
    pub const ROOMGOTO: Self = Self(0x6e617652);
    /// Room description ('room' = 0x726f6f6d)
    pub const ROOMDESC: Self = Self(0x726f6f6d);
    /// End of room description ('endr' = 0x656e6472)
    pub const ROOMDESCEND: Self = Self(0x656e6472);
    /// List of all rooms ('rLst' = 0x724c7374)
    pub const LISTOFALLROOMS: Self = Self(0x724c7374);
    /// Room list ('sLst' = 0x734c7374)
    pub const ROOMLIST: Self = Self(0x734c7374);
    /// Room description (alternative) ('sRom' = 0x73526f6d)
    pub const ROOM: Self = Self(0x73526f6d);

    // Users
    /// New user entered room ('nprs' = 0x6e707273)
    pub const USERNEW: Self = Self(0x6e707273);
    /// User exited room ('eprs' = 0x65707273)
    pub const USEREXIT: Self = Self(0x65707273);
    /// List of users in room ('rprs' = 0x72707273)
    pub const USERLIST: Self = Self(0x72707273);
    /// User moved ('uLoc' = 0x754c6f63)
    pub const USERMOVE: Self = Self(0x754c6f63);
    /// User face changed ('usrF' = 0x75737246)
    pub const USERFACE: Self = Self(0x75737246);
    /// User props changed ('usrP' = 0x75737250)
    pub const USERPROP: Self = Self(0x75737250);
    /// User description ('usrD' = 0x75737244)
    pub const USERDESC: Self = Self(0x75737244);
    /// User renamed ('uNam' = 0x754e616d)
    pub const USERNAMERENAME: Self = Self(0x754e616d);
    /// User color changed ('uCol' = 0x75436f6c)
    pub const USERCOLOR: Self = Self(0x75436f6c);
    /// User status ('uSta' = 0x75537461)
    pub const USERSTATUS: Self = Self(0x75537461);
    /// List of all users ('log ' = 0x6c6f6720)
    pub const LISTOFALLUSERS: Self = Self(0x6c6f6720);

    // Chat
    /// Normal chat message ('talk' = 0x74616c6b)
    pub const TALK: Self = Self(0x74616c6b);
    /// Private message/whisper ('whis' = 0x77686973)
    pub const WHISPER: Self = Self(0x77686973);
    /// Extended talk with author info ('xtlk' = 0x78746c6b)
    pub const XTALK: Self = Self(0x78746c6b);
    /// Extended whisper ('xwis' = 0x78776973)
    pub const XWHISPER: Self = Self(0x78776973);

    // Assets
    /// Query for asset ('qAst' = 0x71417374)
    pub const ASSETQUERY: Self = Self(0x71417374);
    /// Send asset data ('sAst' = 0x73417374)
    pub const ASSETSEND: Self = Self(0x73417374);
    /// Register new asset ('rAst' = 0x72417374)
    pub const ASSETREGI: Self = Self(0x72417374);

    // Props
    /// Prop move ('pLoc' = 0x704c6f63)
    pub const PROPMOVE: Self = Self(0x704c6f63);
    /// Prop delete ('dPrp' = 0x64507270)
    pub const PROPDELETE: Self = Self(0x64507270);
    /// Prop new ('nPrp' = 0x6e507270)
    pub const PROPNEW: Self = Self(0x6e507270);

    // Drawing
    /// Draw command ('draw' = 0x64726177)
    pub const DRAW: Self = Self(0x64726177);
    /// Path move ('pMov' = 0x704d6f76)
    pub const PATHMOVE: Self = Self(0x704d6f76);
    /// Path line ('pLin' = 0x704c696e)
    pub const PATHLINE: Self = Self(0x704c696e);

    // Hotspots
    /// Spot state changed ('sMsg' = 0x734d7367)
    pub const SPOTSTATE: Self = Self(0x734d7367);
    /// Spot move ('sMov' = 0x734d6f76)
    pub const SPOTMOVE: Self = Self(0x734d6f76);

    // Door Operations
    /// Lock door ('lock' = 0x6c6f636b)
    pub const DOORLOCK: Self = Self(0x6c6f636b);
    /// Unlock door ('unlk' = 0x756e6c6b)
    pub const DOORUNLOCK: Self = Self(0x756e6c6b);

    // Server Info
    /// Server information ('sinf' = 0x73696e66)
    pub const SERVERINFO: Self = Self(0x73696e66);
    /// Extended server info request ('sInf' = 0x73496e66)
    pub const EXTENDEDINFO: Self = Self(0x73496e66);

    // Connectivity
    /// Keepalive ping ('ping' = 0x70696e67)
    pub const PING: Self = Self(0x70696e67);
    /// Keepalive pong ('pong' = 0x706f6e67)
    pub const PONG: Self = Self(0x706f6e67);
    /// Blowthru (plugin relay) ('blow' = 0x626c6f77)
    pub const BLOWTHRU: Self = Self(0x626c6f77);

    // Server Commands
    /// HTTP server location ('HTTo' = 0x4854546f)
    pub const HTTPSERVERREQUEST: Self = Self(0x4854546f);
    /// Global message ('gmsg' = 0x676d7367)
    pub const GLOBALMSG: Self = Self(0x676d7367);
    /// Display message ('dMsg' = 0x644d7367)
    pub const DISPLAYMSG: Self = Self(0x644d7367);

    // Media
    /// Play sound ('soun' = 0x736f756e)
    pub const PLAYSOUND: Self = Self(0x736f756e);

    // User Management
    /// Kill user connection ('kill' = 0x6b696c6c)
    pub const KILLUSER: Self = Self(0x6b696c6c);

    // Navigation
    /// Navigation reply ('nPrs' = 0x6e507273)
    pub const NAVERROR: Self = Self(0x6e507273);

    /// Convert MessageId to its 4-character ASCII representation
    pub fn as_str(&self) -> String {
        let bytes = self.0.to_be_bytes();
        String::from_utf8_lossy(&bytes).to_string()
    }

    /// Create MessageId from 4-character ASCII string
    pub fn from_str(s: &str) -> Option<Self> {
        if s.len() != 4 {
            return None;
        }
        let bytes = s.as_bytes();
        let value = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        Some(Self(value))
    }

    /// Get the raw u32 value
    pub const fn as_u32(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<u32> for MessageId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<MessageId> for u32 {
    fn from(msg: MessageId) -> u32 {
        msg.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_id_display() {
        assert_eq!(MessageId::TIYID.as_str(), "tiyr");
        assert_eq!(MessageId::TALK.as_str(), "talk");
        assert_eq!(MessageId::PING.as_str(), "ping");
    }

    #[test]
    fn test_message_id_from_str() {
        assert_eq!(MessageId::from_str("tiyr"), Some(MessageId::TIYID));
        assert_eq!(MessageId::from_str("talk"), Some(MessageId::TALK));
        assert_eq!(MessageId::from_str("ping"), Some(MessageId::PING));
        assert_eq!(MessageId::from_str("xyz"), None); // Too short
        assert_eq!(MessageId::from_str("toolong"), None); // Too long
    }

    #[test]
    fn test_message_id_conversions() {
        let msg = MessageId::TIYID;
        let raw: u32 = msg.into();
        assert_eq!(raw, 0x74697972);

        let msg2: MessageId = raw.into();
        assert_eq!(msg2, msg);
    }
}
