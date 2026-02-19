# Palace Visual Chat System - Architecture

## Overview

The Palace is a visual chat system originally developed in the 1990s. This document describes the architecture of our modern implementation using Rust for the server and C++ with Qt for the client.

## System Components

```
┌─────────────────────────────────────────────────────────────┐
│                        Palace System                         │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐         ┌──────────────┐                  │
│  │ Qt C++ Client│◄───────►│ Rust Server  │                  │
│  │              │  TCP    │              │                  │
│  │ - GUI (QML)  │  9998   │ - Tokio      │                  │
│  │ - Graphics   │         │ - SQLx       │                  │
│  │ - Protocol   │         │ - Database   │                  │
│  └──────┬───────┘         └──────┬───────┘                  │
│         │                        │                           │
│         │                        │                           │
│    ┌────▼─────────────────────────▼─────┐                   │
│    │    libthepalace (Rust + FFI)       │                   │
│    │                                     │                   │
│    │  - Protocol Types & Parsing        │                   │
│    │  - Iptscrae Interpreter            │                   │
│    │  - Prop Format Handling            │                   │
│    │  - Room Format Parsing             │                   │
│    │  - Algorithms (CRC32, Crypto)      │                   │
│    └────────────────────────────────────┘                   │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

## Technology Stack

### Server
- **Language:** Rust (edition 2024)
- **Async Runtime:** Tokio
- **Database:** SQLite with SQLx
- **Migrations:** SQLx migrations
- **Logging:** tracing + tracing-subscriber
- **Config:** TOML with serde

### Client
- **Language:** C++23
- **GUI Framework:** Qt 6.10 with QML
- **Graphics:** Qt RHI (Vulkan/Direct3D/Metal/OpenGL)
- **Build System:** CMake 3.21+
- **Protocol:** FFI to libthepalace

### Shared Library (libthepalace)
- **Language:** Rust
- **Features:** Protocol, Iptscrae, Assets, Room Format
- **FFI:** C bindings via cbindgen
- **Build:** Cargo with feature flags

## Palace Protocol

### Message Structure

All messages share a common 12-byte header:

```rust
struct ClientMsg {
    event_type: u32,    // 4-byte opcode (e.g., 0x74697972 'tiyr')
    length: u32,        // Length of msg body
    ref_num: i32,       // Arbitrary parameter
    msg: Vec<u8>,       // Message-specific payload
}
```

### Endianness Detection

The first message (`MSG_TIYID`) uses character order to detect endianness:
- Big-endian: `0x74697972` reads as "tiyr"
- Little-endian: `0x74697972` reads as "ryit"

All subsequent data uses the detected byte order.

### Core Message Types

From the protocol PDF, there are 60+ message types. Key categories:

#### Connection & Auth
- `MSG_TIYID` (0x74697972) - Initial handshake
- `MSG_LOGON` (0x72656769) - User login
- `MSG_ALTLOGONREPLY` (0x72657032) - Extended login response
- `MSG_AUTHENTICATE` (0x61757468) - Auth request
- `MSG_AUTHRESPONSE` (0x61757472) - Username:password
- `MSG_SUPERUSER` (0x73757372) - Wizard elevation
- `MSG_LOGOFF` (0x62796520) - Disconnect

#### Rooms
- `MSG_ROOMGOTO` (0x6e617652) - Navigate to room
- `MSG_ROOMDESC` (0x726f6f6d) - Room description
- `MSG_ROOMDESCEND` (0x656e6472) - End of room desc
- `MSG_LISTOFALLROOMS` (0x724c7374) - List all rooms

#### Users
- `MSG_USERNEW` (0x6e707273) - User entered
- `MSG_USEREXIT` (0x65707273) - User left
- `MSG_USERLIST` (0x72707273) - User list
- `MSG_USERMOVE` (0x754c6f63) - User moved
- `MSG_USERFACE` (0x75737246) - Face changed
- `MSG_USERPROP` (0x75737250) - Props changed

#### Chat
- `MSG_TALK` (0x74616c6b) - Normal chat
- `MSG_WHISPER` (0x77686973) - Private message
- `MSG_XTALK` (0x78746c6b) - Extended talk
- `MSG_XWHISPER` (0x78776973) - Extended whisper

#### Assets
- `MSG_ASSETQUERY` (0x71417374) - Request asset
- `MSG_ASSETSEND` (0x73417374) - Send asset
- `MSG_ASSETREGI` (0x72417374) - Register asset

#### Misc
- `MSG_PING` (0x70696e67) - Keepalive
- `MSG_PONG` (0x706f6e67) - Keepalive response
- `MSG_BLOWTHRU` (0x626c6f77) - Plugin relay

Full list: See `lib/thepalace/src/messages/mod.rs`

### String Types

Palace uses multiple string representations:

```c
// Pascal-style string (length-prefixed)
struct PString {
    uint8 length;
    char chars[length];
}

