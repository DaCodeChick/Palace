#include "Protocol.h"
#include <QDebug>
#include <QtEndian>

namespace Palace {
namespace Network {

Protocol::Protocol() = default;
Protocol::~Protocol() = default;

// === Byte Order Helpers ===

uint32_t Protocol::readBigEndianU32(const QByteArray& data, int offset) {
    if (offset + 4 > data.size()) return 0;
    return qFromBigEndian<uint32_t>(reinterpret_cast<const uchar*>(data.constData() + offset));
}

uint16_t Protocol::readBigEndianU16(const QByteArray& data, int offset) {
    if (offset + 2 > data.size()) return 0;
    return qFromBigEndian<uint16_t>(reinterpret_cast<const uchar*>(data.constData() + offset));
}

int32_t Protocol::readBigEndianI32(const QByteArray& data, int offset) {
    return static_cast<int32_t>(readBigEndianU32(data, offset));
}

int16_t Protocol::readBigEndianI16(const QByteArray& data, int offset) {
    return static_cast<int16_t>(readBigEndianU16(data, offset));
}

void Protocol::writeBigEndianU32(QByteArray& data, uint32_t value) {
    uchar buf[4];
    qToBigEndian(value, buf);
    data.append(reinterpret_cast<char*>(buf), 4);
}

void Protocol::writeBigEndianU16(QByteArray& data, uint16_t value) {
    uchar buf[2];
    qToBigEndian(value, buf);
    data.append(reinterpret_cast<char*>(buf), 2);
}

void Protocol::writeBigEndianI32(QByteArray& data, int32_t value) {
    writeBigEndianU32(data, static_cast<uint32_t>(value));
}

void Protocol::writeBigEndianI16(QByteArray& data, int16_t value) {
    writeBigEndianU16(data, static_cast<uint16_t>(value));
}

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
    
    header.eventType = readBigEndianU32(data, 0);
    header.length = readBigEndianU32(data, 4);
    header.refNum = readBigEndianU32(data, 8);
    
    qDebug() << "Protocol::parseHeader: type=" << Qt::hex << header.eventType
             << "len=" << Qt::dec << header.length 
             << "ref=" << header.refNum;
    
