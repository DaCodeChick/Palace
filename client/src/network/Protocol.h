#pragma once

#include <QByteArray>
#include <QString>
#include <QList>
#include <cstdint>

namespace Palace {
namespace Network {

// Palace protocol message types (complete specification)
enum class MessageType : uint32_t {
    // Authentication & Connection (Server ↔ Client)
    TIYID           = 0x74697972,  // 'tiyr' - Server sends client its UserID
    LOGON           = 0x72656769,  // 'regi' - Client requests login
    LOGOFF          = 0x62796520,  // 'bye ' - Client disconnects
    AUTHENTICATE    = 0x72796974,  // 'ryit' - Authentication challenge/response
    SERVERINFO      = 0x73696e66,  // 'sinf' - Server information
    VERSION         = 0x76657273,  // 'vers' - Server version
    SUPERUSER       = 0x73757372,  // 'susr' - Superuser/wizard status
    
    // Room Navigation (Server ↔ Client)
    ROOMGOTO        = 0x6e617652,  // 'navR' - Navigate to room
    ROOMDESC        = 0x726f6f6d,  // 'room' - Room description
    ROOMDESCEND     = 0x656e6472,  // 'endr' - End of room description
    ROOMNEW         = 0x6e526f6d,  // 'nRom' - New room created
    ROOMSETDESC     = 0x73526f6d,  // 'sRom' - Set/update room description
    LISTOFALLROOMS  = 0x724c7374,  // 'rLst' - List all rooms
    NAVERROR        = 0x73457272,  // 'sErr' - Navigation error
    
    // User Management (Server ↔ Client)
    USERLIST        = 0x72707273,  // 'rprs' - List of users in room
    USERNEW         = 0x6e707273,  // 'nprs' - New user entered room
    USEREXIT        = 0x65707273,  // 'eprs' - User left room
    USERMOVE        = 0x754c6f63,  // 'uLoc' - User moved position
    USERNAME        = 0x7573724e,  // 'usrN' - User changed name
    USERCOLOR       = 0x75737243,  // 'usrC' - User changed color
    USERFACE        = 0x75737246,  // 'usrF' - User changed face
    USERPROP        = 0x75737250,  // 'usrP' - User changed props
    USERSTATUS      = 0x75537461,  // 'uSta' - User status update
    USERDESC        = 0x75737244,  // 'usrD' - User description
    USERLOG         = 0x6c6f6720,  // 'log ' - User log message
    LISTOFALLUSERS  = 0x754c7374,  // 'uLst' - List all users on server
    
    // Chat & Communication (Server ↔ Client)
    TALK            = 0x74616c6b,  // 'talk' - Normal chat message
    XTALK           = 0x78746c6b,  // 'xtlk' - Extended/encrypted chat
    WHISPER         = 0x77686973,  // 'whis' - Private message (whisper)
    XWHISPER        = 0x78776973,  // 'xwis' - Extended whisper
    GMSG            = 0x676d7367,  // 'gmsg' - Global message
    SMSG            = 0x736d7367,  // 'smsg' - Server message
    RMSG            = 0x726d7367,  // 'rmsg' - Room message
    WMSG            = 0x776d7367,  // 'wmsg' - Wizard message
    
    // Props & Assets (Server ↔ Client)
    PROPNEW         = 0x6e507270,  // 'nPrp' - New prop created
    PROPDEL         = 0x64507270,  // 'dPrp' - Prop deleted
    PROPMOVE        = 0x6d507270,  // 'mPrp' - Prop moved
    ASSETQUERY      = 0x71417374,  // 'qAst' - Query for asset
    ASSETSEND       = 0x73417374,  // 'sAst' - Send asset data
    
    // Hotspots & Pictures (Server ↔ Client)
    SPOTNEW         = 0x6f70536e,  // 'opSn' - New hotspot
    SPOTDEL         = 0x6f705364,  // 'opSd' - Delete hotspot
    SPOTMOVE        = 0x636f4c73,  // 'coLs' - Move hotspot
    SPOTSTATE       = 0x73537461,  // 'sSta' - Change hotspot state
    SPOTSETDESC     = 0x6f705373,  // 'opSs' - Update hotspot description
    PICTNEW         = 0x6e506374,  // 'nPct' - New picture
    PICTMOVE        = 0x704c6f63,  // 'pLoc' - Move picture
    PICTSETDESC     = 0x73506374,  // 'sPct' - Update picture description
    DRAW            = 0x64726177,  // 'draw' - Drawing commands
    
    // Files & URLs (Server ↔ Client)
    FILEQUERY       = 0x7146696c,  // 'qFil' - Query for file
    FILESEND        = 0x7346696c,  // 'sFil' - Send file data
    FILENOTFND      = 0x666e6665,  // 'fnfe' - File not found
    DISPLAYURL      = 0x6475726c,  // 'durl' - Display URL in browser
    HTTPSERVER      = 0x48545450,  // 'HTTP' - HTTP server info
    
