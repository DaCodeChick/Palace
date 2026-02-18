//! Chat message payloads
//!
//! This module implements message structures for chat operations:
//! - MessageId::Talk: Normal chat message to all users in room
//! - MessageId::XTalk: Encrypted chat message to all users in room
//! - MessageId::Whisper: Private chat message to a specific user
//! - MessageId::XWhisper: Encrypted private chat message to a specific user
//! - MessageId::Gmsg: Global message (server-wide)
//! - MessageId::Rmsg: Room message (flagged for superusers)
//! - MessageId::Smsg: Superuser message

use bytes::{Buf, BufMut};

use crate::buffer::{BufExt, BufMutExt};
use crate::messages::{MessageId, MessagePayload};
use crate::UserID;

/// MessageId::Talk - Normal chat message
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

impl MessagePayload for TalkMsg {
    fn message_id() -> MessageId {
        MessageId::Talk
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            text: buf.get_cstring()?,
        })
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_cstring(&self.text);
    }

    fn default_ref_num(&self) -> i32 {
        0 // Can be overridden with actual UserID when creating message
    }
}

/// MessageId::XTalk - Encrypted chat message
///
/// Similar to MessageId::Talk but text is encrypted to prevent sniffing.
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

impl MessagePayload for XTalkMsg {
    fn message_id() -> MessageId {
        MessageId::XTalk
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Self::from_bytes(buf)
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        self.to_bytes(buf);
    }

    fn default_ref_num(&self) -> i32 {
        0 // Can be overridden with actual UserID when creating message
    }
}

/// MessageId::Whisper - Private chat message (request form)
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

impl MessagePayload for WhisperMsg {
    fn message_id() -> MessageId {
        MessageId::Whisper
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Self::from_bytes(buf)
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        self.to_bytes(buf);
    }
}

/// MessageId::XWhisper - Encrypted private chat message (request form)
///
/// Similar to MessageId::Whisper but with encrypted text.
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

impl MessagePayload for XWhisperMsg {
    fn message_id() -> MessageId {
        MessageId::XWhisper
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Self::from_bytes(buf)
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        self.to_bytes(buf);
    }
}

/// MessageId::Gmsg - Global message
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

impl MessagePayload for GmsgMsg {
    fn message_id() -> MessageId {
        MessageId::Gmsg
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Self::from_bytes(buf)
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        self.to_bytes(buf);
    }
}

/// MessageId::Rmsg - Room message
///
/// Similar to MessageId::Talk but flagged for superuser attention.
/// Sent to room users as MessageId::Talk, plus special MessageId::Talk to superusers.
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

impl MessagePayload for RmsgMsg {
    fn message_id() -> MessageId {
        MessageId::Rmsg
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Self::from_bytes(buf)
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        self.to_bytes(buf);
    }
}

/// MessageId::Smsg - Superuser message
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

impl MessagePayload for SmsgMsg {
    fn message_id() -> MessageId {
        MessageId::Smsg
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Self::from_bytes(buf)
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        self.to_bytes(buf);
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
    fn test_talk_msg_payload_trait() {
        let msg = TalkMsg {
            text: "Hello, Palace!".to_string(),
        };

        // Test to_message()
        let message = msg.to_message(12345);
        assert_eq!(message.msg_id, MessageId::Talk);
        assert_eq!(message.ref_num, 12345);

        // Test parse_payload()
        let parsed = message.parse_payload::<TalkMsg>().unwrap();
        assert_eq!(parsed.text, msg.text);
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
