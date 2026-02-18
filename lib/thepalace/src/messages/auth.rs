//! Authentication message payloads
//!
//! This module implements the message structures for Palace authentication:
//! - MessageId::Tiyid: First message to detect endianness
//! - MessageId::Logon: Client login with registration info
//! - MessageId::Authenticate: Server authentication challenge
//! - MessageId::AuthResponse: Client authentication response

use bytes::{Buf, BufMut};

use crate::buffer::{BufExt, BufMutExt};
use crate::messages::flags::{
    AuxFlags, DownloadCaps, Engine2DCaps, Engine3DCaps, Graphics2DCaps, UploadCaps,
};
use crate::messages::{MessageId, MessagePayload};
use crate::RoomID;

/// MessageId::Tiyid - First message sent to detect endianness
///
/// This message is sent immediately upon connection. The receiver
/// uses it to determine the sender's byte ordering (endianness).
///
/// The message has no payload - just the 12-byte header with
/// eventType = 0x74697972 ('tiyr')
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TiyidMsg;

impl TiyidMsg {
    /// Create a new TIYID message
    pub fn new() -> Self {
        Self
    }
}

impl Default for TiyidMsg {
    fn default() -> Self {
        Self::new()
    }
}

impl MessagePayload for TiyidMsg {
    fn message_id() -> MessageId {
        MessageId::Tiyid
    }

    fn from_bytes(_buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self)
    }

    fn to_bytes(&self, _buf: &mut impl BufMut) {
        // Empty payload
    }
}

/// Auxiliary registration record containing user session info
///
/// Used in MessageId::Logon and MessageId::AltLogonReply messages
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuxRegistrationRec {
    /// Registration CRC checksum
    pub crc: u32,
    /// Registration counter/number
    pub counter: u32,
    /// User's display name (max 31 chars)
    pub user_name: String,
    /// Wizard password (max 31 chars) - empty for non-wizards
    pub wiz_password: String,
    /// Auxiliary flags (machine type and authentication status)
    pub aux_flags: AuxFlags,
    /// Pseudo-unique user ID counter
    pub puid_ctr: u32,
    /// Pseudo-unique user ID CRC
    pub puid_crc: u32,
    /// Demo elapsed time in seconds
    pub demo_elapsed: u32,
    /// Total elapsed time in seconds
    pub total_elapsed: u32,
    /// Demo time limit in seconds (0 = no limit)
    pub demo_limit: u32,
    /// Desired room ID to enter
    pub desired_room: RoomID,
    /// Client signature (6 bytes) - identifies the Palace client software
    /// Examples: '350211' for ThePalace, 'PC' + 4-byte version for PalaceChat, 'OPNPAL' for OpenPalace
    pub client_signature: [u8; 6],
    /// Requested protocol version
    pub ul_requested_protocol_version: u32,
    /// Upload capabilities flags
    pub ul_upload_caps: UploadCaps,
    /// Download capabilities flags
    pub ul_download_caps: DownloadCaps,
    /// 2D engine capabilities
    pub ul_2d_engine_caps: Engine2DCaps,
    /// 2D graphics capabilities
    pub ul_2d_graphics_caps: Graphics2DCaps,
    /// 3D engine capabilities
    pub ul_3d_engine_caps: Engine3DCaps,
}

impl AuxRegistrationRec {
    /// Size of the serialized structure in bytes
    pub const SIZE: usize = 128;

    /// Create a new registration record for guest login
    pub fn new_guest(user_name: &str, desired_room: RoomID) -> Self {
        Self {
            crc: 0,
            counter: 0,
            user_name: user_name.to_string(),
            wiz_password: String::new(),
            aux_flags: AuxFlags::empty(),
            puid_ctr: 0,
            puid_crc: 0,
            demo_elapsed: 0,
            total_elapsed: 0,
            demo_limit: 0,
            desired_room,
            client_signature: [0; 6],
            ul_requested_protocol_version: 0,
            ul_upload_caps: UploadCaps::empty(),
            ul_download_caps: DownloadCaps::empty(),
            ul_2d_engine_caps: Engine2DCaps::empty(),
            ul_2d_graphics_caps: Graphics2DCaps::empty(),
            ul_3d_engine_caps: Engine3DCaps::empty(),
        }
    }

    /// Create a new registration record for registered user login
    pub fn new_registered(user_name: &str, crc: u32, counter: u32, desired_room: RoomID) -> Self {
        Self {
            crc,
            counter,
            user_name: user_name.to_string(),
            wiz_password: String::new(),
            aux_flags: AuxFlags::empty(),
            puid_ctr: 0,
            puid_crc: 0,
            demo_elapsed: 0,
            total_elapsed: 0,
            demo_limit: 0,
            desired_room,
            client_signature: [0; 6],
            ul_requested_protocol_version: 0,
            ul_upload_caps: UploadCaps::empty(),
            ul_download_caps: DownloadCaps::empty(),
            ul_2d_engine_caps: Engine2DCaps::empty(),
            ul_2d_graphics_caps: Graphics2DCaps::empty(),
            ul_3d_engine_caps: Engine3DCaps::empty(),
        }
    }