    // Doors & Security (Server ↔ Client)
    DOORLOCK        = 0x6c6f636b,  // 'lock' - Lock door
    DOORUNLOCK      = 0x756e6c6f,  // 'unlo' - Unlock door
    KILLUSER        = 0x6b696c6c,  // 'kill' - Disconnect/ban user
    
    // Keepalive & Misc (Server ↔ Client)
    PING            = 0x70696e67,  // 'ping' - Keepalive ping
    PONG            = 0x706f6e67,  // 'pong' - Keepalive response
    NOOP            = 0x4e4f4f50,  // 'NOOP' - No operation
    BLOWTHRU        = 0x626c6f77,  // 'blow' - Pass-through message
    SERVERDOWN      = 0x646f776e,  // 'down' - Server shutting down
    EXTENDEDINFO    = 0x73496e66,  // 'sInf' - Extended info
};

// Palace protocol header structure
struct ProtocolHeader {
    uint32_t eventType;  // Message type (big-endian)
    uint32_t length;     // Payload length (big-endian)
    uint32_t refNum;     // Reference number (big-endian)
};

// Basic geometric types
struct Point {
    int16_t h;  // horizontal (x coordinate)
    int16_t v;  // vertical (y coordinate)
};

struct Rect {
    int16_t top;
    int16_t left;
    int16_t bottom;
    int16_t right;
};

// Asset specification
struct AssetSpec {
    uint32_t id;       // Asset ID
    uint32_t crc;      // CRC checksum (0 = don't care)
};

// Prop specification (asset used by user)
struct PropSpec {
    AssetSpec spec;
    // Full implementation would include additional prop-specific data
};

// Room flags
enum class RoomFlag : uint32_t {
    None            = 0x0000,
    Closed          = 0x0008,  // Room is closed
    CyborgFreeZone  = 0x0010,  // Disable cyborg.ipt scripts
    Hidden          = 0x0020,  // Room hidden from list
    // Additional flags as needed
};

// User flags
enum class UserFlag : uint16_t {
    None            = 0x0000,
    SuperUser       = 0x0001,  // Wizard/admin
    God             = 0x0002,  // God mode (immortal)
    Kill            = 0x0004,  // Can kill/disconnect users
    Guest           = 0x0008,  // Guest user (unregistered)
    Banished        = 0x0010,  // Banned from server
    Penalized       = 0x0020,  // Restricted permissions
    CommError       = 0x0040,  // Communication error
    Gag             = 0x0080,  // Cannot speak
    Pin             = 0x0100,  // Cannot move
};

// Hotspot types
enum class HotspotType : int16_t {
    Normal          = 0,  // Regular hotspot
    Door            = 1,  // Navigation door
    ShutableDoor    = 2,  // Door that can be locked
    Bolt            = 3,  // Lock for a door
    NavArea         = 4,  // Movement allowed area
    Hidden          = 5,  // Invisible hotspot
};

// Palace event flags (for script triggers)
enum class PalaceEvent : uint32_t {
    Select          = 0x00000001,  // User clicks hotspot
    Lock            = 0x00000002,  // Door locked
    Unlock          = 0x00000004,  // Door unlocked
    Hide            = 0x00000008,  // Hidden
    Show            = 0x00000010,  // Shown
    Startup         = 0x00000020,  // On startup
    Alarm           = 0x00000040,  // Timer event
    Custom          = 0x00000080,  // Custom event
    InChat          = 0x00000100,  // Chat received
    PropChange      = 0x00000200,  // Prop changed
    Enter           = 0x00000400,  // Enter room
    Leave           = 0x00000800,  // Leave room
    OutChat         = 0x00001000,  // Chat sent
    SignOn          = 0x00002000,  // User connected
    SignOff         = 0x00004000,  // User disconnected
    Macro0          = 0x00008000,  // Macro 0
    Macro1          = 0x00010000,  // Macro 1
    Macro2          = 0x00020000,  // Macro 2
    Macro3          = 0x00040000,  // Macro 3
    Macro4          = 0x00080000,  // Macro 4
    Macro5          = 0x00100000,  // Macro 5
    Macro6          = 0x00200000,  // Macro 6
    Macro7          = 0x00400000,  // Macro 7
    Macro8          = 0x00800000,  // Macro 8
    Macro9          = 0x01000000,  // Macro 9
};

// Hotspot state record (one picture associated with hotspot)
struct StateRec {
    int16_t pictID;     // Picture ID
    Point picLoc;       // Offset from hotspot location
};

// Hotspot structure (clickable screen area with optional script)
struct Hotspot {
    uint32_t scriptEventMask;  // Bitmask of PalaceEvent flags
    uint32_t flags;            // Hotspot flags
    uint32_t secureInfo;       // Unused
    uint32_t refCon;           // Unused
    Point loc;                 // Location of hotspot
    int16_t id;                // Hotspot ID
    int16_t dest;              // Room ID for doors, door ID for bolts
    int16_t nbrPts;            // Number of points in clickable zone
    int16_t ptsOfst;           // Offset to Point array in varBuf
    int16_t type;              // HotspotType
    int16_t groupID;           // Group identifier
    int16_t nbrScripts;        // Number of scripts
    int16_t scriptRecOfst;     // Offset to script descriptor
    int16_t state;             // Current state (picture index)
    int16_t nbrStates;         // Number of state pictures
    int16_t stateRecOfst;      // Offset to StateRec array
    int16_t nameOfst;          // Offset to name (PString)
    int16_t scriptTextOfst;    // Offset to script source (CString)
    int16_t alignReserved;     // Padding
    
