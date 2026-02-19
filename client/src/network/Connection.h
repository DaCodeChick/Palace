#ifndef PALACE_CONNECTION_H
#define PALACE_CONNECTION_H

#include <QObject>
#include <QTcpSocket>
#include <QByteArray>
#include <QString>

namespace Palace {

/**
 * @brief TCP connection to Palace server
 * 
 * Handles low-level socket communication with the Palace server.
 * Emits signals for connection state changes and raw data reception.
 */
class Connection : public QObject
{
    Q_OBJECT
    Q_PROPERTY(QString host READ host WRITE setHost NOTIFY hostChanged)
    Q_PROPERTY(quint16 port READ port WRITE setPort NOTIFY portChanged)
    Q_PROPERTY(bool connected READ isConnected NOTIFY connectedChanged)
    Q_PROPERTY(QString errorString READ errorString NOTIFY errorOccurred)

public:
    explicit Connection(QObject *parent = nullptr);
    ~Connection() override;

    // Properties
    QString host() const { return m_host; }
    void setHost(const QString &host);
    
    quint16 port() const { return m_port; }
    void setPort(quint16 port);
    
    bool isConnected() const;
    QString errorString() const;

public slots:
    /// Connect to the server
    void connectToServer();
    
    /// Disconnect from the server
    void disconnectFromServer();
    
    /// Send raw data to the server
    void sendData(const QByteArray &data);

signals:
    void hostChanged();
    void portChanged();
    void connectedChanged();
    void errorOccurred(const QString &error);
    
    /// Emitted when successfully connected
    void connected();
    
    /// Emitted when disconnected
    void disconnected();
    
    /// Emitted when raw data is received
    void dataReceived(const QByteArray &data);

private slots:
    void onSocketConnected();
    void onSocketDisconnected();
    void onSocketError(QAbstractSocket::SocketError error);
    void onSocketReadyRead();

private:
    QTcpSocket *m_socket;
    QString m_host;
    quint16 m_port;
};

} // namespace Palace

#endif // PALACE_CONNECTION_H
