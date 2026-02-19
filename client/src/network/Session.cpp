#include "Session.h"
#include <QDebug>

namespace Palace {
namespace Network {

Session::Session(QObject* parent)
    : QObject(parent)
    , m_connection(new Connection(this))
    , m_protocol()
    , m_connected(false)
    , m_loggedIn(false)
    , m_userId(0)
    , m_currentRoomId(-1)
{
    // Connect to connection signals
    connect(m_connection, &Connection::connected, this, &Session::onConnected);
    connect(m_connection, &Connection::disconnected, this, &Session::onDisconnected);
    connect(m_connection, &Connection::dataReceived, this, &Session::onDataReceived);
    connect(m_connection, &Connection::errorOccurred, this, &Session::onConnectionError);
}

Session::~Session()
{
    if (m_connected) {
        disconnectFromServer();
    }
}

void Session::setUsername(const QString& username)
{
    if (m_username != username) {
        m_username = username;
        emit usernameChanged();
    }
}

void Session::connectToServer(const QString& host, quint16 port)
{
    qDebug() << "Session::connectToServer:" << host << ":" << port;
    m_connection->setHost(host);
    m_connection->setPort(port);
    m_connection->connectToServer();
}

void Session::disconnectFromServer()
{
    qDebug() << "Session::disconnectFromServer";
    m_connection->disconnectFromServer();
}

void Session::login(const QString& username, const QString& wizardPassword)
{
    if (!m_connected) {
        qWarning() << "Session::login: Not connected";
        return;
    }
    
    qDebug() << "Session::login: Logging in as" << username;
    setUsername(username);
    
    QByteArray logonMsg = Protocol::buildLogon(username, wizardPassword);
    m_connection->sendData(logonMsg);
}

void Session::sendChat(const QString& text)
{
    if (!m_loggedIn) {
        qWarning() << "Session::sendChat: Not logged in";
        return;
    }
    
    qDebug() << "Session::sendChat:" << text;
    QByteArray talkMsg = Protocol::buildTalk(text);
    m_connection->sendData(talkMsg);
}

void Session::sendEncryptedChat(const QString& text)
{
    if (!m_loggedIn) {
        qWarning() << "Session::sendEncryptedChat: Not logged in";
        return;
    }
    
    qDebug() << "Session::sendEncryptedChat:" << text;
    QByteArray xtalkMsg = Protocol::buildXTalk(text);
    m_connection->sendData(xtalkMsg);
}

void Session::goToRoom(int16_t roomId)
{
    if (!m_loggedIn) {
        qWarning() << "Session::goToRoom: Not logged in";
        return;
    }
    
    qDebug() << "Session::goToRoom:" << roomId;
    QByteArray gotoMsg = Protocol::buildRoomGoto(roomId);
    m_connection->sendData(gotoMsg);
}

void Session::requestRoomList()
{
    if (!m_loggedIn) {
        qWarning() << "Session::requestRoomList: Not logged in";
        return;
    }
    
    qDebug() << "Session::requestRoomList";
    QByteArray listMsg = Protocol::buildListRooms();
    m_connection->sendData(listMsg);
}

// === Connection Event Handlers ===

void Session::onConnected()
{
    qDebug() << "Session::onConnected";
    m_connected = true;
    m_receiveBuffer.clear();
    emit connectedChanged();
}

void Session::onDisconnected()
{
    qDebug() << "Session::onDisconnected";
    m_connected = false;
    m_loggedIn = false;
    m_userId = 0;
    m_currentRoomId = -1;
    m_currentRoomName.clear();
    m_currentUsers.clear();
    m_roomList.clear();
    m_receiveBuffer.clear();
    
    emit connectedChanged();
    emit loggedInChanged();
    emit currentRoomChanged();
}

void Session::onDataReceived(const QByteArray& data)
{
    qDebug() << "Session::onDataReceived:" << data.size() << "bytes";
    
    // Append to receive buffer
    m_receiveBuffer.append(data);
    
    // Extract and handle complete messages
    QByteArray message;
    while (extractMessage(message)) {
        handleMessage(message);
    }
}

void Session::onConnectionError(const QString& error)
{
    qWarning() << "Session::onConnectionError:" << error;
    emit connectionError(error);
}

// === Message Extraction ===

bool Session::extractMessage(QByteArray& message)
{
    // Need at least 12 bytes for header
    if (m_receiveBuffer.size() < 12) {
        return false;
    }
    
    // Parse header to get message length
    ProtocolHeader header;
    if (!Protocol::parseHeader(m_receiveBuffer, header)) {
        qWarning() << "Session::extractMessage: Failed to parse header";
        return false;
    }
    
    // Check if we have the complete message
    uint32_t totalSize = 12 + header.length;
    if (m_receiveBuffer.size() < static_cast<int>(totalSize)) {
        qDebug() << "Session::extractMessage: Incomplete message, need" << totalSize 
                 << "have" << m_receiveBuffer.size();
        return false;
    }
    
    // Extract the complete message
    message = m_receiveBuffer.left(totalSize);
    m_receiveBuffer.remove(0, totalSize);
    
    qDebug() << "Session::extractMessage: Extracted message of size" << totalSize;
    return true;
}

// === Message Handlers ===

void Session::handleMessage(const QByteArray& message)
{
    MessageType msgType = Protocol::identifyMessage(message);
    QByteArray payload = message.mid(12); // Skip 12-byte header
    
    qDebug() << "Session::handleMessage: Type" << Qt::hex << static_cast<uint32_t>(msgType);
    
    switch (msgType) {
        case MessageType::TIYID:
            handleTiyid(payload);
            break;
        case MessageType::SERVERINFO:
            handleServerInfo(payload);
            break;
        case MessageType::VERSION:
            handleVersion(payload);
            break;
        case MessageType::SERVERDOWN:
            handleServerDown(payload);
            break;
        case MessageType::USERNEW:
            handleUserNew(payload);
            break;
        case MessageType::USEREXIT:
            handleUserLeft(payload);
            break;
        case MessageType::USERLIST:
            handleUserList(payload);
            break;
        case MessageType::USERMOVE:
            handleUserMove(payload);
            break;
        case MessageType::USERNAME:
            handleUserName(payload);
            break;
        case MessageType::USERCOLOR:
            handleUserColor(payload);
            break;
        case MessageType::USERFACE:
            handleUserFace(payload);
            break;
        case MessageType::USERPROP:
            handleUserProp(payload);
            break;
        case MessageType::USERSTATUS:
            handleUserStatus(payload);
            break;
        case MessageType::ROOMDESC:
            handleRoomDesc(payload);
            break;
        case MessageType::ROOMDESCEND:
            handleRoomDescEnd(payload);
            break;
        case MessageType::LISTOFALLROOMS:
            handleRoomList(payload);
            break;
        case MessageType::NAVERROR:
            handleNavError(payload);
            break;
        case MessageType::TALK:
            handleTalk(payload);
            break;
        case MessageType::XTALK:
            handleXTalk(payload);
            break;
        case MessageType::WHISPER:
            handleWhisper(payload);
            break;
        case MessageType::XWHISPER:
            handleWhisper(payload); // Same handler as WHISPER
            break;
        case MessageType::GMSG:
            handleGlobalMsg(payload);
            break;
        case MessageType::RMSG:
            handleRoomMsg(payload);
            break;
        case MessageType::SPOTSTATE:
            handleSpotState(payload);
            break;
        case MessageType::DISPLAYURL:
            handleDisplayUrl(payload);
            break;
        case MessageType::PING:
            handlePing(payload);
            break;
        case MessageType::PONG:
            handlePong(payload);
            break;
        default:
            qWarning() << "Session::handleMessage: Unknown message type" 
                      << Qt::hex << static_cast<uint32_t>(msgType);
            break;
    }
}

void Session::handleTiyid(const QByteArray& payload)
{
    qDebug() << "Session::handleTiyid";
    Protocol::parseTiyid(payload);
    emit serverHandshakeReceived();
}

void Session::handleServerInfo(const QByteArray& payload)
{
    qDebug() << "Session::handleServerInfo";
    Protocol::parseServerInfo(payload);
    
    // After receiving server info, we're effectively logged in
    m_loggedIn = true;
    emit loggedInChanged();
}

void Session::handleUserNew(const QByteArray& payload)
{
    qDebug() << "Session::handleUserNew";
    UserInfo user = Protocol::parseUserNew(payload);
    
    // Add to current user list if in same room
    if (user.roomId == m_currentRoomId) {
        m_currentUsers.append(user);
        emit userJoined(user);
        emit usersUpdated(m_currentUsers);
    }
}

void Session::handleUserLeft(const QByteArray& payload)
{
    qDebug() << "Session::handleUserLeft";
    uint32_t userId = Protocol::parseUserExit(payload);
    if (userId != 0) {
        // Remove from current user list
        for (int i = 0; i < m_currentUsers.size(); ++i) {
            if (m_currentUsers[i].userId == userId) {
                m_currentUsers.removeAt(i);
                break;
            }
        }
        
        emit userLeft(userId);
        emit usersUpdated(m_currentUsers);
    }
}

void Session::handleUserList(const QByteArray& payload)
{
    qDebug() << "Session::handleUserList";
    m_currentUsers = Protocol::parseUserList(payload);
    emit usersUpdated(m_currentUsers);
}

void Session::handleRoomDesc(const QByteArray& payload)
{
    qDebug() << "Session::handleRoomDesc";
    RoomInfo room = Protocol::parseRoomDesc(payload);
    
    m_currentRoomId = room.roomId;
    m_currentRoomName = room.name;
    m_currentUsers.clear(); // Will be populated by subsequent USERLIST message
    
    emit currentRoomChanged();
}

void Session::handleRoomList(const QByteArray& payload)
{
    qDebug() << "Session::handleRoomList";
    m_roomList = Protocol::parseRoomList(payload);
    emit roomListReceived(m_roomList);
}

void Session::handleTalk(const QByteArray& payload)
{
    qDebug() << "Session::handleTalk";
    ChatMessage msg = Protocol::parseTalk(payload);
    emit chatReceived(msg.username, msg.text, msg.isWhisper);
}

void Session::handleXTalk(const QByteArray& payload)
{
    qDebug() << "Session::handleXTalk";
    ChatMessage msg = Protocol::parseXTalk(payload);
    emit chatReceived(msg.username, msg.text, msg.isWhisper);
}

void Session::handlePong(const QByteArray& payload)
{
    Q_UNUSED(payload);
    qDebug() << "Session::handlePong: Received keepalive response";
}

void Session::handleVersion(const QByteArray& payload)
{
    uint32_t version = Protocol::parseVersion(payload);
    qDebug() << "Session::handleVersion: Server version" << QString::number(version, 16);
}

void Session::handleServerDown(const QByteArray& payload)
{
    QString reason = Protocol::parseServerDown(payload);
    qWarning() << "Session::handleServerDown:" << reason;
    
    // Emit connection error to notify UI
    emit connectionError("Server is shutting down: " + reason);
    
    // Disconnect gracefully
    disconnectFromServer();
}

void Session::handleUserMove(const QByteArray& payload)
{
    uint32_t userId;
    Point pos;
    
    if (Protocol::parseUserMove(payload, userId, pos)) {
        // Update user position in current user list
        for (UserInfo& user : m_currentUsers) {
            if (user.userId == userId) {
                user.roomPos = pos;
                emit usersUpdated(m_currentUsers);
                break;
            }
        }
    }
}

void Session::handleUserName(const QByteArray& payload)
{
    uint32_t userId;
    QString name;
    
    if (Protocol::parseUserName(payload, userId, name)) {
        // Update user name in current user list
        for (UserInfo& user : m_currentUsers) {
            if (user.userId == userId) {
                user.name = name;
                emit usersUpdated(m_currentUsers);
                break;
            }
        }
    }
}

void Session::handleUserColor(const QByteArray& payload)
{
    uint32_t userId;
    int16_t color;
    
    if (Protocol::parseUserColor(payload, userId, color)) {
        // Update user color in current user list
        for (UserInfo& user : m_currentUsers) {
            if (user.userId == userId) {
                user.colorNbr = color;
                emit usersUpdated(m_currentUsers);
                break;
            }
        }
    }
}

void Session::handleUserFace(const QByteArray& payload)
{
    uint32_t userId;
    int16_t face;
    
    if (Protocol::parseUserFace(payload, userId, face)) {
        // Update user face in current user list
        for (UserInfo& user : m_currentUsers) {
            if (user.userId == userId) {
                user.faceNbr = face;
                emit usersUpdated(m_currentUsers);
                break;
            }
        }
    }
}

void Session::handleUserProp(const QByteArray& payload)
{
    uint32_t userId;
    QList<PropSpec> props;
    
    if (Protocol::parseUserProp(payload, userId, props)) {
        // Update user props in current user list
        for (UserInfo& user : m_currentUsers) {
            if (user.userId == userId) {
                // Copy props to user's propSpec array (max 9 props)
                int count = qMin(props.size(), 9);
                for (int i = 0; i < count; ++i) {
                    user.propSpec[i] = props[i];
                }
                user.nbrProps = count;
                emit usersUpdated(m_currentUsers);
                break;
            }
        }
    }
}

void Session::handleUserStatus(const QByteArray& payload)
{
    uint32_t userId;
    uint16_t flags;
    
    if (Protocol::parseUserStatus(payload, userId, flags)) {
        // Update user flags in current user list
        for (UserInfo& user : m_currentUsers) {
            if (user.userId == userId) {
                user.flags = flags;
                emit usersUpdated(m_currentUsers);
                break;
            }
        }
    }
}

void Session::handleRoomDescEnd(const QByteArray& payload)
{
    Q_UNUSED(payload);
    qDebug() << "Session::handleRoomDescEnd: Room description complete";
    // Room transmission sequence is complete
    // All room data, hotspots, pictures, etc. have been received
}

void Session::handleNavError(const QByteArray& payload)
{
    QString errorMsg = Protocol::parseNavError(payload);
    qWarning() << "Session::handleNavError:" << errorMsg;
    emit connectionError("Navigation error: " + errorMsg);
}

void Session::handleWhisper(const QByteArray& payload)
{
    ChatMessage msg = Protocol::parseWhisper(payload);
    emit chatReceived(msg.username, msg.text, true); // isWhisper = true
}

void Session::handleGlobalMsg(const QByteArray& payload)
{
    QString msg = Protocol::parseGlobalMsg(payload);
    qDebug() << "Session::handleGlobalMsg:" << msg;
    // Emit as system message (username = empty)
    emit chatReceived("", msg, false);
}

void Session::handleRoomMsg(const QByteArray& payload)
{
    QString msg = Protocol::parseRoomMsg(payload);
    qDebug() << "Session::handleRoomMsg:" << msg;
    // Emit as system message (username = empty)
    emit chatReceived("", msg, false);
}

void Session::handleSpotState(const QByteArray& payload)
{
    uint16_t spotId;
    int16_t state;
    
    if (Protocol::parseSpotState(payload, spotId, state)) {
        qDebug() << "Session::handleSpotState: Hotspot" << spotId << "changed to state" << state;
        // Future: Update room state with new hotspot state
    }
}

void Session::handleDisplayUrl(const QByteArray& payload)
{
    QString url = Protocol::parseDisplayUrl(payload);
    qDebug() << "Session::handleDisplayUrl:" << url;
    // Future: Emit signal to open URL in browser
    // For now, just log it
}

void Session::handlePing(const QByteArray& payload)
{
    Q_UNUSED(payload);
    qDebug() << "Session::handlePing: Received ping, sending pong";
    
    // Respond with pong
    QByteArray pongMsg = Protocol::buildPong();
    m_connection->sendData(pongMsg);
}

} // namespace Network
} // namespace Palace
