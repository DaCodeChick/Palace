//! Palace Protocol message structures.
//!
//! The Palace Protocol uses a common header format for all messages:
//! - 4 bytes: Event type (MessageId as big-endian u32)
//! - 4 bytes: Message length (big-endian u32, payload size excluding header)
//! - 4 bytes: Reference number (big-endian i32, arbitrary parameter)
//! - Variable: Message payload
//!
//! Total header size: 12 bytes
//!
//! # MessagePayload Trait
//!
//! All message payload types should implement the `MessagePayload` trait to provide
//! consistent serialization and deserialization. This allows for type-safe message
//! handling throughout the codebase.
//!
//! ## Example
//!
//! ```ignore
//! use thepalace::messages::{MessageId, MessagePayload, TalkMsg};
//!
//! // Create a chat message
//! let talk = TalkMsg { text: "Hello, Palace!".to_string() };
//!
//! // Convert to a complete Message with a specific UserID
//! let message = talk.to_message(12345);
//! assert_eq!(message.msg_id, MessageId::TALK);
//! assert_eq!(message.ref_num, 12345); // UserID of speaker
//!
//! // Serialize to bytes for transmission
//! let bytes = message.to_bytes();
//!
//! // Parse from bytes
//! let parsed_message = Message::parse(&mut bytes.as_slice()).unwrap();
//!
//! // Extract the typed payload
//! let parsed_talk = parsed_message.parse_payload::<TalkMsg>().unwrap();
//! assert_eq!(parsed_talk.text, "Hello, Palace!");
//! ```
//!
//! ## Implementing MessagePayload
//!
//! To implement the trait for a new message type:
//!
//! ```ignore
//! use bytes::{Buf, BufMut};
//! use thepalace::messages::{MessageId, MessagePayload};
//!
//! #[derive(Debug, Clone, PartialEq)]
//! pub struct MyMsg {
//!     pub value: i32,
//! }
//!
//! impl MessagePayload for MyMsg {
//!     fn message_id() -> MessageId {
//!         MessageId::MYMSG
//!     }
//!
//!     fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
//!         Ok(Self {
//!             value: buf.get_i32(),
//!         })
//!     }
//!
//!     fn to_bytes(&self, buf: &mut impl BufMut) {
//!         buf.put_i32(self.value);
//!     }
//! }
//! ```

use super::MessageId;
use bytes::{Buf, BufMut, BytesMut};
use std::io;

/// Trait for Palace Protocol message payloads.
///
/// All message payload types should implement this trait to provide
/// consistent serialization and deserialization.
pub trait MessagePayload: Sized {
    /// The MessageId for this payload type
    fn message_id() -> MessageId;

    /// Parse the payload from bytes
    fn from_bytes(buf: &mut impl Buf) -> io::Result<Self>;

    /// Serialize the payload to bytes
    fn to_bytes(&self, buf: &mut impl BufMut);

    /// Get the default ref_num value for this message type (usually 0)
    fn default_ref_num(&self) -> i32 {
        0
    }

    /// Convenience method to create a complete Message from this payload
    fn to_message(&self, ref_num: i32) -> Message {
        let mut payload = BytesMut::new();
        self.to_bytes(&mut payload);
        Message::new(Self::message_id(), ref_num, payload.to_vec())
    }

    /// Convenience method to create a Message with default ref_num
    fn to_message_default(&self) -> Message {
        self.to_message(self.default_ref_num())
    }
}

/// Generic Palace Protocol message structure.
///
/// All Palace messages share this common structure with a 12-byte header
/// followed by message-specific payload data.
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    /// Message type identifier
    pub msg_id: MessageId,
    /// Reference number (arbitrary parameter, usage varies by message type)
    pub ref_num: i32,
    /// Message payload data
    pub payload: Vec<u8>,
}

impl Message {
    /// Header size in bytes (event_type + length + ref_num)
    pub const HEADER_SIZE: usize = 12;

    /// Create a new message
    pub fn new(msg_id: MessageId, ref_num: i32, payload: Vec<u8>) -> Self {
        Self {
            msg_id,
            ref_num,
            payload,
        }
    }

    /// Create a message with empty payload
    pub fn new_empty(msg_id: MessageId, ref_num: i32) -> Self {
        Self::new(msg_id, ref_num, Vec::new())
    }

    /// Create a message from a payload that implements MessagePayload
    pub fn from_payload<P: MessagePayload>(payload: &P, ref_num: i32) -> Self {
        payload.to_message(ref_num)
    }

    /// Parse the payload as a specific type
    pub fn parse_payload<P: MessagePayload>(&self) -> io::Result<P> {
        let mut buf = &self.payload[..];
        P::from_bytes(&mut buf)
    }

