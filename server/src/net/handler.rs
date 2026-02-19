//! Connection handler for individual client sessions

use anyhow::{Context, Result};
use bytes::{Buf, BytesMut};
use std::net::SocketAddr;
use thepalace::messages::auth::{LogonMsg, TiyidMsg};
use thepalace::messages::chat::{TalkMsg, XTalkMsg, XWhisperMsg};
use thepalace::messages::flags::RoomFlags;
use thepalace::messages::{
    ListOfAllRoomsMsg, Message, MessageId, MessagePayload, RoomDescMsg, RoomGotoMsg, RoomListRec,
    ServerInfoMsg, UserListMsg, UserNewMsg,
};
use thepalace::{AssetSpec, Point};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::state::{RoomId, ServerMessage, ServerState, UserId};

/// Connection handler for a single client
pub struct ConnectionHandler {
    socket: TcpStream,
    addr: SocketAddr,
    state: ServerState,
    user_id: Option<UserId>,
    username: Option<String>,
    current_room: RoomId,
    read_buffer: BytesMut,
    message_rx: mpsc::UnboundedReceiver<ServerMessage>,
    message_tx: mpsc::UnboundedSender<ServerMessage>,
}

impl ConnectionHandler {
    /// Create a new connection handler
    pub fn new(socket: TcpStream, addr: SocketAddr, state: ServerState) -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();

