# Palace Visual Chat System

A modern implementation of The Palace visual chat system, featuring a Rust server with Tokio and SQLite, and a C++ Qt client with hardware-accelerated graphics.

## Overview

The Palace is a 2D graphical chat system where users navigate rooms as avatars, interact with props, and communicate through chat and scripted behaviors. This implementation aims for compatibility with original Palace clients while providing modern features and performance.

## Features

### Server (Rust)
- 🚀 **Async networking** with Tokio for robust concurrent connections
- 💾 **SQLite database** for users, rooms, props, and metadata
- 🎭 **Full Iptscrae interpreter** for room scripting
- 🔧 **Interactive console** for server administration
- 🔌 **Extension support** for PalaceChat and Phalanx
- ✅ **Compatible** with original Palace clients

### Client (C++ with Qt 6.10)
- 🎨 **Hardware-accelerated rendering** via Qt RHI (Vulkan/D3D/Metal/OpenGL)
- 📱 **Modern QML interface** with responsive design
- 💻 **Cross-platform** (Windows, macOS, Linux)
- 🎯 **Software rendering fallback** for compatibility
- 🖼️ **Full prop support** with 8-bit format (20/32-bit planned)
- 🔌 **Native C++ protocol** implementation

### Protocol Library (libthepalace)
- 📦 **Complete Palace protocol** (60+ message types)
- 🔤 **Iptscrae language** support (lexer, parser, VM)
- 🖼️ **Asset handling** (props, backgrounds)
- 🏠 **Room format parsing** (.ipr files)
- 🔒 **CRC32 and encryption** algorithms
- ✨ **Used by Rust server** (client has independent C++ implementation)

## Architecture

```
┌──────────────────┐         ┌──────────────┐
│ Qt C++ Client    │◄───────►│ Rust Server  │
│                  │  TCP    │              │
│ - QML UI         │  9998   │ - Tokio      │
│ - RHI Graphics   │         │ - SQLx       │
│ - Protocol (C++) │         │ - Protocol   │
│ - Network (Qt)   │         │ - Iptscrae   │
└──────────────────┘         └──────┬───────┘
                                    │
                           ┌────────▼────────┐
                           │  libthepalace   │
                           │  (Rust)         │
                           │                 │
                           │ - Protocol      │
                           │ - Iptscrae VM   │
                           │ - Prop Format   │
                           │ - CRC32/Crypto  │
                           └─────────────────┘
                             (Server only)
```

## Technology Stack

| Component | Technologies |
|-----------|-------------|
| **Server** | Rust, Tokio, SQLx, SQLite, Tracing |
| **Client** | C++23, Qt 6.10, QML, Qt RHI, Qt Network |
| **Protocol** | Server: Rust (libthepalace), Client: C++ (native) |
| **Build** | Cargo (server) + CMake (client) |

## Project Structure

```
Palace/
├── lib/thepalace/          # Shared protocol library (Rust)
│   ├── src/
│   │   ├── lib.rs          # Core types
│   │   ├── algo.rs         # CRC32, encryption
│   │   ├── messages/       # Protocol messages (60+)
│   │   ├── iptscrae/       # Scripting language
│   │   ├── assets/         # Prop format handling
│   │   ├── room/           # Room format parsing
│   │   └── ffi.rs          # C bindings
│   └── include/
│       └── thepalace.h     # Generated C header
│
├── server/                 # Rust server
│   ├── src/
│   │   ├── main.rs
│   │   ├── db/             # Database layer
│   │   ├── net/            # Networking
│   │   ├── room/           # Room management
│   │   ├── script/         # Script execution
│   │   └── console/        # Admin console
│   └── migrations/         # SQLx migrations
│
├── client/                 # C++ Qt client
│   ├── src/
│   │   ├── main.cpp
│   │   ├── network/        # Connection, Protocol, Session
│   │   ├── graphics/       # Rendering (RHI + Software)
│   │   ├── ui/             # QML interface & models
│   │   └── settings/       # Settings management
│   └── resources/          # QML, fonts, icons
│
├── docs/                   # Documentation
│   ├── ARCHITECTURE.md
│   ├── PROTOCOL.md
│   └── IPTSCRAE.md
│
└── assets/                 # Default assets
    ├── props/
    ├── backgrounds/
    └── rooms/
```

## Building

### Prerequisites

**Server:**
- Rust 1.75+ (edition 2024 support)
- SQLite 3.35+

**Client:**
- C++23 compiler (GCC 13+, Clang 16+, MSVC 2022)
- CMake 3.21+
- Qt 6.10+

### Build Instructions

**1. Build the Rust workspace (library + server):**

```bash
# Clone repository
git clone https://github.com/yourusername/Palace.git
cd Palace

# Build everything in release mode
cargo build --release

# Run server
cd server
cargo run --release
```

**2. Build the C++ client:**

```bash
# From Palace root directory
cd client
mkdir build
cd build

# Configure
cmake .. -DCMAKE_BUILD_TYPE=Release

# Build
cmake --build . --parallel

# Run client
./palace-client  # Linux/macOS
# or
palace-client.exe  # Windows
```

**Note:** The client has a native C++ protocol implementation and does not depend on the Rust library.

### Development Build

```bash
# Rust (faster compilation, debug symbols)
cargo build

# C++ client (with debug info)
cmake .. -DCMAKE_BUILD_TYPE=Debug
cmake --build .
```

