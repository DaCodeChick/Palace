# Iptscrae Protocol Analysis

## Overview

This document describes how Iptscrae scripts are transmitted, triggered, and executed in the Palace protocol. This analysis is derived from the Palace Protocol Reference PDF and serves as the foundation for implementing client-side Iptscrae support.

## Key Findings

### 1. Script Transmission Mechanism

Scripts are embedded in **room descriptions** (MSG_ROOMDESC) as part of **Hotspot** structures. When a client enters a room, the server sends the complete room data including all hotspots with their associated scripts.

### 2. Hotspot Structure

Hotspots are clickable areas in a room that can execute scripts in response to events:

```c
struct Hotspot {
    sint32 scriptEventMask;    // Bit flags for which events trigger this script
    sint32 flags;              // Various hotspot flags (unused on server)
    sint32 secureInfo;         // Unused
    sint32 refCon;             // Unused
    Point loc;                 // Location of hotspot (x, y coordinates)
    sint16 id;                 // Hotspot ID number
    sint16 dest;               // Room ID for doors, or door ID for bolts
    sint16 nbrPts;             // Number of points defining click zone
    sint16 ptsOfst;            // Offset to Point array in varBuf
    sint16 type;               // Hotspot type (door, bolt, invisible, etc.)
    sint16 groupID;            // Group identifier
    sint16 nbrScripts;         // Number of scripts (usually 1)
    sint16 scriptRecOfst;      // Offset to script descriptor in varBuf
    sint16 state;              // Current state (selects which picture to show)
    sint16 nbrStates;          // Number of state pictures
    sint16 stateRecOfst;       // Offset to StateRec array in varBuf
    sint16 nameOfst;           // Offset to hotspot name (PString) in varBuf
    sint16 scriptTextOfst;     // Offset to script source code (CString) in varBuf
    sint16 alignReserved;      // Padding for alignment
}
```

**Key fields for scripting:**
- `scriptEventMask`: Determines which Palace events trigger this script
- `scriptTextOfst`: Points to the actual Iptscrae source code string
- `nbrScripts`: Number of scripts (typically 1 per hotspot)

### 3. Script Event Mask Encoding

The `scriptEventMask` is a 32-bit bitmask where each bit represents a Palace event. When an event occurs, the server checks if the corresponding bit is set, and if so, executes the script.

| Event Name | Bit Value | Iptscrae Handler |
|------------|-----------|------------------|
| PE_Select | 0x00000001 | ON SELECT |
| PE_Lock | 0x00000002 | ON LOCK |
| PE_Unlock | 0x00000004 | ON UNLOCK |
| PE_Hide | 0x00000008 | (internal) |
| PE_Show | 0x00000010 | (internal) |
| PE_Startup | 0x00000020 | ON STARTUP |
| PE_Alarm | 0x00000040 | ON ALARM |
| PE_Custom | 0x00000080 | (custom events) |
| PE_InChat | 0x00000100 | ON INCHAT |
| PE_PropChange | 0x00000200 | (internal) |
| PE_Enter | 0x00000400 | ON ENTER |
| PE_Leave | 0x00000800 | ON LEAVE |
| PE_OutChat | 0x00001000 | ON OUTCHAT |
| PE_SignOn | 0x00002000 | ON SIGNON |
| PE_SignOff | 0x00004000 | (internal) |
| PE_Macro0 | 0x00008000 | ON MACRO0 |
| PE_Macro1 | 0x00010000 | ON MACRO1 |
| PE_Macro2 | 0x00020000 | ON MACRO2 |
| PE_Macro3 | 0x00040000 | ON MACRO3 |
| PE_Macro4 | 0x00080000 | ON MACRO4 |
| PE_Macro5 | 0x00100000 | ON MACRO5 |
| PE_Macro6 | 0x00200000 | ON MACRO6 |
| PE_Macro7 | 0x00400000 | ON MACRO7 |
| PE_Macro8 | 0x00800000 | ON MACRO8 |
| PE_Macro9 | 0x01000000 | ON MACRO9 |

**Client-side events** (Priority for cyborg.ipt implementation):
- `PE_OutChat` (0x00001000): User sends a chat message
- `PE_InChat` (0x00000100): User receives a chat message
- `PE_Enter` (0x00000400): User enters a room
- `PE_Leave` (0x00000800): User leaves a room
- `PE_SignOn` (0x00002000): User connects to server
- `PE_Alarm` (0x00000040): Timer event
- `PE_Macro0-9` (0x00008000-0x01000000): User-triggered macros

