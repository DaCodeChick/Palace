//! Chat message payloads
//!
//! This module implements message structures for chat operations:
//! - MSG_TALK: Normal chat message to all users in room
//! - MSG_XTALK: Encrypted chat message to all users in room
//! - MSG_WHISPER: Private chat message to a specific user
//! - MSG_XWHISPER: Encrypted private chat message to a specific user
//! - MSG_GMSG: Global message (server-wide)
//! - MSG_RMSG: Room message (flagged for superusers)
//! - MSG_SMSG: Superuser message

use bytes::{Buf, BufMut};

use crate::buffer::{BufExt, BufMutExt};
use crate::UserID;

/// MSG_TALK - Normal chat message
///
/// Sent bidirectionally for word balloon speech.
/// The UserID of the speaker is in the message header's refNum field.
///
/// Text is limited to 255 characters.
#[derive(Debug, Clone, PartialEq)]
pub struct TalkMsg {
    pub text: String,
}

impl TalkMsg {
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            text: buf.get_cstring()?,
        })
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_cstring(&self.text);
    }
}

/// MSG_XTALK - Encrypted chat message
///
/// Similar to MSG_TALK but text is encrypted to prevent sniffing.
/// The encryption is a simple XOR cipher.
///
/// Format:
/// - len: i16 (length of encrypted text)
/// - text: [u8; len] (encrypted text bytes)
#[derive(Debug, Clone, PartialEq)]
pub struct XTalkMsg {
    pub text: Vec<u8>,
}

impl XTalkMsg {
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        let len = buf.get_i16() as usize;
        let mut text = vec![0u8; len];
        buf.copy_to_slice(&mut text);

        Ok(Self { text })
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i16(self.text.len() as i16);
        buf.put_slice(&self.text);
    }

    /// Decrypt the text using the Palace XOR cipher
    pub fn decrypt(&self) -> String {
        // TODO: Implement proper decryption using crate::algo::crypt()
        // For now, return a placeholder
        String::from_utf8_lossy(&self.text).to_string()
    }

    /// Encrypt plaintext using the Palace XOR cipher
    pub fn encrypt(plaintext: &str) -> Self {
        // TODO: Implement proper encryption using crate::algo::crypt()
        // For now, just store as-is
        Self {
            text: plaintext.as_bytes().to_vec(),
        }
    }
}

/// MSG_WHISPER - Private chat message (request form)
///
/// Client sends this to request a whisper to a specific user.
/// Server relays it to the target user.
///
/// Format (request):
/// - target: UserID (4 bytes)
/// - text: CString (null-terminated, max 255 chars)
#[derive(Debug, Clone, PartialEq)]
pub struct WhisperMsg {
    pub target: UserID,
    pub text: String,
}

impl WhisperMsg {
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            target: buf.get_i32(),
            text: buf.get_cstring()?,
        })
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i32(self.target);
        buf.put_cstring(&self.text);
    }
}

/// MSG_XWHISPER - Encrypted private chat message (request form)
///
/// Similar to MSG_WHISPER but with encrypted text.
///
/// Format (request):
/// - target: UserID (4 bytes)
/// - len: i16 (length of encrypted text)
/// - text: [u8; len] (encrypted text bytes)
#[derive(Debug, Clone, PartialEq)]
pub struct XWhisperMsg {
    pub target: UserID,
    pub text: Vec<u8>,
}

impl XWhisperMsg {
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        let target = buf.get_i32();
        let len = buf.get_i16() as usize;
        let mut text = vec![0u8; len];
        buf.copy_to_slice(&mut text);

        Ok(Self { target, text })
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i32(self.target);
        buf.put_i16(self.text.len() as i16);
        buf.put_slice(&self.text);
    }

    /// Decrypt the text using the Palace XOR cipher
    pub fn decrypt(&self) -> String {
        // TODO: Implement proper decryption using crate::algo::crypt()
        String::from_utf8_lossy(&self.text).to_string()
    }

    /// Encrypt plaintext using the Palace XOR cipher
    pub fn encrypt(target: UserID, plaintext: &str) -> Self {
        // TODO: Implement proper encryption using crate::algo::crypt()
        Self {
            target,
            text: plaintext.as_bytes().to_vec(),
        }
    }
}