    // Parsed data (not in wire format)
    QString name;
    QString scriptText;
    QList<Point> points;
    QList<StateRec> states;
};

// Picture record (background/overlay image)
struct PictureRec {
    int16_t refCon;      // Reference constant
    int16_t picID;       // Picture ID
    QString name;        // Picture name
};

// Drawing command (for room graphics)
struct DrawCmd {
    int16_t nextOfst;    // Offset to next command
    int16_t type;        // Drawing operation type
    int16_t cmdLength;   // Command data length
    int16_t dataOfst;    // Offset to command data
};

// Loose prop (prop placed in room)
struct LooseProp {
    int16_t nextOfst;    // Offset to next loose prop
    AssetSpec spec;      // Prop asset specification
    uint32_t flags;      // Prop flags
    uint32_t refCon;     // Reference constant
    Point loc;           // Location in room
};

// User information (complete structure)
struct UserInfo {
    uint32_t userId;
    QString name;
    int16_t roomId;
    Point roomPos;               // Position in room
    PropSpec propSpec[9];        // Up to 9 props
    int16_t faceNbr;             // Face number
    int16_t colorNbr;            // Color number
    uint16_t flags;              // UserFlag bitmask
    int16_t awayFlag;            // Away status
    int16_t openToMsgs;          // Open to messages
    int16_t nbrProps;            // Number of active props
};

// Room information (complete structure)
struct RoomInfo {
    uint32_t roomFlags;          // RoomFlag bitmask
    uint32_t facesID;            // Default avatar appearance
    int16_t roomId;
    QString name;
    QString pictName;            // Background picture filename
    QString artistName;          // Artist/creator name
    QString password;            // Room password (if any)
    int16_t nbrHotspots;         // Number of hotspots
    int16_t nbrPictures;         // Number of pictures
    int16_t nbrDrawCmds;         // Number of drawing commands
    int16_t nbrPeople;           // Number of people in room
    int16_t nbrLProps;           // Number of loose props
    QList<Hotspot> hotspots;     // Hotspot list
    QList<PictureRec> pictures;  // Picture list
    QList<LooseProp> looseProps; // Loose prop list
};

// Chat message
struct ChatMessage {
    QString username;
    QString text;
    bool isWhisper;
};

class Protocol {
public:
    Protocol();
    ~Protocol();
    
    // Message parsing (Server → Client)
    static bool parseHeader(const QByteArray& data, ProtocolHeader& header);
    static MessageType identifyMessage(const QByteArray& data);
    
    // Authentication & Connection
    static uint32_t parseTiyid(const QByteArray& payload);  // Returns UserID
    static bool parseServerInfo(const QByteArray& payload);
    static uint32_t parseVersion(const QByteArray& payload);  // Returns version
    static bool parseSuperuser(const QByteArray& payload);
    static QString parseServerDown(const QByteArray& payload);  // Returns reason
    
    // Room parsing
    static RoomInfo parseRoomDesc(const QByteArray& payload);
    static bool parseRoomDescEnd(const QByteArray& payload);
    static QList<RoomInfo> parseRoomList(const QByteArray& payload);
    static QString parseNavError(const QByteArray& payload);  // Returns error message
    
    // User parsing
    static UserInfo parseUserNew(const QByteArray& payload);
    static uint32_t parseUserExit(const QByteArray& payload);  // Returns UserID
    static QList<UserInfo> parseUserList(const QByteArray& payload);
    static bool parseUserMove(const QByteArray& payload, uint32_t& userId, Point& pos);
    static bool parseUserName(const QByteArray& payload, uint32_t& userId, QString& name);
    static bool parseUserColor(const QByteArray& payload, uint32_t& userId, int16_t& color);
    static bool parseUserFace(const QByteArray& payload, uint32_t& userId, int16_t& face);
    static bool parseUserProp(const QByteArray& payload, uint32_t& userId, QList<PropSpec>& props);
    static bool parseUserStatus(const QByteArray& payload, uint32_t& userId, uint16_t& flags);
    
