//! Asset message payloads
//!
//! This module implements message structures for asset-related operations:
//! - MessageId::AssetQuery: Request an asset from client or server
//! - MessageId::AssetSend: Send an asset from server to client
//! - MessageId::AssetRegi: Send an asset from client to server (uses AssetSendMsg)
//!
//! Assets can be transmitted in blocks for large files, though the original
//! Palace server only supports single-block transfers.

use bytes::{Buf, BufMut, Bytes};

use crate::buffer::{BufExt, BufMutExt};
use crate::messages::{MessageId, MessagePayload};
use crate::{AssetSpec, AssetType};

/// MessageId::AssetQuery - Request an asset from the receiver
///
/// The server uses this to request props from the client.
/// The client uses this to request arbitrary assets from the server.
///
/// The server only ever requests RT_PROP assets from clients.
/// A CRC value of 0 indicates "don't care" (no verification needed).
///
/// Format:
/// - type: AssetType (4 bytes)
/// - spec: AssetSpec (8 bytes)
#[derive(Debug, Clone, PartialEq)]
pub struct AssetQueryMsg {
    /// Type of asset being requested
    pub asset_type: AssetType,
    /// Asset specification (ID + CRC)
    pub spec: AssetSpec,
}

impl AssetQueryMsg {
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        let type_raw = buf.get_u32();
        let asset_type = AssetType::from_u32(type_raw).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid asset type: 0x{:08X}", type_raw),
            )
        })?;

        Ok(Self {
            asset_type,
            spec: AssetSpec::from_bytes(buf)?,
        })
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_u32(self.asset_type as u32);
        self.spec.to_bytes(buf);
    }
}

impl MessagePayload for AssetQueryMsg {
    fn message_id() -> MessageId {
        MessageId::AssetQuery
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Self::from_bytes(buf)
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        self.to_bytes(buf);
    }
}

/// Asset descriptor - metadata about an asset
///
/// Present only in the first block (blockNbr == 0) of an asset transfer.
///
/// Size: 40 bytes (4 + 4 + 32)
#[derive(Debug, Clone, PartialEq)]
pub struct AssetDescriptor {
    /// Asset flags (client use only)
    pub flags: u32,
    /// Total size of asset in bytes
    pub size: u32,
    /// Asset name (Str31 - fixed 32 bytes)
    pub name: String,
}

impl AssetDescriptor {
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            flags: buf.get_u32(),
            size: buf.get_u32(),
            name: buf.get_str31()?,
        })
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_u32(self.flags);
        buf.put_u32(self.size);
        buf.put_str31(&self.name);
    }
}

/// MessageId::AssetSend / MessageId::AssetRegi - Transmit an asset in blocks
///
/// MessageId::AssetSend / MessageId::AssetRegi - Bidirectional asset transfer
///
/// Used for asset transfer in both directions:
/// - MessageId::AssetSend: Server sends asset to client
/// - MessageId::AssetRegi: Client sends asset to server (register/upload)
///
/// Both message types use the exact same format.
///
/// Assets can be transmitted in multiple blocks, though the original Palace
/// server only supports single-block transfers (one message per asset).
///
/// Format:
/// - type: AssetType (4 bytes)
/// - spec: AssetSpec (8 bytes)
/// - block_size: i32 (4 bytes) - size of this block
/// - block_offset: i32 (4 bytes) - offset from start of asset
/// - block_nbr: i16 (2 bytes) - block number (0-indexed)
/// - nbr_blocks: i16 (2 bytes) - total number of blocks
/// - desc: AssetDescriptor (40 bytes) - only present if block_nbr == 0
/// - data: [u8] (block_size bytes) - actual asset data
#[derive(Debug, Clone, PartialEq)]
pub struct AssetSendMsg {
    /// Type of asset being sent
    pub asset_type: AssetType,
    /// Asset specification (ID + CRC)
    pub spec: AssetSpec,
    /// Size of this block in bytes
    pub block_size: i32,
    /// Offset from start of asset
    pub block_offset: i32,
    /// Block number (0-indexed)
    pub block_nbr: i16,
    /// Total number of blocks
    pub nbr_blocks: i16,
    /// Asset descriptor (only present if block_nbr == 0)
    pub desc: Option<AssetDescriptor>,
    /// Asset data for this block
    pub data: Bytes,
}

impl AssetSendMsg {
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        let type_raw = buf.get_u32();
        let asset_type = AssetType::from_u32(type_raw).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid asset type: 0x{:08X}", type_raw),
            )
        })?;

        let spec = AssetSpec::from_bytes(buf)?;
        let block_size = buf.get_i32();
        let block_offset = buf.get_i32();
        let block_nbr = buf.get_i16();
        let nbr_blocks = buf.get_i16();

        // AssetDescriptor is only present if this is the first block
        let desc = if block_nbr == 0 {
            Some(AssetDescriptor::from_bytes(buf)?)
        } else {
            None
        };

        // Read asset data
        let data = if block_size > 0 {
            buf.copy_to_bytes(block_size as usize)
        } else {
            Bytes::new()
        };

        Ok(Self {
            asset_type,
            spec,
            block_size,
            block_offset,
            block_nbr,
            nbr_blocks,
            desc,
            data,
        })
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_u32(self.asset_type as u32);
        self.spec.to_bytes(buf);
        buf.put_i32(self.block_size);
        buf.put_i32(self.block_offset);
        buf.put_i16(self.block_nbr);
        buf.put_i16(self.nbr_blocks);

        // Write descriptor if this is the first block
        if let Some(ref desc) = self.desc {
            desc.to_bytes(buf);
        }

        // Write asset data
        buf.put_slice(&self.data);
    }

    /// Create a single-block asset send message (most common case)
    pub fn single_block(asset_type: AssetType, spec: AssetSpec, name: String, data: Bytes) -> Self {
        let size = data.len() as u32;
        Self {
            asset_type,
            spec,
            block_size: size as i32,
            block_offset: 0,
            block_nbr: 0,
            nbr_blocks: 1,
            desc: Some(AssetDescriptor {
                flags: 0,
                size,
                name,
            }),
            data,
        }
    }
}