**Server-side events** (Skip for client-only implementation):
- `PE_Select` (0x00000001): User clicks hotspot
- `PE_Lock/Unlock` (0x00000002/0x00000004): Door locking

### 4. Room Description Message (MSG_ROOMDESC)

When entering a room, the server sends MSG_ROOMDESC containing the `RoomRec` structure:

```c
struct RoomRec {
    sint32 roomFlags;          // Room attributes (see below)
    sint32 facesID;            // Default avatar appearance
    sint16 roomID;             // Room ID number
    sint16 roomNameOfst;       // Offset to room name (PString) in varBuf
    sint16 pictNameOfst;       // Offset to background picture filename in varBuf
    sint16 artistNameOfst;     // Offset to artist name in varBuf
    sint16 passwordOfst;       // Offset to room password in varBuf
    sint16 nbrHotspots;        // Number of hotspots in room
    sint16 hotspotOfst;        // Offset to Hotspot array in varBuf (4-byte aligned)
    sint16 nbrPictures;        // Number of pictures in room
    sint16 pictureOfst;        // Offset to PictureRec array in varBuf
    sint16 nbrDrawCmds;        // Number of drawing commands
    sint16 firstDrawCmd;       // Index of first draw command
    sint16 nbrPeople;          // Number of people in room
    sint16 nbrLProps;          // Number of loose props
    sint16 firstLProp;         // Index of first loose prop
    sint16 reserved;           // Reserved field
    sint16 lenVars;            // Length of varBuf array
    uint8 varBuf[lenVars];     // Variable-length data (strings, arrays, etc.)
}
```

**Important:** The `varBuf` array contains all variable-length data. Offsets in the structure (like `hotspotOfst`, `scriptTextOfst`) are indices into this array.

### 5. Cyborg Permission Flags

The protocol defines flags controlling whether client-side scripts (cyborgs) are allowed:

#### Room Flags (in RoomRec.roomFlags)
- `RF_CyborgFreeZone` (0x0010): Client must disable cyborg.ipt scripts in this room

#### Server Permission Flags (in server configuration)
- `PM_AllowCyborgs` (0x0002): Server allows client-side cyborg scripts
- `PM_CyborgsMayKill` (0x0100): Cyborgs can disconnect other users

**Implementation note:** Client should check `RF_CyborgFreeZone` when entering a room and disable script execution if flag is set.

### 6. User Status Flags

User flags are transmitted via MSG_USERSTATUS and MSG_LISTOFALLUSERS:

```c
struct UserListRec {
    UserID userID;
    sint16 flags;      // User status flags (see below)
    RoomID roomID;
    PString name;      // Padded to align length
}
```

User flags:
- `U_SuperUser` (0x0001): Wizard/admin
- `U_God` (0x0002): God mode (immortal)
- `U_Kill` (0x0004): Can kill/disconnect users
- `U_Guest` (0x0008): Guest user (unregistered)
- `U_Banished` (0x0010): Banned from server
- `U_Penalized` (0x0020): Restricted permissions
- `U_CommError`: Communication error flag
- `U_Gag`: Cannot speak
- `U_Pin`: Cannot move

**Note:** There's no explicit "U_Cyborg" flag. Cyborg status is determined by whether the user is running a cyborg.ipt script, not a protocol flag.

### 7. Asset System (For Future Consideration)

The protocol includes an asset system for transmitting large data like props and potentially scripts:

```c
struct AssetDescriptor {
    uint32 flags;
    uint32 size;
    Str31 name;  // Pascal string: 1 byte length + up to 31 chars
}

// MSG_ASSETQUERY - request asset from server/client
struct {
    AssetType type;  // 4-char ASCII code (e.g., 'RT_PROP')
    uint32 id;
    uint32 crc;      // 0 = don't care
}

// MSG_ASSETSEND - send asset data
struct {
    AssetType type;
    AssetSpec spec;
    uint32 blockSize;
    sint32 blockOffset;
    sint16 blockNbr;
    sint16 nbrBlocks;
    AssetDescriptor desc;  // Only present if blockNbr == 0
    uint8 data[blockSize];
}
```

**Current finding:** Hotspot scripts are transmitted inline in MSG_ROOMDESC, not via the asset system. However, the asset system could be used for:
- Precompiled script bytecode (future optimization)
- Shared script libraries
- User-uploaded cyborg.ipt files

### 8. Event Triggering Flow

**Server-side hotspot scripts:**
1. User enters room → Server sends MSG_ROOMDESC with all hotspots
2. Client parses hotspots and stores scripts locally
3. User performs action (clicks, chats, moves) → Client or server checks `scriptEventMask`
4. If bit is set for that event → Execute corresponding Iptscrae script
5. Script runs and may send commands back to server (SAY, GOTOROOM, etc.)