    // Chat parsing
    static ChatMessage parseTalk(const QByteArray& payload);
    static ChatMessage parseXTalk(const QByteArray& payload);
    static ChatMessage parseWhisper(const QByteArray& payload);
    static ChatMessage parseXWhisper(const QByteArray& payload);
    static QString parseGlobalMsg(const QByteArray& payload);
    static QString parseRoomMsg(const QByteArray& payload);
    
    // Prop/Asset parsing
    static bool parsePropNew(const QByteArray& payload);
    static bool parsePropDel(const QByteArray& payload);
    static bool parsePropMove(const QByteArray& payload);
    
    // Hotspot/Picture parsing
    static Hotspot parseSpotNew(const QByteArray& payload);
    static uint16_t parseSpotDel(const QByteArray& payload);  // Returns hotspot ID
    static bool parseSpotMove(const QByteArray& payload);
    static bool parseSpotState(const QByteArray& payload, uint16_t& spotId, int16_t& state);
    
    // File/URL parsing
    static QString parseDisplayUrl(const QByteArray& payload);
    static bool parseFileNotFound(const QByteArray& payload);
    
    // Ping/Pong
    static bool parsePing(const QByteArray& payload);
    static bool parsePong(const QByteArray& payload);
    
    // Message serialization (Client → Server)
    static QByteArray buildLogon(const QString& username, const QString& wizardPassword = QString());
    static QByteArray buildLogoff();
    
    // Room navigation
    static QByteArray buildRoomGoto(int16_t roomId);
    static QByteArray buildListRooms();
    
    // User actions
    static QByteArray buildUserMove(const Point& pos);
    static QByteArray buildUserName(const QString& name);
    static QByteArray buildUserColor(int16_t color);
    static QByteArray buildUserFace(int16_t face);
    static QByteArray buildUserProp(const QList<PropSpec>& props);
    
    // Chat
    static QByteArray buildTalk(const QString& text);
    static QByteArray buildXTalk(const QString& text);
    static QByteArray buildWhisper(uint32_t targetUserId, const QString& text);
    static QByteArray buildGlobalMsg(const QString& text);
    
    // Keepalive
    static QByteArray buildPing();
    static QByteArray buildPong();
    
    // Hotspot/Door actions
    static QByteArray buildSpotState(uint16_t spotId, int16_t state);
    static QByteArray buildDoorLock(uint16_t spotId);
    static QByteArray buildDoorUnlock(uint16_t spotId);
    
private:
    // String helpers
    static QString readPascalString(const QByteArray& data, int& offset);
    static void writePascalString(QByteArray& data, const QString& str);
    static QString readCString(const QByteArray& data, int offset, int maxLen = 255);
    
    // Helper to parse hotspot from room description varBuf
    static Hotspot parseHotspot(const QByteArray& varBuf, int offset, int varBufStart);
    
    // Helper to build message with header
    static QByteArray buildMessage(MessageType type, const QByteArray& payload, uint32_t refNum = 0);
    
    // Native byte order helpers - no endian conversion (server handles detection)
    static inline void appendU32(QByteArray& data, uint32_t value) {
        data.append(reinterpret_cast<const char*>(&value), sizeof(value));
    }
    
    static inline void appendU16(QByteArray& data, uint16_t value) {
        data.append(reinterpret_cast<const char*>(&value), sizeof(value));
    }
    
    static inline void appendI32(QByteArray& data, int32_t value) {
        data.append(reinterpret_cast<const char*>(&value), sizeof(value));
    }
    
    static inline void appendI16(QByteArray& data, int16_t value) {
        data.append(reinterpret_cast<const char*>(&value), sizeof(value));
    }
    
    static inline uint32_t readU32(const QByteArray& data, int offset) {
        if (offset + 4 > data.size()) return 0;
        return *reinterpret_cast<const uint32_t*>(data.constData() + offset);
    }
    
    static inline uint16_t readU16(const QByteArray& data, int offset) {
        if (offset + 2 > data.size()) return 0;
        return *reinterpret_cast<const uint16_t*>(data.constData() + offset);
    }
    
    static inline int32_t readI32(const QByteArray& data, int offset) {
        if (offset + 4 > data.size()) return 0;
        return *reinterpret_cast<const int32_t*>(data.constData() + offset);
    }
    
    static inline int16_t readI16(const QByteArray& data, int offset) {
        if (offset + 2 > data.size()) return 0;
        return *reinterpret_cast<const int16_t*>(data.constData() + offset);
    }
};

} // namespace Network
} // namespace Palace