    /// Get the total message size (header + payload)
    pub fn total_size(&self) -> usize {
        Self::HEADER_SIZE + self.payload.len()
    }

    /// Get the payload size
    pub fn payload_size(&self) -> usize {
        self.payload.len()
    }

    /// Parse a message from a buffer.
    ///
    /// Reads the 12-byte header and then the payload based on the length field.
    ///
    /// # Errors
    ///
    /// Returns an error if there aren't enough bytes or if the message is malformed.
    pub fn parse<B: Buf>(buf: &mut B) -> io::Result<Self> {
        if buf.remaining() < Self::HEADER_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!(
                    "need {} bytes for header, got {}",
                    Self::HEADER_SIZE,
                    buf.remaining()
                ),
            ));
        }

        // Read header
        let msg_id_u32 = buf.get_u32();
        let msg_id = MessageId::from_u32(msg_id_u32).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown message ID: 0x{:08x}", msg_id_u32),
            )
        })?;
        let length = buf.get_u32() as usize;
        let ref_num = buf.get_i32();

        // Check if we have enough bytes for payload
        if buf.remaining() < length {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!("need {} bytes for payload, got {}", length, buf.remaining()),
            ));
        }

        // Read payload
        let mut payload = vec![0u8; length];
        buf.copy_to_slice(&mut payload);

        Ok(Self {
            msg_id,
            ref_num,
            payload,
        })
    }

    /// Serialize the message to a buffer.
    ///
    /// Writes the 12-byte header followed by the payload.
    pub fn serialize<B: BufMut>(&self, buf: &mut B) {
        // Write header
        buf.put_u32(self.msg_id.as_u32());
        buf.put_u32(self.payload.len() as u32);
        buf.put_i32(self.ref_num);

        // Write payload
        buf.put_slice(&self.payload);
    }

    /// Convert the message to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.total_size());
        self.serialize(&mut buf);
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{Bytes, BytesMut};

    #[test]
    fn test_message_new() {
        let msg = Message::new(MessageId::Ping, 123, vec![1, 2, 3, 4]);
        assert_eq!(msg.msg_id, MessageId::Ping);
        assert_eq!(msg.ref_num, 123);
        assert_eq!(msg.payload, vec![1, 2, 3, 4]);
        assert_eq!(msg.payload_size(), 4);
        assert_eq!(msg.total_size(), 16); // 12 header + 4 payload
    }

    #[test]
    fn test_message_empty() {
        let msg = Message::new_empty(MessageId::Pong, 456);
        assert_eq!(msg.msg_id, MessageId::Pong);
        assert_eq!(msg.ref_num, 456);
        assert_eq!(msg.payload.len(), 0);
        assert_eq!(msg.total_size(), 12); // Just header
    }

    #[test]
    fn test_message_roundtrip() {
        let original = Message::new(MessageId::Talk, 789, vec![0xDE, 0xAD, 0xBE, 0xEF]);

        // Serialize
        let bytes = original.to_bytes();
        assert_eq!(bytes.len(), 16); // 12 + 4

        // Parse
        let mut buf = Bytes::from(bytes);
        let parsed = Message::parse(&mut buf).unwrap();

        assert_eq!(parsed.msg_id, original.msg_id);
        assert_eq!(parsed.ref_num, original.ref_num);
        assert_eq!(parsed.payload, original.payload);
    }

    #[test]
    fn test_message_parse_insufficient_header() {
        let bytes = vec![0u8; 8]; // Only 8 bytes, need 12
        let mut buf = Bytes::from(bytes);
        let result = Message::parse(&mut buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_message_parse_insufficient_payload() {
        let mut buf = BytesMut::new();
        buf.put_u32(MessageId::Ping.as_u32());
        buf.put_u32(100); // Claims 100 bytes payload
        buf.put_i32(0);
        buf.put_slice(&[1, 2, 3]); // Only 3 bytes

        let mut reader = buf.freeze();
        let result = Message::parse(&mut reader);
        assert!(result.is_err());
    }

    #[test]
    fn test_message_serialize() {
        let msg = Message::new(MessageId::Whisper, 42, vec![0xAA, 0xBB]);

        let mut buf = BytesMut::new();
        msg.serialize(&mut buf);

        let bytes = buf.freeze();
        assert_eq!(bytes.len(), 14); // 12 header + 2 payload

        // Verify header
        assert_eq!(&bytes[0..4], &MessageId::Whisper.as_u32().to_be_bytes());
        assert_eq!(&bytes[4..8], &2u32.to_be_bytes()); // payload length
        assert_eq!(&bytes[8..12], &42i32.to_be_bytes()); // ref_num
        assert_eq!(&bytes[12..14], &[0xAA, 0xBB]); // payload
    }
}
