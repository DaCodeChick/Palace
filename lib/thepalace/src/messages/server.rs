//! Server message payloads
//!
//! This module implements message structures for server-related operations:
//! - MessageId::Ping: Keepalive ping from server/client
//! - MessageId::Pong: Keepalive pong response
//! - MessageId::ServerInfo: Server configuration and capabilities
//! - MessageId::ExtendedInfo: Extended server information
//! - MessageId::UserList: List of users in a room
//! - MessageId::ListOfAllUsers: Complete list of all users on server
//! - MessageId::UserLog: Notification that a user logged on

use bytes::{Buf, BufMut};

use crate::buffer::{BufExt, BufMutExt};
use crate::messages::flags::{DownloadCaps, ServerFlags, UploadCaps};
use crate::messages::{MessageId, MessagePayload};

use super::user::UserRec;

/// MessageId::Ping - Keepalive ping message
///
/// Empty payload. The refNum field in the message header can carry
/// an arbitrary value that will be echoed in the MessageId::Pong response.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PingMsg;

impl MessagePayload for PingMsg {
    fn message_id() -> MessageId {
        MessageId::Ping
    }

    fn from_bytes(_buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self)
    }

    fn to_bytes(&self, _buf: &mut impl BufMut) {}
}

/// MessageId::Pong - Keepalive pong response
///
/// Empty payload. The refNum field in the message header should echo
/// the refNum from the corresponding MessageId::Ping message.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PongMsg;

impl MessagePayload for PongMsg {
    fn message_id() -> MessageId {
        MessageId::Pong
    }

    fn from_bytes(_buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self)
    }

    fn to_bytes(&self, _buf: &mut impl BufMut) {}
}

/// MessageId::ServerInfo - Server configuration and capabilities
///
/// Sent by server to client during logon to describe server characteristics.
/// Size: 104 bytes (4 + 64 + 4 + 4 + 4 + variable padding)
#[derive(Debug, Clone, PartialEq)]
pub struct ServerInfoMsg {
    /// Server permission flags (what's allowed on this server)
    pub server_permissions: ServerFlags,
    /// Server name (Str63 = 64 bytes fixed)
    pub server_name: String,
    /// Server option flags (configuration settings)
    pub server_options: u32,
    /// Upload capabilities
    pub upload_caps: UploadCaps,
    /// Download capabilities
    pub download_caps: DownloadCaps,
}

impl ServerInfoMsg {
    /// Create a new ServerInfoMsg
    pub fn new(
        server_permissions: ServerFlags,
        server_name: impl Into<String>,
        server_options: u32,
        upload_caps: UploadCaps,
        download_caps: DownloadCaps,
    ) -> Self {
        Self {
            server_permissions,
            server_name: server_name.into(),
            server_options,
            upload_caps,
            download_caps,
        }
    }
}

impl MessagePayload for ServerInfoMsg {
    fn message_id() -> MessageId {
        MessageId::ServerInfo
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        let server_permissions = ServerFlags::from_bits_truncate(buf.get_u32());
        let server_name = buf.get_str63()?;
        let server_options = buf.get_u32();
        let upload_caps = UploadCaps::from_bits_truncate(buf.get_u32());
        let download_caps = DownloadCaps::from_bits_truncate(buf.get_u32());

        Ok(Self {
            server_permissions,
            server_name,
            server_options,
            upload_caps,
            download_caps,
        })
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_u32(self.server_permissions.bits());
        buf.put_str63(&self.server_name);
        buf.put_u32(self.server_options);
        buf.put_u32(self.upload_caps.bits());
        buf.put_u32(self.download_caps.bits());
    }
}

/// MessageId::UserList - List of users in current room
///
/// Sent from server to client as part of room entry process.
/// The refNum field contains the number of users in the room.
#[derive(Debug, Clone, PartialEq)]
pub struct UserListMsg {
    /// Array of users in the room
    pub users: Vec<UserRec>,
}

impl UserListMsg {
    /// Create a new UserListMsg
    pub fn new(users: Vec<UserRec>) -> Self {
        Self { users }
    }

    /// Number of users
    pub const fn count(&self) -> usize {
        self.users.len()
    }
}

impl MessagePayload for UserListMsg {
    fn message_id() -> MessageId {
        MessageId::UserList
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        let mut users = Vec::new();
        while buf.remaining() >= UserRec::SIZE {
            users.push(UserRec::from_bytes(buf)?);
        }
        Ok(Self { users })
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        for user in &self.users {
            user.to_bytes(buf);
        }
    }
}

/// MessageId::ListOfAllUsers - Complete list of all users on server
///
/// Same format as UserListMsg but contains all users across all rooms.
#[derive(Debug, Clone, PartialEq)]
pub struct ListOfAllUsersMsg {
    /// Array of all users on server
    pub users: Vec<UserRec>,
}