/// MSG_GMSG - Global message
///
/// Sent from server to all connected users regardless of room.
/// Text is a CString, limited to 255 characters.
#[derive(Debug, Clone, PartialEq)]
pub struct GmsgMsg {
    pub text: String,
}

impl GmsgMsg {
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            text: buf.get_cstring()?,
        })
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_cstring(&self.text);
    }
}

/// MSG_RMSG - Room message
///
/// Similar to MSG_TALK but flagged for superuser attention.
/// Sent to room users as MSG_TALK, plus special MSG_TALK to superusers.
///
/// Text is a CString, limited to 255 characters.
#[derive(Debug, Clone, PartialEq)]
pub struct RmsgMsg {
    pub text: String,
}

impl RmsgMsg {
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            text: buf.get_cstring()?,
        })
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_cstring(&self.text);
    }
}

/// MSG_SMSG - Superuser message
///
/// Message sent only to superusers (wizards/gods) in the room.
/// Text is a CString, limited to 255 characters.
#[derive(Debug, Clone, PartialEq)]
pub struct SmsgMsg {
    pub text: String,
}

impl SmsgMsg {
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            text: buf.get_cstring()?,
        })
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_cstring(&self.text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_talk_msg_roundtrip() {
        let msg = TalkMsg {
            text: "Hello, Palace!".to_string(),
        };

        let mut buf = BytesMut::new();
        msg.to_bytes(&mut buf);

        let mut reader = buf.freeze();
        let parsed = TalkMsg::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_xtalk_msg_roundtrip() {
        let msg = XTalkMsg {
            text: vec![0x48, 0x65, 0x6c, 0x6c, 0x6f], // "Hello"
        };

        let mut buf = BytesMut::new();
        msg.to_bytes(&mut buf);

        assert_eq!(buf.len(), 2 + 5); // i16 length + 5 bytes

        let mut reader = buf.freeze();
        let parsed = XTalkMsg::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_xtalk_encrypt() {
        let encrypted = XTalkMsg::encrypt("Test");
        assert_eq!(encrypted.text, b"Test"); // Currently just passes through

        let decrypted = encrypted.decrypt();
        assert_eq!(decrypted, "Test");
    }

    #[test]
    fn test_whisper_msg_roundtrip() {
        let msg = WhisperMsg {
            target: 12345,
            text: "Secret message".to_string(),
        };

        let mut buf = BytesMut::new();
        msg.to_bytes(&mut buf);

        let mut reader = buf.freeze();
        let parsed = WhisperMsg::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_xwhisper_msg_roundtrip() {
        let msg = XWhisperMsg {
            target: 54321,
            text: vec![0x53, 0x65, 0x63, 0x72, 0x65, 0x74], // "Secret"
        };

        let mut buf = BytesMut::new();
        msg.to_bytes(&mut buf);

        assert_eq!(buf.len(), 4 + 2 + 6); // UserID + i16 length + 6 bytes

        let mut reader = buf.freeze();
        let parsed = XWhisperMsg::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_gmsg_msg_roundtrip() {
        let msg = GmsgMsg {
            text: "Server announcement".to_string(),
        };

        let mut buf = BytesMut::new();
        msg.to_bytes(&mut buf);

        let mut reader = buf.freeze();
        let parsed = GmsgMsg::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_rmsg_msg_roundtrip() {
        let msg = RmsgMsg {
            text: "Room announcement".to_string(),
        };

        let mut buf = BytesMut::new();
        msg.to_bytes(&mut buf);

        let mut reader = buf.freeze();
        let parsed = RmsgMsg::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_smsg_msg_roundtrip() {
        let msg = SmsgMsg {
            text: "Wizard only message".to_string(),
        };

        let mut buf = BytesMut::new();
        msg.to_bytes(&mut buf);

        let mut reader = buf.freeze();
        let parsed = SmsgMsg::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, msg);
    }
}
