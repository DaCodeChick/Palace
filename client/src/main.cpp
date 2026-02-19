#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include "network/Session.h"

int main(int argc, char *argv[])
{
    QGuiApplication app(argc, argv);
    
    app.setOrganizationName("Palace");
    app.setOrganizationDomain("palace.chat");
    app.setApplicationName("Palace Client");
    
    // Register custom types for QML
    qmlRegisterType<Palace::Network::Session>("Palace.Network", 1, 0, "Session");
    
    QQmlApplicationEngine engine;
    
    // Create session object
    Palace::Network::Session session;
    
    // Expose session to QML
    engine.rootContext()->setContextProperty("session", &session);
    
    // Load main QML
    const QUrl url(QStringLiteral("qrc:/qml/Main.qml"));
    QObject::connect(&engine, &QQmlApplicationEngine::objectCreated,
                     &app, [url](QObject *obj, const QUrl &objUrl) {
        if (!obj && url == objUrl)
            QCoreApplication::exit(-1);
    }, Qt::QueuedConnection);
    
    engine.load(url);
    
    return app.exec();
}