    /// Parse from bytes
    pub fn from_bytes<B: Buf>(buf: &mut B) -> Result<Self, std::io::Error> {
        if buf.remaining() < Self::SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "insufficient data for AuxRegistrationRec",
            ));
        }

        let crc = buf.get_u32();
        let counter = buf.get_u32();
        let user_name = buf.get_str31()?;
        let wiz_password = buf.get_str31()?;
        let aux_flags = AuxFlags::from_bits_truncate(buf.get_i32());
        let puid_ctr = buf.get_u32();
        let puid_crc = buf.get_u32();
        let demo_elapsed = buf.get_u32();
        let total_elapsed = buf.get_u32();
        let demo_limit = buf.get_u32();
        let desired_room = buf.get_i16();

        // Read 6-byte client signature
        let mut client_signature = [0u8; 6];
        buf.copy_to_slice(&mut client_signature);

        let ul_requested_protocol_version = buf.get_u32();
        let ul_upload_caps = UploadCaps::from_bits_truncate(buf.get_u32());
        let ul_download_caps = DownloadCaps::from_bits_truncate(buf.get_u32());
        let ul_2d_engine_caps = Engine2DCaps::from_bits_truncate(buf.get_u32());
        let ul_2d_graphics_caps = Graphics2DCaps::from_bits_truncate(buf.get_u32());
        let ul_3d_engine_caps = Engine3DCaps::from_bits_truncate(buf.get_u32());

        Ok(Self {
            crc,
            counter,
            user_name,
            wiz_password,
            aux_flags,
            puid_ctr,
            puid_crc,
            demo_elapsed,
            total_elapsed,
            demo_limit,
            desired_room,
            client_signature,
            ul_requested_protocol_version,
            ul_upload_caps,
            ul_download_caps,
            ul_2d_engine_caps,
            ul_2d_graphics_caps,
            ul_3d_engine_caps,
        })
    }

    /// Write to bytes
    pub fn to_bytes<B: BufMut>(&self, buf: &mut B) {
        buf.put_u32(self.crc);
        buf.put_u32(self.counter);
        buf.put_str31(&self.user_name);
        buf.put_str31(&self.wiz_password);
        buf.put_i32(self.aux_flags.bits());
        buf.put_u32(self.puid_ctr);
        buf.put_u32(self.puid_crc);
        buf.put_u32(self.demo_elapsed);
        buf.put_u32(self.total_elapsed);
        buf.put_u32(self.demo_limit);
        buf.put_i16(self.desired_room);
        // Write 6-byte client signature
        buf.put_slice(&self.client_signature);
        buf.put_u32(self.ul_requested_protocol_version);
        buf.put_u32(self.ul_upload_caps.bits());
        buf.put_u32(self.ul_download_caps.bits());
        buf.put_u32(self.ul_2d_engine_caps.bits());
        buf.put_u32(self.ul_2d_graphics_caps.bits());
        buf.put_u32(self.ul_3d_engine_caps.bits());
    }
}

/// MessageId::Logon - Client login request
///
/// Sent by client to initiate a session with the server.
/// Contains all the registration and capability information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogonMsg {
    /// Registration record with user info
    pub rec: AuxRegistrationRec,
}

impl LogonMsg {
    /// Create a new LOGON message
    pub fn new(rec: AuxRegistrationRec) -> Self {
        Self { rec }
    }

    /// Create a LOGON message for guest login
    pub fn guest(user_name: &str, desired_room: RoomID) -> Self {
        Self {
            rec: AuxRegistrationRec::new_guest(user_name, desired_room),
        }
    }

    /// Create a LOGON message for registered user login
    pub fn registered(user_name: &str, crc: u32, counter: u32, desired_room: RoomID) -> Self {
        Self {
            rec: AuxRegistrationRec::new_registered(user_name, crc, counter, desired_room),
        }
    }

    /// Parse from bytes
    pub fn from_bytes<B: Buf>(buf: &mut B) -> Result<Self, std::io::Error> {
        let rec = AuxRegistrationRec::from_bytes(buf)?;
        Ok(Self { rec })
    }

    /// Write to bytes
    pub fn to_bytes<B: BufMut>(&self, buf: &mut B) {
        self.rec.to_bytes(buf);
    }
}