impl ListOfAllUsersMsg {
    /// Create a new ListOfAllUsersMsg
    pub fn new(users: Vec<UserRec>) -> Self {
        Self { users }
    }

    /// Number of users
    pub const fn count(&self) -> usize {
        self.users.len()
    }
}

impl MessagePayload for ListOfAllUsersMsg {
    fn message_id() -> MessageId {
        MessageId::ListOfAllUsers
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        let mut users = Vec::new();
        while buf.remaining() >= UserRec::SIZE {
            users.push(UserRec::from_bytes(buf)?);
        }
        Ok(Self { users })
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        for user in &self.users {
            user.to_bytes(buf);
        }
    }
}

/// MessageId::UserLog - User logged onto server notification
///
/// Sent from server to clients when a new user logs onto the server.
/// The refNum field contains the UserID of the user who logged on.
#[derive(Debug, Clone, PartialEq)]
pub struct UserLogMsg {
    /// Revised number of users on the server
    pub nbr_users: i32,
}

impl UserLogMsg {
    /// Create a new UserLogMsg
    pub const fn new(nbr_users: i32) -> Self {
        Self { nbr_users }
    }
}

impl MessagePayload for UserLogMsg {
    fn message_id() -> MessageId {
        MessageId::UserLog
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            nbr_users: buf.get_i32(),
        })
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i32(self.nbr_users);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_msg() {
        let ping = PingMsg;
        let mut buf = vec![];
        ping.to_bytes(&mut buf);
        assert_eq!(buf.len(), 0);

        let parsed = PingMsg::from_bytes(&mut &buf[..]).unwrap();
        assert_eq!(parsed, ping);
    }

    #[test]
    fn test_pong_msg() {
        let pong = PongMsg;
        let mut buf = vec![];
        pong.to_bytes(&mut buf);
        assert_eq!(buf.len(), 0);

        let parsed = PongMsg::from_bytes(&mut &buf[..]).unwrap();
        assert_eq!(parsed, pong);
    }

    #[test]
    fn test_server_info_msg() {
        let server_info = ServerInfoMsg::new(
            ServerFlags::DIRECT_PLAY | ServerFlags::ALLOW_CYBORGS,
            "Test Palace Server",
            0x00000002, // Password security
            UploadCaps::FILES_PALACE | UploadCaps::ASSETS_PALACE,
            DownloadCaps::FILES_PALACE | DownloadCaps::ASSETS_PALACE,
        );

        let mut buf = vec![];
        server_info.to_bytes(&mut buf);
        assert_eq!(buf.len(), 80); // 4 + 64 (Str63) + 4 + 4 + 4

        let parsed = ServerInfoMsg::from_bytes(&mut &buf[..]).unwrap();
        assert_eq!(parsed.server_name, "Test Palace Server");
        assert_eq!(
            parsed.server_permissions,
            ServerFlags::DIRECT_PLAY | ServerFlags::ALLOW_CYBORGS
        );
        assert_eq!(parsed.server_options, 0x00000002);
        assert_eq!(
            parsed.upload_caps,
            UploadCaps::FILES_PALACE | UploadCaps::ASSETS_PALACE
        );
        assert_eq!(
            parsed.download_caps,
            DownloadCaps::FILES_PALACE | DownloadCaps::ASSETS_PALACE
        );
    }

    #[test]
    fn test_user_log_msg() {
        let user_log = UserLogMsg::new(42);

        let mut buf = vec![];
        user_log.to_bytes(&mut buf);
        assert_eq!(buf.len(), 4);

        let parsed = UserLogMsg::from_bytes(&mut &buf[..]).unwrap();
        assert_eq!(parsed.nbr_users, 42);
    }

    #[test]
    fn test_user_list_msg_empty() {
        let user_list = UserListMsg::new(vec![]);

        let mut buf = vec![];
        user_list.to_bytes(&mut buf);
        assert_eq!(buf.len(), 0);
        assert_eq!(user_list.count(), 0);

        let parsed = UserListMsg::from_bytes(&mut &buf[..]).unwrap();
        assert_eq!(parsed.users.len(), 0);
    }

    #[test]
    fn test_list_of_all_users_msg_empty() {
        let all_users = ListOfAllUsersMsg::new(vec![]);

        let mut buf = vec![];
        all_users.to_bytes(&mut buf);
        assert_eq!(buf.len(), 0);
        assert_eq!(all_users.count(), 0);

        let parsed = ListOfAllUsersMsg::from_bytes(&mut &buf[..]).unwrap();
        assert_eq!(parsed.users.len(), 0);
    }
}