## Running

### Server

```bash
cd server

# Create default config if needed
cp palace.toml.example palace.toml

# Run migrations
cargo sqlx migrate run

# Start server
cargo run --release -- --config palace.toml
```

**Server console commands:**
```
> help              - Show all commands
> status            - Server status
> users             - List connected users
> rooms             - List rooms with occupancy
> kick <user>       - Disconnect user
> ban <ip|user>     - Ban IP or user
> broadcast <msg>   - Send global message
> shutdown          - Graceful shutdown
```

### Client

```bash
./build/palace-client
```

**Connection settings:**
- Default server: localhost
- Default port: 9998
- Login as guest or registered user

## Configuration

### Server Config (`server/palace.toml`)

```toml
[server]
host = "0.0.0.0"
port = 9998
max_connections = 100

[database]
path = "palace.db"
pool_size = 10

[security]
allow_guests = true
allow_cyborgs = true
max_prop_size = 1048576  # 1MB

[logging]
level = "info"
```

### Client Settings

Configured via UI:
- Graphics mode (Auto/Hardware/Software)
- Server bookmarks
- Avatar preferences
- Chat settings

## Palace Protocol

The Palace protocol is a binary TCP protocol with 60+ message types. Key features:

- **Endianness detection:** First message determines byte order
- **Message structure:** 12-byte header + variable payload
- **String formats:** PString, CString, Str31, Str63
- **Asset system:** Props identified by CRC32
- **Scripting:** Iptscrae event-driven scripts

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for complete protocol details.

## Iptscrae Scripting

Iptscrae is a stack-based scripting language for interactive behaviors:

```iptscrae
# Room greeting script
ON ENTER {
    WHONAME " has entered the room!" & SAY
}

# Interactive hotspot
ON SELECT {
    "You clicked me!" SAY
    2 SETFACE  # Set to smiling face
}
```

**Script Events:**
- `ON ENTER` - User enters room
- `ON LEAVE` - User leaves room
- `ON SELECT` - Hotspot clicked
- `ON INCHAT` - Chat received
- `ON ALARM` - Timer event
- ... and 15+ more

See [docs/IPTSCRAE.md](docs/IPTSCRAE.md) for language reference.

## Development Tools

### Ghidra MCP Bridge

For reverse-engineering original Palace binaries to extract protocol details:

```bash
# See .opencode/AGENTS.md for workflow
# Query Ghidra for message handlers, VM code, etc.
```

### Testing

```bash
# Rust tests
cargo test

# Run specific test
cargo test test_crc32

# With output
cargo test -- --nocapture
```

## Compatibility

### Tested Clients
- ✅ The Palace Client (Windows/Mac original)
- ⏳ PalaceChat (testing in progress)
- ⏳ Phalanx (testing in progress)

### Protocol Support
- ✅ Core messages (connection, rooms, users, chat)
- ✅ Asset queries and transfer
- ✅ Basic Iptscrae scripts
- ⏳ Advanced features (drawing, file transfer)
- ⏳ Extensions (PalaceChat/Phalanx)

## Roadmap

### Phase 1: Foundation ✅
- [x] Project structure
- [x] Protocol library skeleton
- [ ] Core types and algorithms
- [ ] Message parsing

### Phase 2: Server MVP
- [ ] TCP networking with Tokio
- [ ] Database schema and migrations
- [ ] Core message handlers
- [ ] Room navigation
- [ ] Chat functionality
- [ ] Asset serving

### Phase 3: Client MVP
- [ ] Qt project setup
- [ ] Network connection
- [ ] Basic UI (QML)
- [ ] Room rendering (software)
- [ ] Chat interface

### Phase 4: Graphics
- [ ] Qt RHI rendering
- [ ] Hardware acceleration
- [ ] Sprite rendering
- [ ] Texture management

### Phase 5: Iptscrae
- [ ] Complete interpreter
- [ ] All stdlib functions
- [ ] Event system
- [ ] Cyborg sandboxing

### Phase 6: Extensions
- [ ] PalaceChat support
- [ ] Phalanx support
- [ ] Compatibility testing

### Phase 7: Release
- [ ] Documentation
- [ ] Packaging (installers)
- [ ] CI/CD
- [ ] v1.0 release

## Contributing

Contributions welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Areas for Contribution
- Protocol message implementations
- Iptscrae stdlib functions
- Client UI improvements
- Cross-platform testing
- Documentation
- Extension support

## License

GPL-3.0-or-later

This project implements The Palace protocol for interoperability purposes. Palace is a trademark of its respective owners.

## Acknowledgments

- **Original Palace:** The Palace, Inc. (1995-1999)
- **Communities.com** for the protocol documentation
- **Palace community** for keeping the platform alive

## Contact

- **Repository:** https://github.com/yourusername/Palace
- **Issues:** https://github.com/yourusername/Palace/issues
- **Discussions:** https://github.com/yourusername/Palace/discussions

## Resources

- [Architecture Documentation](docs/ARCHITECTURE.md)
- [Protocol Reference](docs/protocol/PalaceProtocolRef.pdf)
- [Iptscrae Language Guide](docs/IPTSCRAE.md)
- [Database Schema](docs/DATABASE.md)
- [Building Instructions](docs/BUILDING.md)