        Self {
            socket,
            addr,
            state,
            user_id: None,
            username: None,
            current_room: 0, // Start in Gate
            read_buffer: BytesMut::with_capacity(8192),
            message_rx,
            message_tx,
        }
    }

    /// Handle the connection (public entry point)
    pub async fn handle(self) -> Result<()> {
        self.run().await
    }

    /// Run the connection handler
    async fn run(mut self) -> Result<()> {
        // Send initial TIYID message for endianness detection
        self.send_tiyid().await?;

        // Main event loop
        loop {
            tokio::select! {
                // Read from socket
                result = self.socket.read_buf(&mut self.read_buffer) => {
                    match result {
                        Ok(0) => {
                            info!("Client {} disconnected", self.addr);
                            break;
                        }
                        Ok(n) => {
                            debug!("Read {} bytes from {}", n, self.addr);
                            self.process_messages().await?;
                        }
                        Err(e) => {
                            error!("Read error from {}: {}", self.addr, e);
                            break;
                        }
                    }
                }

                // Receive broadcast messages
                Some(msg) = self.message_rx.recv() => {
                    self.handle_server_message(msg).await?;
                }
            }
        }

        // Cleanup on disconnect
        if let Some(user_id) = self.user_id {
            self.state.unregister_session(user_id).await;
        }

        Ok(())
    }

    /// Send TIYID message for endianness detection
    async fn send_tiyid(&mut self) -> Result<()> {
        let msg = TiyidMsg::new().to_message_default();
        self.send_message(&msg).await
    }

    /// Process incoming messages from the read buffer
    async fn process_messages(&mut self) -> Result<()> {
        loop {
            // Check if we have enough bytes for a header
            if self.read_buffer.remaining() < Message::HEADER_SIZE {
                break;
            }

            // Try to parse a message (peek without consuming)
            let mut peek_buf = &self.read_buffer[..];
            let message = match Message::parse(&mut peek_buf) {
                Ok(msg) => {
                    // Successfully parsed, now consume from read_buffer
                    let total_size = Message::HEADER_SIZE + msg.payload.len();
                    self.read_buffer.advance(total_size);
                    msg
                }
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // Need more data
                    break;
                }
                Err(e) => {
                    return Err(e).context("Failed to parse message");
                }
            };

            debug!("Received message: {:?}", message.msg_id);
            self.handle_message(message).await?;
        }

        Ok(())
    }

    /// Handle a single incoming message
    async fn handle_message(&mut self, message: Message) -> Result<()> {
        match message.msg_id {
            MessageId::Logon => self.handle_logon(message).await?,
            MessageId::Talk => self.handle_talk(message).await?,
            MessageId::XTalk => self.handle_xtalk(message).await?,
            MessageId::XWhisper => self.handle_whisper(message).await?,
            MessageId::RoomGoto => self.handle_room_goto(message).await?,
            MessageId::ListOfAllRooms => self.handle_list_rooms(message).await?,
            MessageId::Ping => self.handle_ping(message).await?,
            MessageId::Pong => { /* Ignore pong */ }
            _ => {
                warn!("Unhandled message type: {:?}", message.msg_id);
            }
        }

        Ok(())
    }

    /// Handle logon message
    async fn handle_logon(&mut self, message: Message) -> Result<()> {
        let logon = message
            .parse_payload::<LogonMsg>()
            .context("Failed to parse logon message")?;

        let username = logon.rec.user_name.clone();
        info!("User '{}' logging in from {}", username, self.addr);

        // Check if IP is banned
        if self.state.db().is_ip_banned(&self.addr.ip().to_string()).await? {
            warn!("Banned IP attempted to connect: {}", self.addr.ip());
            return Ok(()); // Just close connection
        }

        // Try to find existing user or create new one
        let user = match self.state.db().get_user_by_username(&username).await? {
            Some(existing_user) => {
                // Check if user is banned
                if self.state.db().is_user_banned(existing_user.user_id).await? {
                    warn!("Banned user attempted to connect: {}", username);
                    return Ok(());
                }
                
                // Update last login
                self.state.db().update_last_login(existing_user.user_id).await?;
                existing_user
            }
            None => {
                // Create new guest user
                let user_id = self.state.db().create_user(&username, None).await?;
                self.state.db().get_user_by_id(user_id).await?
                    .context("Failed to get newly created user")?
            }
        };

        let user_id = user.user_id;
        self.user_id = Some(user_id);
        self.username = Some(username.clone());

        // Register session in state
        self.state
            .register_session(
                user_id,
                username.clone(),
                self.current_room,
                self.addr,
                self.message_tx.clone(),
            )
            .await;

        // Send server info
        self.send_server_info(user_id).await?;

        // Send user list for current room
        self.send_user_list().await?;

        // Send room description
        self.send_room_description().await?;

        // Notify other users
        self.broadcast_user_joined().await?;

        Ok(())
    }

    /// Handle talk (chat) message
    async fn handle_talk(&mut self, message: Message) -> Result<()> {
        let talk = message
            .parse_payload::<TalkMsg>()
            .context("Failed to parse talk message")?;

        if let Some(user_id) = self.user_id {
            info!("User {} says: {}", user_id, talk.text);

            // Broadcast to room
            let broadcast_msg = ServerMessage::Chat {
                from_user_id: user_id,
                room_id: self.current_room,
                message: talk.text.clone(),
                encrypted: false,
            };

            self.state
                .broadcast_to_room(self.current_room, broadcast_msg)
                .await;
        }

        Ok(())
    }

    /// Handle xtalk (extended chat) message
    async fn handle_xtalk(&mut self, message: Message) -> Result<()> {
        let xtalk = message
            .parse_payload::<XTalkMsg>()
            .context("Failed to parse xtalk message")?;

        // Decrypt the message text
        let text = xtalk
            .decrypt()
            .context("Failed to decrypt xtalk message")?;

        if let Some(user_id) = self.user_id {
            info!("User {} says (extended): {}", user_id, text);

            // Broadcast to room (send encrypted bytes)
            let broadcast_msg = ServerMessage::Chat {
                from_user_id: user_id,
                room_id: self.current_room,
                message: text,
                encrypted: true,
            };

            self.state
                .broadcast_to_room(self.current_room, broadcast_msg)
                .await;
        }

        Ok(())
    }

    /// Handle whisper (private message)
    async fn handle_whisper(&mut self, message: Message) -> Result<()> {
        let whisper = message
            .parse_payload::<XWhisperMsg>()
            .context("Failed to parse whisper message")?;

        // Decrypt the message text
        let text = whisper
            .decrypt()
            .context("Failed to decrypt whisper message")?;

        if let Some(from_user_id) = self.user_id {
            let target_user_id = whisper.target as UserId;
            info!(
                "User {} whispers to {}: {}",
                from_user_id, target_user_id, text
            );

            // Send to target user (simplified - would need XWhisperMsg)
            // For now, just log it
            // TODO: Implement private messaging properly
        }

        Ok(())
    }

    /// Handle room goto message
    async fn handle_room_goto(&mut self, message: Message) -> Result<()> {
        let goto = message
            .parse_payload::<RoomGotoMsg>()
            .context("Failed to parse room goto message")?;

        if let Some(user_id) = self.user_id {
            let new_room = goto.dest;
            info!("User {} moving to room {}", user_id, new_room);

            // Move user to new room
            if self.state.move_user_to_room(user_id, new_room).await {
                let old_room = self.current_room;
                self.current_room = new_room;

                // Notify users in old room
                let left_msg = ServerMessage::UserLeft {
                    user_id,
                    room_id: old_room,
                };
                self.state.broadcast_to_room(old_room, left_msg).await;

                // Send new room description
                self.send_room_description().await?;

                // Send user list for new room
                self.send_user_list().await?;

                // Notify users in new room
                self.broadcast_user_joined().await?;
            } else {
                warn!("Room {} not found", new_room);
            }
        }

        Ok(())
    }

    /// Handle list rooms request
    async fn handle_list_rooms(&mut self, _message: Message) -> Result<()> {
        // Get rooms from database
        let rooms = self.state.db().get_all_rooms().await?;

        // Create room list message with current user counts
        let mut room_list_recs = Vec::new();
        for room in rooms {
            let user_count = self.state.get_room_user_count(room.room_id as i16).await;
            room_list_recs.push(RoomListRec {
                room_id: room.room_id as i32,
                flags: RoomFlags::from_bits_truncate(room.flags as u16),
                nbr_users: user_count,
                name: room.name,
            });
        }

        let room_list = ListOfAllRoomsMsg {
            rooms: room_list_recs,
        };

        let msg = room_list.to_message_default();
        self.send_message(&msg).await?;

        Ok(())
    }

    /// Handle ping message
    async fn handle_ping(&mut self, _message: Message) -> Result<()> {
        // Send pong response
        let pong = Message::new_empty(MessageId::Pong, 0);
        self.send_message(&pong).await?;
        Ok(())
    }

    /// Handle server broadcast messages
    async fn handle_server_message(&mut self, msg: ServerMessage) -> Result<()> {
        match msg {
            ServerMessage::UserJoined {
                user_id,
                room_id,
                username,
            } => {
                if room_id == self.current_room && Some(user_id) != self.user_id {
                    info!("User '{}' joined room {}", username, room_id);
                    // Send UserNew message to this client
                    self.send_user_new(user_id, &username).await?;
                }
            }
            ServerMessage::UserLeft { user_id, room_id } => {
                if room_id == self.current_room && Some(user_id) != self.user_id {
                    info!("User {} left room {}", user_id, room_id);
                    // Send user status update
                    // TODO: Implement proper user leave notification
                }
            }
            ServerMessage::Chat {
                from_user_id,
                room_id,
                message: text,
                encrypted,
            } => {
                if room_id == self.current_room {
                    if encrypted {
                        // Re-encrypt and send as XTalkMsg
                        let xtalk = XTalkMsg::encrypt(&text)
                            .context("Failed to encrypt chat message")?;
                        let msg = xtalk.to_message(from_user_id as i32);
                        self.send_message(&msg).await?;
                    } else {
                        // Send as plain TalkMsg
                        let talk = TalkMsg { text };
                        let msg = talk.to_message(from_user_id as i32);
                        self.send_message(&msg).await?;
                    }
                }
            }
            ServerMessage::UserDisconnected { user_id: _ } => {
                // Handle user disconnect
                // TODO: Send user status update
            }
        }

        Ok(())
    }

    /// Send server info message
    async fn send_server_info(&mut self, user_id: UserId) -> Result<()> {
        use thepalace::messages::flags::{DownloadCaps, ServerFlags, UploadCaps};

        let server_info = ServerInfoMsg::new(
            ServerFlags::empty(),
            "Palace Server".to_string(), // Use hardcoded name for now
            0,
            UploadCaps::empty(),
            DownloadCaps::empty(),
        );

        let msg = server_info.to_message(user_id as i32);
        self.send_message(&msg).await
    }

    /// Send user list for current room
    async fn send_user_list(&mut self) -> Result<()> {
        let users = self.state.get_room_users(self.current_room).await;

        let user_list = UserListMsg {
            users: users
                .into_iter()
                .map(|(user_id, username)| thepalace::messages::UserRec {
                    user_id: user_id as i32,
                    room_pos: Point::new(128, 128), // Default position
                    prop_spec: [AssetSpec { id: 0, crc: 0 }; 9],
                    room_id: self.current_room,
                    face_nbr: 0,
                    color_nbr: 0,
                    away_flag: 0,
                    open_to_msgs: 1,
                    nbr_props: 0,
                    name: username,
                })
                .collect(),
        };

        let msg = user_list.to_message_default();
        self.send_message(&msg).await
    }

    /// Send room description
    async fn send_room_description(&mut self) -> Result<()> {
        use bytes::BufMut;
        use thepalace::messages::flags::RoomFlags;
        use thepalace::messages::RoomRec;

        // Get room from database
        if let Some(room) = self.state.db().get_room(self.current_room).await? {
            // Build variable buffer with room strings
            let mut var_buf = BytesMut::new();

            // Room name (PString format: length byte + data)
            let room_name_ofst = var_buf.len() as i16;
            var_buf.put_u8(room.name.len() as u8);
            var_buf.put_slice(room.name.as_bytes());

            // Background picture name
            let pict_name_ofst = var_buf.len() as i16;
            let bg_name = room.background_image.unwrap_or_else(|| format!("room{}.png", room.room_id));
            var_buf.put_u8(bg_name.len() as u8);
            var_buf.put_slice(bg_name.as_bytes());

            // Artist name
            let artist_name_ofst = var_buf.len() as i16;
            let artist = room.artist.unwrap_or_else(|| "Palace Server".to_string());
            var_buf.put_u8(artist.len() as u8);
            var_buf.put_slice(artist.as_bytes());

            // Password (empty)
            let password_ofst = var_buf.len() as i16;
            var_buf.put_u8(0);

            let len_vars = var_buf.len() as i16;

            // Get current user count from in-memory state
            let nbr_people = self.state.get_room_user_count(self.current_room).await;

            let room_rec = RoomRec {
                room_flags: RoomFlags::from_bits_truncate(room.flags as u16),
                faces_id: room.faces_id as i32,
                room_id: room.room_id as i16,
                room_name_ofst,
                pict_name_ofst,
                artist_name_ofst,
                password_ofst,
                nbr_hotspots: 0, // TODO: Query hotspots from DB
                hotspot_ofst: 0,
                nbr_pictures: 0,
                picture_ofst: 0,
                nbr_draw_cmds: 0,
                first_draw_cmd: 0,
                nbr_people,
                nbr_lprops: 0, // TODO: Query loose props from DB
                first_lprop: 0,
                len_vars,
                var_buf: var_buf.freeze(),
            };

            let room_desc = RoomDescMsg { room: room_rec };

            let msg = room_desc.to_message_default();
            self.send_message(&msg).await?;
        }

        Ok(())
    }

    /// Broadcast user joined to room
    async fn broadcast_user_joined(&mut self) -> Result<()> {
        if let (Some(user_id), Some(username)) = (self.user_id, &self.username) {
            let broadcast_msg = ServerMessage::UserJoined {
                user_id,
                room_id: self.current_room,
                username: username.clone(),
            };

            self.state
                .broadcast_to_room(self.current_room, broadcast_msg)
                .await;
        }

        Ok(())
    }

    /// Send UserNew message for a specific user
    async fn send_user_new(&mut self, user_id: UserId, username: &str) -> Result<()> {
        let user_new = UserNewMsg {
            new_user: thepalace::messages::UserRec {
                user_id: user_id as i32,
                room_pos: Point::new(128, 128),
                prop_spec: [AssetSpec { id: 0, crc: 0 }; 9],
                room_id: self.current_room,
                face_nbr: 0,
                color_nbr: 0,
                away_flag: 0,
                open_to_msgs: 1,
                nbr_props: 0,
                name: username.to_string(),
            },
        };

        let msg = user_new.to_message_default();
        self.send_message(&msg).await
    }

    /// Send a message to the client
    async fn send_message(&mut self, message: &Message) -> Result<()> {
        let bytes = message.to_bytes();
        self.socket
            .write_all(&bytes)
            .await
            .context("Failed to send message")?;

        debug!("Sent message: {:?} ({} bytes)", message.msg_id, bytes.len());
        Ok(())
    }
}