// C-style string (null-terminated)
struct CString {
    char chars[];  // ends with \0
}

// Fixed-size Pascal strings
struct Str31 {
    uint8 length;
    char chars[31];  // padded with zeros
}

struct Str63 {
    uint8 length;
    char chars[63];
}
```

### Core Data Types

```rust
// 2D Point
struct Point {
    v: i16,  // vertical
    h: i16,  // horizontal
}

// Asset identification
struct AssetSpec {
    id: i32,
    crc: u32,
}

// Asset types
const RT_PROP: u32 = 0x50726f70;        // 'Prop'
const RT_USERBASE: u32 = 0x55736572;    // 'User'
const RT_IPUSERBASE: u32 = 0x49557372;  // 'IUsr'

// ID types
type UserID = i32;
type RoomID = i16;
type HotspotID = i16;
```

## Iptscrae Scripting Language

### Overview

Iptscrae is a stack-based scripting language embedded in Palace for interactive behaviors.

### Language Characteristics

- **Stack-based:** Operations manipulate a value stack
- **Loosely typed:** Values can be integers or strings
- **Event-driven:** Scripts respond to events (ENTER, SELECT, etc.)
- **Procedural:** No functions, just linear execution with control flow

### Script Events

Scripts can respond to 20+ event types:

```rust
bitflags! {
    pub struct EventMask: u32 {
        const SELECT     = 0x00000001;  // Hotspot clicked
        const LOCK       = 0x00000002;  // Door locked
        const UNLOCK     = 0x00000004;  // Door unlocked
        const HIDE       = 0x00000008;  // Hotspot hidden
        const SHOW       = 0x00000010;  // Hotspot shown
        const STARTUP    = 0x00000020;  // Room startup
        const ALARM      = 0x00000040;  // Timer alarm
        const CUSTOM     = 0x00000080;  // Custom event
        const INCHAT     = 0x00000100;  // Chat received
        const PROPCHANGE = 0x00000200;  // Prop changed
        const ENTER      = 0x00000400;  // User entered
        const LEAVE      = 0x00000800;  // User left
        const OUTCHAT    = 0x00001000;  // Chat sent
        const SIGNON     = 0x00002000;  // User logged on
        const SIGNOFF    = 0x00004000;  // User logged off
        const MACRO0     = 0x00008000;  // Macro 0
        // ... MACRO1-9 (up to 0x01000000)
    }
}

// Usage:
let mask = EventMask::SELECT | EventMask::ENTER | EventMask::LEAVE;
if mask.contains(EventMask::SELECT) { /* ... */ }
```

### Example Scripts

```iptscrae
# Simple greeting
ON ENTER {
    WHONAME " has entered the room!" & SAY
}

# Interactive hotspot
ON SELECT {
    "You clicked me!" SAY
    2 SETFACE  # Set to smiling face
}

