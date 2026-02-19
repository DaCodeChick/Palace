#include "Connection.h"
#include <QDebug>

namespace Palace {
namespace Network {

Connection::Connection(QObject* parent)
    : QObject(parent)
    , m_socket(new QTcpSocket(this))
    , m_host("localhost")
    , m_port(9998)
    , m_connected(false)
{
    // Connect socket signals to our slots
    connect(m_socket, &QTcpSocket::connected, this, &Connection::onConnected);
    connect(m_socket, &QTcpSocket::disconnected, this, &Connection::onDisconnected);
    connect(m_socket, &QTcpSocket::readyRead, this, &Connection::onReadyRead);
    connect(m_socket, &QTcpSocket::errorOccurred, this, &Connection::onErrorOccurred);
}

Connection::~Connection()
{
    if (m_connected) {
        m_socket->disconnectFromHost();
    }
}

QString Connection::host() const
{
    return m_host;
}

void Connection::setHost(const QString& host)
{
    if (m_host != host) {
        m_host = host;
        emit hostChanged();
    }
}

quint16 Connection::port() const
{
    return m_port;
}

void Connection::setPort(quint16 port)
{
    if (m_port != port) {
        m_port = port;
        emit portChanged();
    }
}

bool Connection::isConnected() const
{
    return m_connected;
}

QString Connection::errorString() const
{
    return m_errorString;
}

void Connection::connectToServer()
{
    if (m_connected) {
        qWarning() << "Connection::connectToServer: Already connected";
        return;
    }

    qDebug() << "Connection::connectToServer: Connecting to" << m_host << ":" << m_port;
    m_socket->connectToHost(m_host, m_port);
}

void Connection::disconnectFromServer()
{
    if (!m_connected) {
        qWarning() << "Connection::disconnectFromServer: Not connected";
        return;
    }

    qDebug() << "Connection::disconnectFromServer: Disconnecting";
    m_socket->disconnectFromHost();
}

void Connection::sendData(const QByteArray& data)
{
    if (!m_connected) {
        qWarning() << "Connection::sendData: Not connected";
        return;
    }

    qint64 bytesWritten = m_socket->write(data);
    if (bytesWritten == -1) {
        qWarning() << "Connection::sendData: Failed to write data:" << m_socket->errorString();
        return;
    }

    qDebug() << "Connection::sendData: Sent" << bytesWritten << "bytes";
}

void Connection::onConnected()
{
    qDebug() << "Connection::onConnected: Successfully connected to server";
    m_connected = true;
    m_errorString.clear();
    
    emit connectedChanged();
    emit errorStringChanged();
    emit connected();
}

void Connection::onDisconnected()
{
    qDebug() << "Connection::onDisconnected: Disconnected from server";
    m_connected = false;
    
    emit connectedChanged();
    emit disconnected();
}

void Connection::onReadyRead()
{
    QByteArray data = m_socket->readAll();
    qDebug() << "Connection::onReadyRead: Received" << data.size() << "bytes";
    
    emit dataReceived(data);
}

void Connection::onErrorOccurred(QAbstractSocket::SocketError socketError)
{
    Q_UNUSED(socketError);
    
    m_errorString = m_socket->errorString();
    qWarning() << "Connection::onErrorOccurred:" << m_errorString;
    
    emit errorStringChanged();
    emit errorOccurred(m_errorString);
}

} // namespace Network
} // namespace Palace