**Client-side cyborg.ipt scripts:**
1. User connects → Client loads `~/.config/Palace/cyborg.ipt`
2. Client parses script and registers event handlers (ON OUTCHAT, ON INCHAT, etc.)
3. Client events occur (chat received, room entered) → Client triggers corresponding handlers
4. Script executes and may modify CHATSTR or send new messages

**Key difference:** Cyborg scripts run entirely on the client and are triggered by client-side events. Hotspot scripts are sent by the server and may run on either client or server depending on the event type.

## Implementation Roadmap

### Phase 1: Protocol Support (Add to Protocol.h/cpp)

**New structures to add:**

```cpp
// Event mask bit flags
enum class PalaceEvent : uint32_t {
    Select      = 0x00000001,
    Lock        = 0x00000002,
    Unlock      = 0x00000004,
    Hide        = 0x00000008,
    Show        = 0x00000010,
    Startup     = 0x00000020,
    Alarm       = 0x00000040,
    Custom      = 0x00000080,
    InChat      = 0x00000100,
    PropChange  = 0x00000200,
    Enter       = 0x00000400,
    Leave       = 0x00000800,
    OutChat     = 0x00001000,
    SignOn      = 0x00002000,
    SignOff     = 0x00004000,
    Macro0      = 0x00008000,
    Macro1      = 0x00010000,
    Macro2      = 0x00020000,
    Macro3      = 0x00040000,
    Macro4      = 0x00080000,
    Macro5      = 0x00100000,
    Macro6      = 0x00200000,
    Macro7      = 0x00400000,
    Macro8      = 0x00800000,
    Macro9      = 0x01000000
};

// Room flags
enum class RoomFlag : uint32_t {
    Closed          = 0x0008,
    CyborgFreeZone  = 0x0010,  // Disable cyborg.ipt in this room
    Hidden          = 0x0020
};

struct Point {
    int16_t h;  // horizontal (x)
    int16_t v;  // vertical (y)
};

struct Hotspot {
    uint32_t scriptEventMask;  // Bitmask of PalaceEvent flags
    uint32_t flags;
    uint32_t secureInfo;
    uint32_t refCon;
    Point loc;
    int16_t id;
    int16_t dest;
    int16_t nbrPts;
    int16_t ptsOfst;
    int16_t type;
    int16_t groupID;
    int16_t nbrScripts;
    int16_t scriptRecOfst;
    int16_t state;
    int16_t nbrStates;
    int16_t stateRecOfst;
    int16_t nameOfst;
    int16_t scriptTextOfst;
    int16_t alignReserved;
    
    // Parsed data (not in wire format)
    QString name;
    QString scriptText;
    QList<Point> points;
};

struct RoomInfo {
    uint32_t roomFlags;      // RoomFlag bitmask
    uint32_t facesID;
    int16_t roomId;
    QString name;
    QString pictName;
    QString artistName;
    QList<Hotspot> hotspots;
    // ... other fields
};
```

**New parsing methods:**

```cpp
// In Protocol class
static Hotspot parseHotspot(const QByteArray& varBuf, int offset);
static QString parseScript(const QByteArray& varBuf, int scriptTextOfst);
static bool roomHasCyborgFreeZone(uint32_t roomFlags);
```

### Phase 2: Session Integration

**Modify Session.h/cpp to:**

1. Store current room's hotspots and check for `RF_CyborgFreeZone`
2. Trigger Iptscrae events when protocol messages arrive:
   - `MSG_TALK` received → Trigger `PE_InChat` event
   - `MSG_TALK` sent → Trigger `PE_OutChat` event (before sending)
   - `MSG_ROOMDESC` received → Trigger `PE_Enter` event
   - Leaving room → Trigger `PE_Leave` event
   - Successful logon → Trigger `PE_SignOn` event

3. Expose Session methods to Iptscrae engine:
   - `sendChat(QString)` → SAY command
   - `sendEmote(QString)` → ME command
   - `goToRoom(int16_t)` → GOTOROOM command
   - `getUserName()` → WHOAMI command
   - `getUserId()` → WHOME command

### Phase 3: Iptscrae Engine Core

**Create client/src/scripting/ directory with:**

1. `IptscriptValue.h/cpp` - Type system (Integer, String, Symbol, Array, Atomlist)
2. `IptscriptStack.h/cpp` - Stack operations for RPN execution
3. `IptscriptLexer.h/cpp` - Tokenize Iptscrae source
4. `IptscriptParser.h/cpp` - Build AST from tokens, extract event handlers
5. `IptscriptEngine.h/cpp` - Main execution engine with safety limits
6. `IptscriptCommands.h/cpp` - Command implementations (SAY, ME, CHATSTR, etc.)
7. `IptscriptEvents.h/cpp` - Event dispatcher