# Variable usage
ON ENTER {
    0 counter =
    {
        counter 10 < {
            counter ITOA SAY
            counter 1 + counter =
        } IF
    } WHILE
}
```

### Standard Library Functions

**Stack Operations:**
- `DUP` - Duplicate top value
- `DROP` - Remove top value
- `SWAP` - Swap top two values
- `OVER` - Copy second value to top
- `ROT` - Rotate top three values
- `PICK` - Copy nth value to top

**Arithmetic:**
- `+`, `-`, `*`, `/`, `MOD`
- Negation via unary operator: `-5`

**Comparison:**
- `=`, `!=`, `<`, `>`, `<=`, `>=`

**Logic:**
- `AND`, `OR`, `NOT`, `XOR`

**String:**
- `&` - Concatenate
- `STRLEN` - String length
- `TOUPPER`, `TOLOWER`
- `ITOA` - Integer to string
- `ATOI` - String to integer

**Control Flow:**
- `IF`, `ELSE`
- `WHILE`, `DO`
- `BREAK`

**Palace Functions:**
- `SAY` - Display message
- `CHAT` - Send chat
- `LOCALMSG` - Message only local user sees
- `ROOMMSG` - Message everyone in room sees
- `PRIVATEMSG` - Private message to user (requires userID)
- `USERNAME` - Get current username
- `WHOME` - Get current user ID
- `WHONAME` - Get username for userID (requires userID parameter)
- `SETFACE` - Set avatar face (0-12)
- `SETCOLOR` - Set roundhead color (0-15)
- `GETPROPS`, `SETPROPS` - User props
- `ROOMNAME`, `ROOMID` - Room info
- `GOTOROOM` - Navigate to room
- `LOCK`, `UNLOCK` - Door control (requires doorID)

### Security Model

#### Server Scripts (Full Trust)
- Room scripts executed on server
- Full access to all functions
- No sandboxing needed
- Can affect all users

#### Cyborg Scripts (Sandboxed)
- Client-side user scripts
- Can be disabled per-room or server-wide
- Restrictions:
  - Instruction limit (prevent infinite loops)
  - Execution timeout
  - No network access
  - No file access
  - Limited navigation (respect security)
  - Cannot access other users' variables

```rust
pub struct ScriptContext {
    security_level: SecurityLevel,
    instruction_limit: usize,
    timeout: Duration,
    allowed_functions: HashSet<String>,
}

