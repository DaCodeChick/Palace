#pragma once

#include <QByteArray>
#include <QString>
#include <QList>
#include <cstdint>

namespace Palace {
namespace Network {

// Palace protocol message types (subset for MVP)
enum class MessageType : uint32_t {
    // Server → Client
    TIYID = 0x74697964,        // 'tiyd' - Server identification
    RPRS = 0x72707273,         // 'rprs' - Reply with server info
    SUSER = 0x73757372,        // 'susr' - Server user info
    ROOMDESC = 0x726f6f6d,     // 'room' - Room description
    USERLIST = 0x75706e73,     // 'upns' - User list
    USERNEW = 0x6e707273,      // 'nprs' - New user entering
    USERLEFT = 0x65707273,     // 'eprs' - User leaving
    TALK = 0x74616c6b,         // 'talk' - Chat message
    XTALK = 0x7874616c,        // 'xtal' - Encrypted chat
    PING = 0x70696e67,         // 'ping' - Keepalive
    PONG = 0x706f6e67,         // 'pong' - Keepalive response
    RMLIST = 0x724c7374,       // 'rLst' - Room list
    
    // Client → Server
    LOGON = 0x72656769,        // 'regi' - User login
    TALK_CLIENT = 0x74616c6b,  // 'talk' - Send chat
    XTALK_CLIENT = 0x7874616c, // 'xtal' - Send encrypted chat
    PING_CLIENT = 0x70696e67,  // 'ping' - Send keepalive
    ROOMGOTO = 0x73474f54,     // 'sGOT' - Navigate to room
    LISTROOMS = 0x724c7374,    // 'rLst' - Request room list
};

// Palace protocol header structure
struct ProtocolHeader {
    uint32_t eventType;  // Message type (big-endian)
    uint32_t length;     // Payload length (big-endian)
    uint32_t refNum;     // Reference number (big-endian)
};

// User information
struct UserInfo {
    uint32_t userId;
    QString name;
    int16_t roomId;
    int16_t x;
    int16_t y;
    uint16_t faceColor;
    // Simplified for MVP - full implementation needs props, flags, etc.
};

// Room information
struct RoomInfo {
    int16_t roomId;
    QString name;
    uint16_t userCount;
    // Simplified for MVP - full implementation needs flags, hotspots, props, etc.
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
    
    // Parse specific message types
    static bool parseTiyid(const QByteArray& payload);
    static bool parseServerInfo(const QByteArray& payload);
    static UserInfo parseUserNew(const QByteArray& payload);
    static bool parseUserLeft(const QByteArray& payload, uint32_t& userId);
    static QList<UserInfo> parseUserList(const QByteArray& payload);
    static RoomInfo parseRoomDesc(const QByteArray& payload);
    static QList<RoomInfo> parseRoomList(const QByteArray& payload);
    static ChatMessage parseTalk(const QByteArray& payload);
    static ChatMessage parseXTalk(const QByteArray& payload);
    
    // Message serialization (Client → Server)
    static QByteArray buildLogon(const QString& username, const QString& wizardPassword);
    static QByteArray buildTalk(const QString& text);
    static QByteArray buildXTalk(const QString& text);
    static QByteArray buildRoomGoto(int16_t roomId);
    static QByteArray buildListRooms();
    static QByteArray buildPing();
    
private:
    // Helper functions for byte order conversion
    static uint32_t readBigEndianU32(const QByteArray& data, int offset);
    static uint16_t readBigEndianU16(const QByteArray& data, int offset);
    static int32_t readBigEndianI32(const QByteArray& data, int offset);
    static int16_t readBigEndianI16(const QByteArray& data, int offset);
    
    static void writeBigEndianU32(QByteArray& data, uint32_t value);
    static void writeBigEndianU16(QByteArray& data, uint16_t value);
    static void writeBigEndianI32(QByteArray& data, int32_t value);
    static void writeBigEndianI16(QByteArray& data, int16_t value);
    
    // Helper to read Pascal strings (1-byte length prefix + string)
    static QString readPascalString(const QByteArray& data, int& offset);
    static void writePascalString(QByteArray& data, const QString& str);
    
    // Helper to read C strings (null-terminated)
    static QString readCString(const QByteArray& data, int offset, int maxLen);
};

} // namespace Network
} // namespace Palace
