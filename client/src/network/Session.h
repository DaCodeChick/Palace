#pragma once

#include "Connection.h"
#include "Protocol.h"
#include <QObject>
#include <QString>
#include <QList>
#include <QByteArray>

namespace Palace {
namespace Network {

/**
 * @brief High-level Palace session management
 * 
 * Manages connection state, protocol handling, and coordinates
 * between network layer and UI layer.
 */
class Session : public QObject {
    Q_OBJECT
    Q_PROPERTY(bool connected READ isConnected NOTIFY connectedChanged)
    Q_PROPERTY(bool loggedIn READ isLoggedIn NOTIFY loggedInChanged)
    Q_PROPERTY(QString username READ username WRITE setUsername NOTIFY usernameChanged)
    Q_PROPERTY(QString currentRoomName READ currentRoomName NOTIFY currentRoomChanged)
    Q_PROPERTY(int currentRoomId READ currentRoomId NOTIFY currentRoomChanged)

public:
    explicit Session(QObject* parent = nullptr);
    ~Session() override;
    
    // Connection state
    bool isConnected() const { return m_connected; }
    bool isLoggedIn() const { return m_loggedIn; }
    
    // User properties
    QString username() const { return m_username; }
    void setUsername(const QString& username);
    
    uint32_t userId() const { return m_userId; }
    
    // Room state
    QString currentRoomName() const { return m_currentRoomName; }
    int currentRoomId() const { return m_currentRoomId; }
    
    QList<UserInfo> currentUsers() const { return m_currentUsers; }
    QList<RoomInfo> roomList() const { return m_roomList; }

public slots:
    // Connection management
    void connectToServer(const QString& host, quint16 port);
    void disconnectFromServer();
    
    // Authentication
    void login(const QString& username, const QString& wizardPassword = QString());
    
    // Chat
    void sendChat(const QString& text);
    void sendEncryptedChat(const QString& text);
    
    // Room navigation
    void goToRoom(int16_t roomId);
    void requestRoomList();

signals:
    // Connection signals
    void connectedChanged();
    void loggedInChanged();
    void connectionError(const QString& error);
    
    // User signals
    void usernameChanged();
    
    // Room signals
    void currentRoomChanged();
    void userJoined(const UserInfo& user);
    void userLeft(uint32_t userId);
    void usersUpdated(const QList<UserInfo>& users);
    void roomListReceived(const QList<RoomInfo>& rooms);
    
    // Chat signals
    void chatReceived(const QString& username, const QString& text, bool isWhisper);
    
    // Protocol signals
    void serverHandshakeReceived();

private slots:
    // Connection event handlers
    void onConnected();
    void onDisconnected();
    void onDataReceived(const QByteArray& data);
    void onConnectionError(const QString& error);
    
    // Protocol message handlers
    void handleMessage(const QByteArray& message);
    void handleTiyid(const QByteArray& payload);
    void handleServerInfo(const QByteArray& payload);
    void handleUserNew(const QByteArray& payload);
    void handleUserLeft(const QByteArray& payload);
    void handleUserList(const QByteArray& payload);
    void handleRoomDesc(const QByteArray& payload);
    void handleRoomList(const QByteArray& payload);
    void handleTalk(const QByteArray& payload);
    void handleXTalk(const QByteArray& payload);
    void handlePong(const QByteArray& payload);

private:
    Connection* m_connection;
    Protocol m_protocol;
    
    // Connection state
    bool m_connected;
    bool m_loggedIn;
    
    // User state
    QString m_username;
    uint32_t m_userId;
    
    // Room state
    QString m_currentRoomName;
    int16_t m_currentRoomId;
    QList<UserInfo> m_currentUsers;
    QList<RoomInfo> m_roomList;
    
    // Message buffer for incomplete messages
    QByteArray m_receiveBuffer;
    
    // Helper to extract complete messages from buffer
    bool extractMessage(QByteArray& message);
};

} // namespace Network
} // namespace Palace