pub enum SecurityLevel {
    Server,      // Full trust
    Cyborg,      // Sandboxed
    Admin,       // Elevated privileges
}
```

## Prop Format

### Prop Structure

Props are visual assets (images) used for avatars and room decorations.

```rust
struct PropHeader {
    magic: u32,        // 0x50726f70 ('Prop')
    flags: PropFlags,
    width: i16,
    height: i16,
    // ... format-specific data
}
```

### Prop Flags

```rust
bitflags! {
    pub struct PropFlags: u16 {
        const FORMAT_8BIT    = 0x0000;  // 8-bit indexed color
        const HEAD           = 0x0002;  // Head prop
        const GHOST          = 0x0004;  // Ghost (transparent)
        const RARE           = 0x0008;  // Rare prop
        const ANIMATE        = 0x0010;  // Animated
        const BOUNCE         = 0x0020;  // Bounces
        const FORMAT_20BIT   = 0x0040;  // 20-bit color
        const FORMAT_32BIT   = 0x0100;  // 32-bit color
        const FORMAT_S20BIT  = 0x0200;  // Signed 20-bit
        const FORMAT_MASK    = Self::FORMAT_20BIT.bits()
                             | Self::FORMAT_32BIT.bits()
                             | Self::FORMAT_S20BIT.bits();
    }
}
```

### 8-bit Format (Initial Implementation)

- Indexed color palette (256 colors)
- Each pixel is 1 byte (palette index)
- Color 255 typically transparent
- Palette stored separately or uses default Mac palette

```rust
pub struct Prop8Bit {
    width: u16,
    height: u16,
    palette: [RGB; 256],
    pixels: Vec<u8>,  // width * height bytes
}
```

### Asset Storage

Assets stored on filesystem with CRC32 as filename:

```
assets/props/A95ADE76.prop
assets/props/B1234567.prop
```

Database stores metadata and file path references.

## Room Format

### RoomRec Structure

Rooms are complex structures containing:
- Background image
- Hotspots (interactive areas)
- Loose props (placed decorations)
- Scripts
- Pictures

```rust
pub struct RoomRec {
    pub id: RoomID,
    pub name: String,
    pub background_id: i16,
    pub artist: String,
    pub password: Option<String>,
    pub flags: RoomFlags,
    pub faces_id: i16,
    pub hotspots: Vec<Hotspot>,
    pub pictures: Vec<PictureRec>,
    pub loose_props: Vec<LPropRec>,
    pub max_occupancy: i16,
}
```

### Room Flags

```rust
bitflags! {
    pub struct RoomFlags: u16 {
        const AUTHOR_LOCKED   = 0x0001;  // Author only
        const PRIVATE         = 0x0002;  // Private room
        const NO_PAINT        = 0x0004;  // Drawing disabled
        const CLOSED          = 0x0008;  // Closed
        const NO_SCRIPT       = 0x0010;  // Scripts disabled
        const HIDDEN          = 0x0020;  // Hidden from list
        const NO_GUESTS       = 0x0040;  // Guests not allowed
        const WIZARDS_ONLY    = 0x0080;  // Wizards only
        const DROP_ZONE       = 0x0100;  // Drop zone
        const NO_LPROPS       = 0x0200;  // No loose props
        const CYBORG_FREE     = 0x1000;  // Cyborg scripts disabled
    }
}
```

### Hotspot Structure

```rust
pub struct Hotspot {
    pub id: HotspotID,
    pub name: String,
    pub loc: Point,
    pub hotspot_type: HotspotType,
    pub dest: i16,              // Room ID for doors
    pub script_event_mask: u32,
    pub script_text: String,
    pub state: i16,             // 0=unlocked, 1=locked
    pub points: Vec<Point>,     // Polygon outline
}

pub enum HotspotType {
    Normal = 0,           // Just a script holder
    Door = 1,             // Navigation
    ShutableDoor = 2,     // Can open/close
    LockableDoor = 3,     // Can lock
    Bolt = 4,             // Locks other doors
    NavArea = 5,          // Navigation area
}
```

## Database Schema

### SQLite Schema

```sql
-- Users
CREATE TABLE users (
    user_id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE COLLATE NOCASE,
    password_hash TEXT,
    wizard_password TEXT,
    flags INTEGER NOT NULL DEFAULT 8,
    registration_date INTEGER NOT NULL,
    last_login INTEGER
);

-- Rooms
CREATE TABLE rooms (
    room_id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    artist TEXT,
    background_image TEXT,
    flags INTEGER NOT NULL DEFAULT 0,
    max_occupancy INTEGER DEFAULT 0,
    faces_id INTEGER DEFAULT 0,
    room_data BLOB
);

