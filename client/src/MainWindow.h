#ifndef MAINWINDOW_H
#define MAINWINDOW_H

#include <QObject>

class MainWindow : public QObject
{
    Q_OBJECT
    
public:
    explicit MainWindow(QObject *parent = nullptr);
    ~MainWindow();

signals:
    void statusChanged(const QString &status);

private slots:
    void onConnected();
    void onDisconnected();
};

#endif // MAINWINDOW_H
