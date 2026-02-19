#include "Protocol.h"
#include <QDebug>

namespace Palace {
namespace Network {

Protocol::Protocol() = default;
Protocol::~Protocol() = default;

// === String Helpers ===

QString Protocol::readPascalString(const QByteArray& data, int& offset) {
    if (offset >= data.size()) return QString();
    
    uint8_t len = static_cast<uint8_t>(data[offset]);
    offset++;
    
    if (offset + len > data.size()) return QString();
    
    QString result = QString::fromLatin1(data.constData() + offset, len);
    offset += len;
    
    return result;
}

void Protocol::writePascalString(QByteArray& data, const QString& str) {
    QByteArray latin1 = str.toLatin1();
    uint8_t len = static_cast<uint8_t>(qMin(latin1.size(), 255));
    data.append(static_cast<char>(len));
    data.append(latin1.left(len));
}

QString Protocol::readCString(const QByteArray& data, int offset, int maxLen) {
    if (offset >= data.size()) return QString();
    
    int endPos = data.indexOf('\0', offset);
    if (endPos == -1) {
        endPos = qMin(offset + maxLen, data.size());
    }
    
    return QString::fromLatin1(data.constData() + offset, endPos - offset);
}

// === Message Parsing ===

bool Protocol::parseHeader(const QByteArray& data, ProtocolHeader& header) {
    if (data.size() < 12) {
        qWarning() << "Protocol::parseHeader: Data too short for header";
        return false;
    }
    
    header.eventType = readU32(data, 0);
    header.length = readU32(data, 4);
    header.refNum = readU32(data, 8);
    
    qDebug() << "Protocol::parseHeader: type=" << Qt::hex << header.eventType
             << "len=" << Qt::dec << header.length 
             << "ref=" << header.refNum;
    
    return true;
}

MessageType Protocol::identifyMessage(const QByteArray& data) {
    if (data.size() < 4) return static_cast<MessageType>(0);
    uint32_t type = readU32(data, 0);
    return static_cast<MessageType>(type);
}

uint32_t Protocol::parseTiyid(const QByteArray& payload) {
    // TIYID message contains UserID assigned by server
    if (payload.size() < 4) {
        qWarning() << "Protocol::parseTiyid: Payload too short";
        return 0;
    }
    
    uint32_t userId = readU32(payload, 0);
    qDebug() << "Protocol::parseTiyid: Received UserID =" << userId;
    return userId;
}

bool Protocol::parseServerInfo(const QByteArray& payload) {
    // Server info contains capabilities, permissions, etc.
    // For MVP, we just acknowledge receipt
    qDebug() << "Protocol::parseServerInfo: Received server info (" << payload.size() << "bytes)";
    return true;
}

UserInfo Protocol::parseUserNew(const QByteArray& payload) {
    UserInfo user = {};
    
    if (payload.size() < 12) {
        qWarning() << "Protocol::parseUserNew: Payload too short";
        return user;
    }
    
    int offset = 0;
    user.userId = readU32(payload, offset); offset += 4;
    user.roomPos.h = readI16(payload, offset); offset += 2;
    user.roomPos.v = readI16(payload, offset); offset += 2;
    user.roomId = readI16(payload, offset); offset += 2;
    user.faceNbr = readI16(payload, offset); offset += 2;
    user.colorNbr = readI16(payload, offset); offset += 2;
    
    // Name is Pascal string
    user.name = readPascalString(payload, offset);
    
    qDebug() << "Protocol::parseUserNew: User" << user.userId << user.name 
             << "at (" << user.roomPos.h << "," << user.roomPos.v << ") in room" << user.roomId;
    
    return user;
}

uint32_t Protocol::parseUserExit(const QByteArray& payload) {
    if (payload.size() < 4) {
        qWarning() << "Protocol::parseUserExit: Payload too short";
        return 0;
    }
    
    uint32_t userId = readU32(payload, 0);
    qDebug() << "Protocol::parseUserExit: User" << userId << "left";
    return userId;
}

QList<UserInfo> Protocol::parseUserList(const QByteArray& payload) {
    QList<UserInfo> users;
    
    if (payload.size() < 4) {
        qWarning() << "Protocol::parseUserList: Payload too short";
        return users;
    }
    
    int offset = 0;
    uint32_t userCount = readU32(payload, offset); offset += 4;
    
    qDebug() << "Protocol::parseUserList: Parsing" << userCount << "users";
    
    for (uint32_t i = 0; i < userCount && offset < payload.size(); ++i) {
        UserInfo user = {};
        
        if (offset + 12 > payload.size()) break;
        
        user.userId = readU32(payload, offset); offset += 4;
        user.roomPos.h = readI16(payload, offset); offset += 2;
        user.roomPos.v = readI16(payload, offset); offset += 2;
        user.roomId = readI16(payload, offset); offset += 2;
        user.faceNbr = readI16(payload, offset); offset += 2;
        user.colorNbr = readI16(payload, offset); offset += 2;
        
        // Name is Pascal string
        user.name = readPascalString(payload, offset);
        
        users.append(user);
    }
    
    qDebug() << "Protocol::parseUserList: Parsed" << users.size() << "users";
    return users;
}
    
RoomInfo Protocol::parseRoomDesc(const QByteArray& payload) {
    RoomInfo room = {};
    
    if (payload.size() < 4) {
        qWarning() << "Protocol::parseRoomDesc: Payload too short";
        return room;
    }
    
    int offset = 0;
    room.roomFlags = readU32(payload, offset); offset += 4;
    room.facesID = readU32(payload, offset); offset += 4;
    room.roomId = readI16(payload, offset); offset += 2;
    
    // Room name is Pascal string
    room.name = readPascalString(payload, offset);
    
    qDebug() << "Protocol::parseRoomDesc: Room" << room.roomId << room.name;
    return room;
}

QList<RoomInfo> Protocol::parseRoomList(const QByteArray& payload) {
    QList<RoomInfo> rooms;
    
    if (payload.size() < 4) {
        qWarning() << "Protocol::parseRoomList: Payload too short";
        return rooms;
    }
    
    int offset = 0;
    uint32_t roomCount = readU32(payload, offset); offset += 4;
    
    qDebug() << "Protocol::parseRoomList: Parsing" << roomCount << "rooms";
    
    for (uint32_t i = 0; i < roomCount && offset < payload.size(); ++i) {
        RoomInfo room = {};
        
        if (offset + 2 > payload.size()) break;
        
        room.roomId = readI16(payload, offset); offset += 2;
        room.name = readPascalString(payload, offset);
        
        // User count follows name
        if (offset + 2 <= payload.size()) {
            room.nbrPeople = readI16(payload, offset); offset += 2;
        }
        
        rooms.append(room);
    }
    
    qDebug() << "Protocol::parseRoomList: Parsed" << rooms.size() << "rooms";
    return rooms;
}

ChatMessage Protocol::parseTalk(const QByteArray& payload) {
    ChatMessage msg;
    msg.isWhisper = false;
    
    if (payload.size() < 1) {
        qWarning() << "Protocol::parseTalk: Payload too short";
        return msg;
    }
    
    int offset = 0;
    msg.username = readPascalString(payload, offset);
    msg.text = readPascalString(payload, offset);
    
    qDebug() << "Protocol::parseTalk:" << msg.username << ":" << msg.text;
    return msg;
}

ChatMessage Protocol::parseXTalk(const QByteArray& payload) {
    // For MVP, treat encrypted chat same as regular
    // Full implementation needs RC4 decryption
    ChatMessage msg = parseTalk(payload);
    qDebug() << "Protocol::parseXTalk: (encrypted chat - MVP treats as plain)";
    return msg;
}

// === New Protocol Methods ===

uint32_t Protocol::parseVersion(const QByteArray& payload) {
    if (payload.size() < 4) {
        qWarning() << "Protocol::parseVersion: Payload too short";
        return 0;
    }
    
    uint32_t version = readU32(payload, 0);
    qDebug() << "Protocol::parseVersion: Server version =" << QString::number(version, 16);
    return version;
}

QString Protocol::parseServerDown(const QByteArray& payload) {
    // Server sends a reason string for shutdown
    QString reason;
    
    if (payload.size() > 0) {
        int offset = 0;
        reason = readPascalString(payload, offset);
    } else {
        reason = "Server shutting down";
    }
    
    qDebug() << "Protocol::parseServerDown:" << reason;
    return reason;
}

QString Protocol::parseNavError(const QByteArray& payload) {
    // Navigation error message
    QString errorMsg;
    
    if (payload.size() > 0) {
        int offset = 0;
        errorMsg = readPascalString(payload, offset);
    } else {
        errorMsg = "Navigation error";
    }
    
    qWarning() << "Protocol::parseNavError:" << errorMsg;
    return errorMsg;
}

bool Protocol::parseRoomDescEnd(const QByteArray& payload) {
    // ROOMDESCEND marks end of room transmission sequence
    // Typically has no payload
    qDebug() << "Protocol::parseRoomDescEnd: Room description complete";
    return true;
}

bool Protocol::parseUserMove(const QByteArray& payload, uint32_t& userId, Point& pos) {
    if (payload.size() < 8) {
        qWarning() << "Protocol::parseUserMove: Payload too short";
        return false;
    }
    
    int offset = 0;
    userId = readU32(payload, offset); offset += 4;
    pos.h = readI16(payload, offset); offset += 2;
    pos.v = readI16(payload, offset); offset += 2;
    
    qDebug() << "Protocol::parseUserMove: User" << userId << "moved to (" << pos.h << "," << pos.v << ")";
    return true;
}

bool Protocol::parseUserName(const QByteArray& payload, uint32_t& userId, QString& name) {
    if (payload.size() < 5) { // Min: 4 bytes userId + 1 byte string length
        qWarning() << "Protocol::parseUserName: Payload too short";
        return false;
    }
    
    int offset = 0;
    userId = readU32(payload, offset); offset += 4;
    name = readPascalString(payload, offset);
    
    qDebug() << "Protocol::parseUserName: User" << userId << "changed name to" << name;
    return true;
}

bool Protocol::parseUserColor(const QByteArray& payload, uint32_t& userId, int16_t& color) {
    if (payload.size() < 6) {
        qWarning() << "Protocol::parseUserColor: Payload too short";
        return false;
    }
    
    int offset = 0;
    userId = readU32(payload, offset); offset += 4;
    color = readI16(payload, offset); offset += 2;
    
    qDebug() << "Protocol::parseUserColor: User" << userId << "changed color to" << color;
    return true;
}

bool Protocol::parseUserFace(const QByteArray& payload, uint32_t& userId, int16_t& face) {
    if (payload.size() < 6) {
        qWarning() << "Protocol::parseUserFace: Payload too short";
        return false;
    }
    
    int offset = 0;
    userId = readU32(payload, offset); offset += 4;
    face = readI16(payload, offset); offset += 2;
    
    qDebug() << "Protocol::parseUserFace: User" << userId << "changed face to" << face;
    return true;
}

bool Protocol::parseUserProp(const QByteArray& payload, uint32_t& userId, QList<PropSpec>& props) {
    if (payload.size() < 4) {
        qWarning() << "Protocol::parseUserProp: Payload too short";
        return false;
    }
    
    int offset = 0;
    userId = readU32(payload, offset); offset += 4;
    
    // Read prop count if present
    if (offset + 2 <= payload.size()) {
        int16_t nbrProps = readI16(payload, offset); offset += 2;
        
        // Read each prop (AssetSpec = 8 bytes: id + crc)
        for (int i = 0; i < nbrProps && offset + 8 <= payload.size(); ++i) {
            PropSpec prop;
            prop.spec.id = readU32(payload, offset); offset += 4;
            prop.spec.crc = readU32(payload, offset); offset += 4;
            props.append(prop);
        }
    }
    
    qDebug() << "Protocol::parseUserProp: User" << userId << "has" << props.size() << "props";
    return true;
}

bool Protocol::parseUserStatus(const QByteArray& payload, uint32_t& userId, uint16_t& flags) {
    if (payload.size() < 6) {
        qWarning() << "Protocol::parseUserStatus: Payload too short";
        return false;
    }
    
    int offset = 0;
    userId = readU32(payload, offset); offset += 4;
    flags = readU16(payload, offset); offset += 2;
    
    qDebug() << "Protocol::parseUserStatus: User" << userId << "flags =" << QString::number(flags, 16);
    return true;
}

ChatMessage Protocol::parseWhisper(const QByteArray& payload) {
    ChatMessage msg;
    msg.isWhisper = true;
    
    if (payload.size() < 4) {
        qWarning() << "Protocol::parseWhisper: Payload too short";
        return msg;
    }
    
    int offset = 0;
    uint32_t userId = readU32(payload, offset); offset += 4;
    msg.text = readPascalString(payload, offset);
    
    qDebug() << "Protocol::parseWhisper: Whisper from user" << userId << ":" << msg.text;
    return msg;
}

ChatMessage Protocol::parseXWhisper(const QByteArray& payload) {
    // Full implementation needs RC4 decryption
    ChatMessage msg = parseWhisper(payload);
    qDebug() << "Protocol::parseXWhisper: (encrypted whisper - MVP treats as plain)";
    return msg;
}

QString Protocol::parseGlobalMsg(const QByteArray& payload) {
    QString msg;
    
    if (payload.size() > 0) {
        int offset = 0;
        msg = readPascalString(payload, offset);
    }
    
    qDebug() << "Protocol::parseGlobalMsg:" << msg;
    return msg;
}

QString Protocol::parseRoomMsg(const QByteArray& payload) {
    QString msg;
    
    if (payload.size() > 0) {
        int offset = 0;
        msg = readPascalString(payload, offset);
    }
    
    qDebug() << "Protocol::parseRoomMsg:" << msg;
    return msg;
}

bool Protocol::parsePropNew(const QByteArray& payload) {
    // Full implementation would parse prop creation
    qDebug() << "Protocol::parsePropNew: Prop created (not fully implemented)";
    return true;
}

bool Protocol::parsePropDel(const QByteArray& payload) {
    // Full implementation would parse prop deletion
    qDebug() << "Protocol::parsePropDel: Prop deleted (not fully implemented)";
    return true;
}

bool Protocol::parsePropMove(const QByteArray& payload) {
    // Full implementation would parse prop movement
    qDebug() << "Protocol::parsePropMove: Prop moved (not fully implemented)";
    return true;
}

Hotspot Protocol::parseSpotNew(const QByteArray& payload) {
    Hotspot spot;
    // Full implementation would parse complete hotspot structure
    qDebug() << "Protocol::parseSpotNew: Hotspot created (not fully implemented)";
    return spot;
}

uint16_t Protocol::parseSpotDel(const QByteArray& payload) {
    if (payload.size() < 2) {
        qWarning() << "Protocol::parseSpotDel: Payload too short";
        return 0;
    }
    
    uint16_t spotId = readU16(payload, 0);
    qDebug() << "Protocol::parseSpotDel: Hotspot" << spotId << "deleted";
    return spotId;
}

bool Protocol::parseSpotMove(const QByteArray& payload) {
    // Full implementation would parse hotspot movement
    qDebug() << "Protocol::parseSpotMove: Hotspot moved (not fully implemented)";
    return true;
}

bool Protocol::parseSpotState(const QByteArray& payload, uint16_t& spotId, int16_t& state) {
    if (payload.size() < 4) {
        qWarning() << "Protocol::parseSpotState: Payload too short";
        return false;
    }
    
    int offset = 0;
    spotId = readU16(payload, offset); offset += 2;
    state = readI16(payload, offset); offset += 2;
    
    qDebug() << "Protocol::parseSpotState: Hotspot" << spotId << "state =" << state;
    return true;
}

QString Protocol::parseDisplayUrl(const QByteArray& payload) {
    QString url;
    
    if (payload.size() > 0) {
        int offset = 0;
        url = readPascalString(payload, offset);
    }
    
    qDebug() << "Protocol::parseDisplayUrl:" << url;
    return url;
}

bool Protocol::parseFileNotFound(const QByteArray& payload) {
    qDebug() << "Protocol::parseFileNotFound: Requested file not found on server";
    return true;
}

bool Protocol::parsePing(const QByteArray& payload) {
    qDebug() << "Protocol::parsePing: Received ping";
    return true;
}

bool Protocol::parsePong(const QByteArray& payload) {
    qDebug() << "Protocol::parsePong: Received pong";
    return true;
}

// === Message Building ===

QByteArray Protocol::buildLogon(const QString& username, const QString& wizardPassword) {
    QByteArray msg;
    
    // Header
    appendU32(msg, static_cast<uint32_t>(MessageType::LOGON));
    
    // Reserve space for length (will update later)
    int lengthPos = msg.size();
    appendU32(msg, 0);
    
    // RefNum (0 for now)
    appendU32(msg, 0);
    
    // Payload
    int payloadStart = msg.size();
    
    // Registration record fields (simplified for MVP)
    appendU32(msg, 0); // regCRC
    appendU32(msg, 0); // regCounter
    writePascalString(msg, username);
    writePascalString(msg, wizardPassword);
    appendU32(msg, 0); // ulUploadCaps
    appendU32(msg, 0); // ulDownloadCaps
    
    // Update length field
    uint32_t payloadLen = msg.size() - payloadStart;
    *reinterpret_cast<uint32_t*>(msg.data() + lengthPos) = payloadLen;
    
    qDebug() << "Protocol::buildLogon: Built logon message for" << username;
    return msg;
}

QByteArray Protocol::buildTalk(const QString& text) {
    QByteArray msg;
    
    appendU32(msg, static_cast<uint32_t>(MessageType::TALK));
    
    int lengthPos = msg.size();
    appendU32(msg, 0);
    appendU32(msg, 0); // refNum
    
    int payloadStart = msg.size();
    writePascalString(msg, text);
    
    uint32_t payloadLen = msg.size() - payloadStart;
    *reinterpret_cast<uint32_t*>(msg.data() + lengthPos) = payloadLen;
    
    qDebug() << "Protocol::buildTalk: Built talk message:" << text;
    return msg;
}

QByteArray Protocol::buildXTalk(const QString& text) {
    // For MVP, send as plain talk
    // Full implementation needs RC4 encryption
    QByteArray msg = buildTalk(text);
    
    // Change message type to XTALK
    *reinterpret_cast<uint32_t*>(msg.data()) = static_cast<uint32_t>(MessageType::XTALK);
    
    qDebug() << "Protocol::buildXTalk: Built xtalk message (MVP: unencrypted)";
    return msg;
}

QByteArray Protocol::buildRoomGoto(int16_t roomId) {
    QByteArray msg;
    
    appendU32(msg, static_cast<uint32_t>(MessageType::ROOMGOTO));
    
    int lengthPos = msg.size();
    appendU32(msg, 0);
    appendU32(msg, 0); // refNum
    
    int payloadStart = msg.size();
    appendI16(msg, roomId);
    
    uint32_t payloadLen = msg.size() - payloadStart;
    *reinterpret_cast<uint32_t*>(msg.data() + lengthPos) = payloadLen;
    
    qDebug() << "Protocol::buildRoomGoto: Built room goto message for room" << roomId;
    return msg;
}

QByteArray Protocol::buildListRooms() {
    QByteArray msg;
    
    appendU32(msg, static_cast<uint32_t>(MessageType::LISTOFALLROOMS));
    appendU32(msg, 0); // length (no payload)
    appendU32(msg, 0); // refNum
    
    qDebug() << "Protocol::buildListRooms: Built list rooms message";
    return msg;
}

QByteArray Protocol::buildPing() {
    QByteArray msg;
    
    appendU32(msg, static_cast<uint32_t>(MessageType::PING));
    appendU32(msg, 0); // length (no payload)
    appendU32(msg, 0); // refNum
    
    qDebug() << "Protocol::buildPing: Built ping message";
    return msg;
}

QByteArray Protocol::buildPong() {
    QByteArray msg;
    
    appendU32(msg, static_cast<uint32_t>(MessageType::PONG));
    appendU32(msg, 0); // length (no payload)
    appendU32(msg, 0); // refNum
    
    qDebug() << "Protocol::buildPong: Built pong message";
    return msg;
}

QByteArray Protocol::buildLogoff() {
    QByteArray msg;
    
    appendU32(msg, static_cast<uint32_t>(MessageType::LOGOFF));
    appendU32(msg, 0); // length (no payload)
    appendU32(msg, 0); // refNum
    
    qDebug() << "Protocol::buildLogoff: Built logoff message";
    return msg;
}

QByteArray Protocol::buildUserMove(const Point& pos) {
    QByteArray msg;
    
    appendU32(msg, static_cast<uint32_t>(MessageType::USERMOVE));
    
    int lengthPos = msg.size();
    appendU32(msg, 0);
    appendU32(msg, 0); // refNum
    
    int payloadStart = msg.size();
    appendI16(msg, pos.h);
    appendI16(msg, pos.v);
    
    uint32_t payloadLen = msg.size() - payloadStart;
    *reinterpret_cast<uint32_t*>(msg.data() + lengthPos) = payloadLen;
    
    qDebug() << "Protocol::buildUserMove: Moving to (" << pos.h << "," << pos.v << ")";
    return msg;
}

QByteArray Protocol::buildUserName(const QString& name) {
    QByteArray msg;
    
    appendU32(msg, static_cast<uint32_t>(MessageType::USERNAME));
    
    int lengthPos = msg.size();
    appendU32(msg, 0);
    appendU32(msg, 0); // refNum
    
    int payloadStart = msg.size();
    writePascalString(msg, name);
    
    uint32_t payloadLen = msg.size() - payloadStart;
    *reinterpret_cast<uint32_t*>(msg.data() + lengthPos) = payloadLen;
    
    qDebug() << "Protocol::buildUserName: Changing name to" << name;
    return msg;
}

QByteArray Protocol::buildUserColor(int16_t color) {
    QByteArray msg;
    
    appendU32(msg, static_cast<uint32_t>(MessageType::USERCOLOR));
    
    int lengthPos = msg.size();
    appendU32(msg, 0);
    appendU32(msg, 0); // refNum
    
    int payloadStart = msg.size();
    appendI16(msg, color);
    
    uint32_t payloadLen = msg.size() - payloadStart;
    *reinterpret_cast<uint32_t*>(msg.data() + lengthPos) = payloadLen;
    
    qDebug() << "Protocol::buildUserColor: Changing color to" << color;
    return msg;
}

QByteArray Protocol::buildUserFace(int16_t face) {
    QByteArray msg;
    
    appendU32(msg, static_cast<uint32_t>(MessageType::USERFACE));
    
    int lengthPos = msg.size();
    appendU32(msg, 0);
    appendU32(msg, 0); // refNum
    
    int payloadStart = msg.size();
    appendI16(msg, face);
    
    uint32_t payloadLen = msg.size() - payloadStart;
    *reinterpret_cast<uint32_t*>(msg.data() + lengthPos) = payloadLen;
    
    qDebug() << "Protocol::buildUserFace: Changing face to" << face;
    return msg;
}

QByteArray Protocol::buildUserProp(const QList<PropSpec>& props) {
    QByteArray msg;
    
    appendU32(msg, static_cast<uint32_t>(MessageType::USERPROP));
    
    int lengthPos = msg.size();
    appendU32(msg, 0);
    appendU32(msg, 0); // refNum
    
    int payloadStart = msg.size();
    appendI16(msg, static_cast<int16_t>(props.size()));
    
    // Write each prop (AssetSpec = 8 bytes: id + crc)
    for (const PropSpec& prop : props) {
        appendU32(msg, prop.spec.id);
        appendU32(msg, prop.spec.crc);
    }
    
    uint32_t payloadLen = msg.size() - payloadStart;
    *reinterpret_cast<uint32_t*>(msg.data() + lengthPos) = payloadLen;
    
    qDebug() << "Protocol::buildUserProp: Setting" << props.size() << "props";
    return msg;
}

QByteArray Protocol::buildWhisper(uint32_t targetUserId, const QString& text) {
    QByteArray msg;
    
    appendU32(msg, static_cast<uint32_t>(MessageType::WHISPER));
    
    int lengthPos = msg.size();
    appendU32(msg, 0);
    appendU32(msg, 0); // refNum
    
    int payloadStart = msg.size();
    appendU32(msg, targetUserId);
    writePascalString(msg, text);
    
    uint32_t payloadLen = msg.size() - payloadStart;
    *reinterpret_cast<uint32_t*>(msg.data() + lengthPos) = payloadLen;
    
    qDebug() << "Protocol::buildWhisper: Whispering to user" << targetUserId << ":" << text;
    return msg;
}

QByteArray Protocol::buildGlobalMsg(const QString& text) {
    QByteArray msg;
    
    appendU32(msg, static_cast<uint32_t>(MessageType::GMSG));
    
    int lengthPos = msg.size();
    appendU32(msg, 0);
    appendU32(msg, 0); // refNum
    
    int payloadStart = msg.size();
    writePascalString(msg, text);
    
    uint32_t payloadLen = msg.size() - payloadStart;
    *reinterpret_cast<uint32_t*>(msg.data() + lengthPos) = payloadLen;
    
    qDebug() << "Protocol::buildGlobalMsg: Global message:" << text;
    return msg;
}

QByteArray Protocol::buildSpotState(uint16_t spotId, int16_t state) {
    QByteArray msg;
    
    appendU32(msg, static_cast<uint32_t>(MessageType::SPOTSTATE));
    
    int lengthPos = msg.size();
    appendU32(msg, 0);
    appendU32(msg, 0); // refNum
    
    int payloadStart = msg.size();
    appendU16(msg, spotId);
    appendI16(msg, state);
    
    uint32_t payloadLen = msg.size() - payloadStart;
    *reinterpret_cast<uint32_t*>(msg.data() + lengthPos) = payloadLen;
    
    qDebug() << "Protocol::buildSpotState: Setting hotspot" << spotId << "to state" << state;
    return msg;
}

QByteArray Protocol::buildDoorLock(uint16_t spotId) {
    QByteArray msg;
    
    appendU32(msg, static_cast<uint32_t>(MessageType::DOORLOCK));
    
    int lengthPos = msg.size();
    appendU32(msg, 0);
    appendU32(msg, 0); // refNum
    
    int payloadStart = msg.size();
    appendU16(msg, spotId);
    
    uint32_t payloadLen = msg.size() - payloadStart;
    *reinterpret_cast<uint32_t*>(msg.data() + lengthPos) = payloadLen;
    
    qDebug() << "Protocol::buildDoorLock: Locking door" << spotId;
    return msg;
}

QByteArray Protocol::buildDoorUnlock(uint16_t spotId) {
    QByteArray msg;
    
    appendU32(msg, static_cast<uint32_t>(MessageType::DOORUNLOCK));
    
    int lengthPos = msg.size();
    appendU32(msg, 0);
    appendU32(msg, 0); // refNum
    
    int payloadStart = msg.size();
    appendU16(msg, spotId);
    
    uint32_t payloadLen = msg.size() - payloadStart;
    *reinterpret_cast<uint32_t*>(msg.data() + lengthPos) = payloadLen;
    
    qDebug() << "Protocol::buildDoorUnlock: Unlocking door" << spotId;
    return msg;
}

} // namespace Network
} // namespace Palace