impl MessagePayload for AssetSendMsg {
    fn message_id() -> MessageId {
        MessageId::AssetSend
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
    fn test_asset_query_msg_roundtrip() {
        let msg = AssetQueryMsg {
            asset_type: AssetType::Prop,
            spec: AssetSpec {
                id: 12345,
                crc: 0xABCDEF01,
            },
        };

        let mut buf = BytesMut::new();
        msg.to_bytes(&mut buf);

        assert_eq!(buf.len(), 4 + 10); // AssetType + AssetSpec (with 2-byte padding)

        let mut reader = buf.freeze();
        let parsed = AssetQueryMsg::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_asset_descriptor_roundtrip() {
        let desc = AssetDescriptor {
            flags: 0x00100000, // 20-bit format
            size: 1024,
            name: "Test Asset".to_string(),
        };

        let mut buf = BytesMut::new();
        desc.to_bytes(&mut buf);

        assert_eq!(buf.len(), 40); // 4 + 4 + 32 (Str31)

        let mut reader = buf.freeze();
        let parsed = AssetDescriptor::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, desc);
    }

    #[test]
    fn test_asset_send_msg_single_block() {
        let data = Bytes::from_static(b"Hello, Palace!");

        let msg = AssetSendMsg::single_block(
            AssetType::Prop,
            AssetSpec {
                id: 100,
                crc: 0x12345678,
            },
            "test.prop".to_string(),
            data.clone(),
        );

        assert_eq!(msg.block_nbr, 0);
        assert_eq!(msg.nbr_blocks, 1);
        assert_eq!(msg.block_size, data.len() as i32);
        assert!(msg.desc.is_some());

        let mut buf = BytesMut::new();
        msg.to_bytes(&mut buf);

        // 4 (type) + 10 (spec with padding) + 4 (block_size) + 4 (block_offset) + 2 (block_nbr) + 2 (nbr_blocks) + 40 (desc) + data.len()
        let expected_size = 4 + 10 + 4 + 4 + 2 + 2 + 40 + data.len();
        assert_eq!(buf.len(), expected_size);

        let mut reader = buf.freeze();
        let parsed = AssetSendMsg::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, msg);
        assert_eq!(parsed.data, data);
    }

    #[test]
    fn test_asset_send_msg_multi_block() {
        // Test second block (no descriptor)
        let data = Bytes::from_static(b"Block 2 data");

        let msg = AssetSendMsg {
            asset_type: AssetType::Prop,
            spec: AssetSpec {
                id: 200,
                crc: 0x87654321,
            },
            block_size: data.len() as i32,
            block_offset: 1024,
            block_nbr: 1, // Second block
            nbr_blocks: 3,
            desc: None, // No descriptor for non-first blocks
            data: data.clone(),
        };

        let mut buf = BytesMut::new();
        msg.to_bytes(&mut buf);

        // 4 (type) + 10 (spec with padding) + 4 (block_size) + 4 (block_offset) + 2 (block_nbr) + 2 (nbr_blocks) + data.len()
        // No descriptor since block_nbr != 0
        let expected_size = 4 + 10 + 4 + 4 + 2 + 2 + data.len();
        assert_eq!(buf.len(), expected_size);

        let mut reader = buf.freeze();
        let parsed = AssetSendMsg::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, msg);
        assert_eq!(parsed.data, data);
        assert!(parsed.desc.is_none());
    }

    #[test]
    fn test_asset_query_msg_payload_trait() {
        let msg = AssetQueryMsg {
            asset_type: AssetType::Prop,
            spec: AssetSpec { id: 1, crc: 0 },
        };

        // Test to_message()
        let message = msg.to_message(0);
        assert_eq!(message.msg_id, MessageId::AssetQuery);
        assert_eq!(message.ref_num, 0);

        // Test parse_payload()
        let parsed = message.parse_payload::<AssetQueryMsg>().unwrap();
        assert_eq!(parsed.asset_type, msg.asset_type);
        assert_eq!(parsed.spec, msg.spec);
    }

    #[test]
    fn test_asset_send_msg_payload_trait() {
        let msg = AssetSendMsg::single_block(
            AssetType::Prop,
            AssetSpec { id: 1, crc: 0 },
            "test".to_string(),
            Bytes::from_static(b"data"),
        );

        // Test to_message()
        let message = msg.to_message(0);
        assert_eq!(message.msg_id, MessageId::AssetSend);

        // Test parse_payload()
        let parsed = message.parse_payload::<AssetSendMsg>().unwrap();
        assert_eq!(parsed.asset_type, msg.asset_type);
        assert_eq!(parsed.data, msg.data);
    }
}
