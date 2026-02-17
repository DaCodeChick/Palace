#include "MainWindow.h"

MainWindow::MainWindow(QObject *parent)
    : QObject(parent)
{
    // TODO: Initialize window
}

MainWindow::~MainWindow()
{
    // Cleanup
}

void MainWindow::onConnected()
{
    emit statusChanged("Connected");
}

void MainWindow::onDisconnected()
{
    emit statusChanged("Disconnected");
}