-- Props registry
CREATE TABLE props (
    prop_id INTEGER PRIMARY KEY AUTOINCREMENT,
    crc32 INTEGER NOT NULL UNIQUE,
    name TEXT NOT NULL,
    flags INTEGER NOT NULL,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    file_path TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

-- Loose props in rooms
CREATE TABLE room_loose_props (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    room_id INTEGER NOT NULL,
    prop_id INTEGER NOT NULL,
    pos_h INTEGER NOT NULL,
    pos_v INTEGER NOT NULL,
    FOREIGN KEY (room_id) REFERENCES rooms(room_id) ON DELETE CASCADE,
    FOREIGN KEY (prop_id) REFERENCES props(prop_id) ON DELETE CASCADE
);

-- Hotspots
CREATE TABLE hotspots (
    hotspot_id INTEGER PRIMARY KEY AUTOINCREMENT,
    room_id INTEGER NOT NULL,
    id INTEGER NOT NULL,
    name TEXT,
    type INTEGER NOT NULL,
    dest_room_id INTEGER,
    dest_hotspot_id INTEGER,
    loc_h INTEGER NOT NULL,
    loc_v INTEGER NOT NULL,
    script_event_mask INTEGER NOT NULL DEFAULT 0,
    script_text TEXT,
    state INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (room_id) REFERENCES rooms(room_id) ON DELETE CASCADE
);

-- Ban list
CREATE TABLE bans (
    ban_id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER,
    ip_address TEXT,
    reason TEXT,
    banned_at INTEGER NOT NULL,
    expires_at INTEGER,
    banned_by_user_id INTEGER,
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE
);
```

## Server Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────┐
│              Palace Server                       │
├─────────────────────────────────────────────────┤
│                                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐      │
│  │Console   │  │TCP       │  │Database  │      │
│  │(stdin)   │  │Listener  │  │(SQLite)  │      │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘      │
│       │             │              │            │
│       ├─────────────┼──────────────┤            │
│       │             │              │            │
│  ┌────▼─────────────▼──────────────▼──────┐    │
│  │          Server Core (Tokio)           │    │
│  │                                         │    │
│  │  ┌─────────────────────────────────┐   │    │
│  │  │  Connection Manager             │   │    │
│  │  │  - One task per connection      │   │    │
│  │  │  - Message framing & parsing    │   │    │
│  │  └────────────┬────────────────────┘   │    │
│  │               │                         │    │
│  │  ┌────────────▼────────────────────┐   │    │
│  │  │  Message Handler (Dispatcher)   │   │    │
│  │  │  - Routes messages to handlers  │   │    │
│  │  └────────────┬────────────────────┘   │    │
│  │               │                         │    │
│  │  ┌────────────▼────────────────────┐   │    │
│  │  │  Room Manager                   │   │    │
│  │  │  - Room state                   │   │    │
│  │  │  - User tracking                │   │    │
│  │  │  - Event broadcasting           │   │    │
│  │  └────────────┬────────────────────┘   │    │
│  │               │                         │    │
│  │  ┌────────────▼────────────────────┐   │    │
│  │  │  Script Executor                │   │    │
│  │  │  - Iptscrae VM                  │   │    │
│  │  │  - Event triggers               │   │    │
│  │  └─────────────────────────────────┘   │    │
│  └─────────────────────────────────────────┘    │
│                                                  │
└─────────────────────────────────────────────────┘
```

### Concurrency Model

- **Tokio tasks:** One per connection, plus listener and console
- **Shared state:** Arc<Mutex<>> for room state, user sessions
- **Message passing:** mpsc channels for inter-task communication
- **Database:** SQLx connection pool for concurrent queries

## Client Architecture

### Component Diagram

```
┌────────────────────────────────────────────────┐
│            Palace Client (Qt/QML)               │
├────────────────────────────────────────────────┤
│                                                 │
│  ┌──────────────────────────────────────────┐  │
│  │         QML User Interface               │  │
│  │                                          │  │
│  │  ┌──────────┐  ┌──────────┐            │  │
│  │  │Login     │  │Main      │            │  │
│  │  │Dialog    │  │Window    │            │  │
│  │  └──────────┘  └──────────┘            │  │
│  │                                          │  │
│  │  ┌──────────────────────────────────┐   │  │
│  │  │   PalaceCanvas (QQuickItem)      │   │  │
│  │  │   - Room rendering               │   │  │
│  │  │   - User avatars                 │   │  │
│  │  └──────────────────────────────────┘   │  │
│  └──────────────────────────────────────────┘  │
│                     │                           │
│  ┌──────────────────▼──────────────────────┐   │
│  │       C++ Application Layer             │   │
│  │                                          │   │
│  │  ┌────────────┐     ┌────────────┐     │   │
│  │  │ Network    │     │ Graphics   │     │   │
│  │  │ Connection │     │ Renderer   │     │   │
│  │  └──────┬─────┘     └──────┬─────┘     │   │
│  │         │                  │            │   │
│  │         │   ┌──────────────▼─────┐     │   │
│  │         │   │  RHI/Software      │     │   │
│  │         │   │  Rendering         │     │   │
│  │         │   └────────────────────┘     │   │
│  │         │                               │   │
│  │  ┌──────▼──────────────────────────┐   │   │
│  │  │   Protocol (FFI to libthepalace)│   │   │
│  │  └─────────────────────────────────┘   │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Graphics Pipeline

#### Hardware Accelerated (Qt RHI)

```cpp
// Qt selects best backend:
// - Windows: Direct3D 11/12
// - macOS: Metal
// - Linux: Vulkan
// - Fallback: OpenGL

class PalaceCanvas : public QQuickItem {
    QSGNode *updatePaintNode(QSGNode *old, 
                             UpdatePaintNodeData *data) override {
        // Create custom scene graph node
        // Submit rendering commands via RHI
    }
};
```

#### Software Fallback

```cpp
class SoftwareCanvas : public QWidget {
    void paintEvent(QPaintEvent *event) override {
        QPainter painter(this);
        // CPU-based rendering
        painter.drawImage(0, 0, background);
        for (auto &user : users) {
            painter.drawImage(user.pos, user.avatar);
        }
    }
};
```

## Build System

### Cargo Workspace (Rust)

```toml
# Root Cargo.toml
[workspace]
members = ["lib/thepalace", "server"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "GPL-3.0"
```

### CMake (C++ Client)

```cmake
cmake_minimum_required(VERSION 3.21)
project(PalaceClient VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 23)

find_package(Qt6 6.10 REQUIRED COMPONENTS 
    Core Gui Quick Qml Network ShaderTools)

# Build Rust library first
add_custom_target(thepalace_rust ALL
    COMMAND cargo build --release
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR})

# Link client against libthepalace
add_executable(palace-client ${SOURCES})
target_link_libraries(palace-client 
    Qt6::Quick 
    thepalace)
```

## Security Considerations

### Authentication
- **Passwords:** Currently plaintext in protocol (legacy)
- **Wizard Auth:** Separate wizard password for elevation
- **Future:** Add TLS/SSL, modern auth methods

### Input Validation
- **Message sizes:** Validate length fields
- **String bounds:** Check string lengths
- **SQL Injection:** Use parameterized queries (SQLx)
- **Script timeouts:** Prevent infinite loops

### Rate Limiting
- **Connection limits:** Max connections per IP
- **Message throttling:** Limit messages per second
- **Asset requests:** Throttle asset queries

## Performance Optimization

### Server
- **Connection pooling:** SQLx manages database connections
- **Async I/O:** Tokio for non-blocking operations
- **Message batching:** Group updates for busy rooms
- **Asset caching:** Cache frequently-used props in memory

### Client
- **Sprite batching:** Render all avatars in single call
- **Texture atlasing:** Combine small textures
- **Dirty rectangles:** Only redraw changed areas
- **LOD:** Reduce quality for distant users

## Testing Strategy

### Unit Tests
- Protocol message parsing/serialization
- Iptscrae lexer, parser, VM
- Prop format decoding
- CRC32 and encryption algorithms

### Integration Tests
- Server connection handling
- Room navigation
- Chat message routing
- Database operations

### Compatibility Tests
- Connect with original Palace client
- Test all message types
- Verify Iptscrae scripts
- Cross-platform client testing

## References

- **Protocol PDF:** `/home/admin/Downloads/PalaceProtocolRef.pdf`
- **Original Palace:** The Palace, Inc. (1995-1999)
- **Qt Documentation:** https://doc.qt.io/qt-6/
- **Tokio Documentation:** https://tokio.rs/
- **SQLx Documentation:** https://github.com/launchbadge/sqlx