impl MessagePayload for LogonMsg {
    fn message_id() -> MessageId {
        MessageId::Logon
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        let rec = AuxRegistrationRec::from_bytes(buf)?;
        Ok(Self { rec })
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        self.rec.to_bytes(buf);
    }
}

/// MessageId::AltLogonReply - Alternative logon reply from server
///
/// Sent by server in response to MessageId::Logon when using
/// alternative authentication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AltLogonReplyMsg {
    /// Registration record with updated user info
    pub rec: AuxRegistrationRec,
}

impl AltLogonReplyMsg {
    /// Create a new ALTLOGONREPLY message
    pub fn new(rec: AuxRegistrationRec) -> Self {
        Self { rec }
    }

    /// Parse from bytes
    pub fn from_bytes<B: Buf>(buf: &mut B) -> Result<Self, std::io::Error> {
        let rec = AuxRegistrationRec::from_bytes(buf)?;
        Ok(Self { rec })
    }

    /// Write to bytes
    pub fn to_bytes<B: BufMut>(&self, buf: &mut B) {
        self.rec.to_bytes(buf);
    }
}

impl MessagePayload for AltLogonReplyMsg {
    fn message_id() -> MessageId {
        MessageId::AltLogonReply
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        let rec = AuxRegistrationRec::from_bytes(buf)?;
        Ok(Self { rec })
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        self.rec.to_bytes(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_tiyid_msg() {
        let msg = TiyidMsg::new();
        assert_eq!(msg, TiyidMsg::default());
    }

    #[test]
    fn test_aux_registration_rec_guest() {
        let rec = AuxRegistrationRec::new_guest("TestUser", 100);
        assert_eq!(rec.user_name, "TestUser");
        assert_eq!(rec.desired_room, 100);
        assert_eq!(rec.crc, 0);
        assert_eq!(rec.counter, 0);
    }

    #[test]
    fn test_aux_registration_rec_registered() {
        let rec = AuxRegistrationRec::new_registered("TestUser", 0x12345678, 42, 100);
        assert_eq!(rec.user_name, "TestUser");
        assert_eq!(rec.desired_room, 100);
        assert_eq!(rec.crc, 0x12345678);
        assert_eq!(rec.counter, 42);
    }

    #[test]
    fn test_aux_registration_rec_roundtrip() {
        let rec = AuxRegistrationRec::new_registered("Alice", 0xABCDEF12, 999, 5);

        let mut buf = BytesMut::with_capacity(AuxRegistrationRec::SIZE);
        rec.to_bytes(&mut buf);

        assert_eq!(buf.len(), AuxRegistrationRec::SIZE);

        let mut read_buf = buf.freeze();
        let rec2 = AuxRegistrationRec::from_bytes(&mut read_buf).unwrap();

        assert_eq!(rec, rec2);
    }

    #[test]
    fn test_logon_msg_guest() {
        let msg = LogonMsg::guest("Bob", 42);
        assert_eq!(msg.rec.user_name, "Bob");
        assert_eq!(msg.rec.desired_room, 42);
        assert_eq!(msg.rec.crc, 0);
    }

    #[test]
    fn test_logon_msg_registered() {
        let msg = LogonMsg::registered("Charlie", 0x11223344, 555, 10);
        assert_eq!(msg.rec.user_name, "Charlie");
        assert_eq!(msg.rec.desired_room, 10);
        assert_eq!(msg.rec.crc, 0x11223344);
        assert_eq!(msg.rec.counter, 555);
    }

    #[test]
    fn test_logon_msg_roundtrip() {
        let msg = LogonMsg::guest("TestUser", 1);

        let mut buf = BytesMut::with_capacity(AuxRegistrationRec::SIZE);
        msg.to_bytes(&mut buf);

        let mut read_buf = buf.freeze();
        let msg2 = LogonMsg::from_bytes(&mut read_buf).unwrap();

        assert_eq!(msg, msg2);
    }

    #[test]
    fn test_message_payload_trait() {
        use crate::messages::MessagePayload;

        // Test TiyidMsg
        let tiyid = TiyidMsg::new();
        let message = tiyid.to_message_default();
        assert_eq!(message.msg_id, MessageId::Tiyid);
        assert_eq!(message.payload.len(), 0);

        // Test LogonMsg
        let logon = LogonMsg::guest("Alice", 5);
        let message = logon.to_message(0);
        assert_eq!(message.msg_id, MessageId::Logon);
        assert_eq!(message.payload.len(), AuxRegistrationRec::SIZE);

        // Parse it back
        let parsed = message.parse_payload::<LogonMsg>().unwrap();
        assert_eq!(parsed.rec.user_name, "Alice");
        assert_eq!(parsed.rec.desired_room, 5);
    }
}
