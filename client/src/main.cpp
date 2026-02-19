#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QDebug>
#include "network/Session.h"

int main(int argc, char *argv[])
{
    qDebug() << "Palace Client starting...";
    QGuiApplication app(argc, argv);
    qDebug() << "QGuiApplication created";
    
    app.setOrganizationName("Palace");
    app.setOrganizationDomain("palace.chat");
    app.setApplicationName("Palace Client");
    qDebug() << "App metadata set";
    
    // Register custom types for QML
    qmlRegisterType<Palace::Network::Session>("Palace.Network", 1, 0, "Session");
    qDebug() << "QML types registered";
    
    QQmlApplicationEngine engine;
    qDebug() << "QML engine created";
    
    // Create session object
    Palace::Network::Session session;
    qDebug() << "Session object created";
    
    // Expose session to QML
    engine.rootContext()->setContextProperty("session", &session);
    qDebug() << "Session exposed to QML context";
    
    // Load main QML
    const QUrl url(QStringLiteral("qrc:/qml/Main.qml"));
    qDebug() << "Loading QML from:" << url;
    QObject::connect(&engine, &QQmlApplicationEngine::objectCreated,
                     &app, [url](QObject *obj, const QUrl &objUrl) {
        if (!obj && url == objUrl)
            QCoreApplication::exit(-1);
    }, Qt::QueuedConnection);
    
    engine.load(url);
    
    return app.exec();
}