**Engine integration with Session:**

```cpp
class Session {
    // ...
    IptscriptEngine* m_scriptEngine;
    
    void handleIncomingChat(const ChatMessage& msg) {
        // Trigger PE_InChat event in cyborg.ipt
        m_scriptEngine->triggerEvent(PalaceEvent::InChat, msg.text);
        
        // Then display in UI
        emit chatReceived(msg);
    }
    
    void sendChat(const QString& text) {
        QString modifiedText = text;
        
        // Trigger PE_OutChat event (may modify CHATSTR)
        if (!m_currentRoom.hasCyborgFreeZone()) {
            modifiedText = m_scriptEngine->triggerEvent(PalaceEvent::OutChat, text);
        }
        
        // Send modified text to server
        QByteArray msg = Protocol::buildTalk(modifiedText);
        m_connection->send(msg);
    }
};
```

### Phase 4: Cyborg.ipt File Loading

**Implementation:**

1. On startup, check for `~/.config/Palace/cyborg.ipt`
2. If exists, load and parse with `IptscriptEngine::loadScript(filePath)`
3. Extract event handlers (ON OUTCHAT, ON INCHAT, etc.)
4. Register handlers with event dispatcher
5. Display load status in console ("Cyborg.ipt loaded successfully" or errors)

**File locations (cross-platform):**
- Linux: `~/.config/Palace/cyborg.ipt`
- macOS: `~/Library/Application Support/Palace/cyborg.ipt`
- Windows: `%APPDATA%\Palace\cyborg.ipt`

### Phase 5: Safety & Sandboxing

**Required limits:**

1. **Instruction counter**: Max 10,000 operations per event handler
2. **Execution timeout**: 500ms max per handler
3. **Stack depth**: Max 256 items
4. **Recursion depth**: Max 32 levels for atomlist calls
5. **String length**: Max 4096 chars
6. **Command whitelist**: Only allow safe client-side commands

**Blocked operations:**
- File system access (no file I/O beyond cyborg.ipt)
- Network requests (beyond existing Palace protocol commands)
- System commands
- Server-side commands (SETROOMNAME, KILLUSER, etc.)

### Phase 6: Testing Strategy

**Test scripts:**

1. **Basic echo test:**
```iptscrae
ON OUTCHAT {
    "You said: " CHATSTR & SAY
}
```

2. **Auto-greeter:**
```iptscrae
ON ENTER {
    "Hello from " ROOMNAME & SAY
}
```

3. **Chat filter:**
```iptscrae
ON INCHAT {
    CHATSTR "coffee" INSTR IF {
        "coffee.wav" SOUND
    }
}
```

4. **Stack operations:**
```iptscrae
ON SIGNON {
    2 3 + ITOA " is the answer!" & SAY
}
```

## Open Questions

1. **Hotspot click handling**: Do clients send MSG_SPOTSTATE to server, or does server detect clicks? 
   - *Answer needed for PE_Select event*

2. **Asset-based scripts**: Can scripts be transmitted via MSG_ASSETSEND instead of inline in MSG_ROOMDESC?
   - *Not mentioned in protocol doc, likely no*

3. **Script compilation**: Does the original Palace client compile scripts to bytecode?
   - *Unknown - we'll interpret directly for simplicity*

4. **Multi-user script coordination**: How do hotspot scripts running on multiple clients stay synchronized?
   - *Likely server-authoritative, clients just render results*

## References

- **Palace Protocol Reference PDF** - Complete protocol specification
- **Iptscrae Language Guide PDF** - Complete language reference (5,533 lines)
- **server/src/db/models.rs** - Database schema with `script_event_mask` and `script_text`
- **server/src/config.rs** - Server configuration with `allow_cyborgs` flag

## Next Steps

1. ✅ **Protocol analysis complete** (this document)
2. **Extend Protocol.h/cpp** with Hotspot, RoomFlag, PalaceEvent structures
3. **Implement RoomDesc parsing** to extract hotspots and scripts from varBuf
4. **Update Session.cpp** to store room hotspots and check RF_CyborgFreeZone
5. **Begin Iptscrae engine implementation** (Phase 1: Core infrastructure)

---

**Document Status:** Complete - Protocol analysis finished, ready to begin implementation.
**Last Updated:** 2026-02-19
**Author:** OpenCode AI Assistant
