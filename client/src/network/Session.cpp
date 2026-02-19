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
        case MessageType::RPRS:
            handleServerInfo(payload);
            break;
        case MessageType::USERNEW:
            handleUserNew(payload);
            break;
        case MessageType::USERLEFT:
            handleUserLeft(payload);
            break;
        case MessageType::USERLIST:
            handleUserList(payload);
            break;
        case MessageType::ROOMDESC:
            handleRoomDesc(payload);
            break;
        case MessageType::RMLIST:
            handleRoomList(payload);
            break;
        case MessageType::TALK:
            handleTalk(payload);
            break;
        case MessageType::XTALK:
            handleXTalk(payload);
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
    uint32_t userId;
    if (Protocol::parseUserLeft(payload, userId)) {
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

} // namespace Network
} // namespace Palace