    return true;
}

MessageType Protocol::identifyMessage(const QByteArray& data) {
    if (data.size() < 4) return static_cast<MessageType>(0);
    uint32_t type = readBigEndianU32(data, 0);
    return static_cast<MessageType>(type);
}

uint32_t Protocol::parseTiyid(const QByteArray& payload) {
    // TIYID message contains UserID assigned by server
    if (payload.size() < 4) {
        qWarning() << "Protocol::parseTiyid: Payload too short";
        return 0;
    }
    
    uint32_t userId = readBigEndianU32(payload, 0);
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
    user.userId = readBigEndianU32(payload, offset); offset += 4;
    user.roomPos.h = readBigEndianI16(payload, offset); offset += 2;
    user.roomPos.v = readBigEndianI16(payload, offset); offset += 2;
    user.roomId = readBigEndianI16(payload, offset); offset += 2;
    user.faceNbr = readBigEndianI16(payload, offset); offset += 2;
    user.colorNbr = readBigEndianI16(payload, offset); offset += 2;
    
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
    
    uint32_t userId = readBigEndianU32(payload, 0);
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
    uint32_t userCount = readBigEndianU32(payload, offset); offset += 4;
    
    qDebug() << "Protocol::parseUserList: Parsing" << userCount << "users";
    
    for (uint32_t i = 0; i < userCount && offset < payload.size(); ++i) {
        UserInfo user = {};
        
        if (offset + 12 > payload.size()) break;
        
        user.userId = readBigEndianU32(payload, offset); offset += 4;
        user.roomPos.h = readBigEndianI16(payload, offset); offset += 2;
        user.roomPos.v = readBigEndianI16(payload, offset); offset += 2;
        user.roomId = readBigEndianI16(payload, offset); offset += 2;
        user.faceNbr = readBigEndianI16(payload, offset); offset += 2;
        user.colorNbr = readBigEndianI16(payload, offset); offset += 2;
        
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
    room.roomFlags = readBigEndianU32(payload, offset); offset += 4;
    room.facesID = readBigEndianU32(payload, offset); offset += 4;
    room.roomId = readBigEndianI16(payload, offset); offset += 2;
    
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
    uint32_t roomCount = readBigEndianU32(payload, offset); offset += 4;
    
    qDebug() << "Protocol::parseRoomList: Parsing" << roomCount << "rooms";
    
    for (uint32_t i = 0; i < roomCount && offset < payload.size(); ++i) {
        RoomInfo room = {};
        
        if (offset + 2 > payload.size()) break;
        
        room.roomId = readBigEndianI16(payload, offset); offset += 2;
        room.name = readPascalString(payload, offset);
        
        // User count follows name
        if (offset + 2 <= payload.size()) {
            room.nbrPeople = readBigEndianI16(payload, offset); offset += 2;
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

// === Message Building ===

QByteArray Protocol::buildLogon(const QString& username, const QString& wizardPassword) {
    QByteArray msg;
    
    // Header
    writeBigEndianU32(msg, static_cast<uint32_t>(MessageType::LOGON));
    
    // Reserve space for length (will update later)
    int lengthPos = msg.size();
    writeBigEndianU32(msg, 0);
    
    // RefNum (0 for now)
    writeBigEndianU32(msg, 0);
    
    // Payload
    int payloadStart = msg.size();
    
    // Registration record fields (simplified for MVP)
    writeBigEndianU32(msg, 0); // regCRC
    writeBigEndianU32(msg, 0); // regCounter
    writePascalString(msg, username);
    writePascalString(msg, wizardPassword);
    writeBigEndianU32(msg, 0); // ulUploadCaps
    writeBigEndianU32(msg, 0); // ulDownloadCaps
    
    // Update length field
    uint32_t payloadLen = msg.size() - payloadStart;
    qToBigEndian(payloadLen, reinterpret_cast<uchar*>(msg.data() + lengthPos));
    
    qDebug() << "Protocol::buildLogon: Built logon message for" << username;
    return msg;
}

QByteArray Protocol::buildTalk(const QString& text) {
    QByteArray msg;
    
    writeBigEndianU32(msg, static_cast<uint32_t>(MessageType::TALK));
    
    int lengthPos = msg.size();
    writeBigEndianU32(msg, 0);
    writeBigEndianU32(msg, 0); // refNum
    
    int payloadStart = msg.size();
    writePascalString(msg, text);
    
    uint32_t payloadLen = msg.size() - payloadStart;
    qToBigEndian(payloadLen, reinterpret_cast<uchar*>(msg.data() + lengthPos));
    
    qDebug() << "Protocol::buildTalk: Built talk message:" << text;
    return msg;
}

QByteArray Protocol::buildXTalk(const QString& text) {
    // For MVP, send as plain talk
    // Full implementation needs RC4 encryption
    QByteArray msg = buildTalk(text);
    
    // Change message type to XTALK
    qToBigEndian(static_cast<uint32_t>(MessageType::XTALK), 
                 reinterpret_cast<uchar*>(msg.data()));
    
    qDebug() << "Protocol::buildXTalk: Built xtalk message (MVP: unencrypted)";
    return msg;
}

QByteArray Protocol::buildRoomGoto(int16_t roomId) {
    QByteArray msg;
    
    writeBigEndianU32(msg, static_cast<uint32_t>(MessageType::ROOMGOTO));
    
    int lengthPos = msg.size();
    writeBigEndianU32(msg, 0);
    writeBigEndianU32(msg, 0); // refNum
    
    int payloadStart = msg.size();
    writeBigEndianI16(msg, roomId);
    
    uint32_t payloadLen = msg.size() - payloadStart;
    qToBigEndian(payloadLen, reinterpret_cast<uchar*>(msg.data() + lengthPos));
    
    qDebug() << "Protocol::buildRoomGoto: Built room goto message for room" << roomId;
    return msg;
}

QByteArray Protocol::buildListRooms() {
    QByteArray msg;
    
    writeBigEndianU32(msg, static_cast<uint32_t>(MessageType::LISTOFALLROOMS));
    writeBigEndianU32(msg, 0); // length (no payload)
    writeBigEndianU32(msg, 0); // refNum
    
    qDebug() << "Protocol::buildListRooms: Built list rooms message";
    return msg;
}

QByteArray Protocol::buildPing() {
    QByteArray msg;
    
    writeBigEndianU32(msg, static_cast<uint32_t>(MessageType::PING));
    writeBigEndianU32(msg, 0); // length (no payload)
    writeBigEndianU32(msg, 0); // refNum
    
    qDebug() << "Protocol::buildPing: Built ping message";
    return msg;
}

} // namespace Network
} // namespace Palace
