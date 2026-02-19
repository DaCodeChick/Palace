import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window
import Palace.Network 1.0

ApplicationWindow {
    id: root
    visible: true
    width: 1024
    height: 768
    minimumWidth: 800
    minimumHeight: 600
    title: "Palace Client"
    
    // Session object (exposed from C++)
    property var session: null
    
    // Current view state
    property bool isLoggedIn: session ? session.loggedIn : false
    
    color: "#2c2c2c"
    
    // Show login dialog at startup
    Component.onCompleted: {
        if (!isLoggedIn) {
            loginDialog.show()
        }
    }
    
    // Main content area
    StackLayout {
        id: mainStack
        anchors.fill: parent
        currentIndex: isLoggedIn ? 1 : 0
        
        // Index 0: Connection/loading screen
        Item {
            Rectangle {
                anchors.centerIn: parent
                width: 300
                height: 200
                color: "#3c3c3c"
                radius: 8
                
                ColumnLayout {
                    anchors.centerIn: parent
                    spacing: 20
                    
                    Label {
                        text: "Not Connected"
                        font.pixelSize: 20
                        Layout.alignment: Qt.AlignHCenter
                        color: "#ffffff"
                    }
                    
                    Button {
                        text: "Connect to Server"
                        Layout.alignment: Qt.AlignHCenter
                        onClicked: loginDialog.show()
                    }
                }
            }
        }
        
        // Index 1: Main room view
        RoomView {
            id: roomView
            session: root.session
        }
    }
    
    // Login Dialog
    LoginDialog {
        id: loginDialog
        session: root.session
        
        onAccepted: {
            if (session) {
                session.connectToServer(serverHost, serverPort)
                session.login(username, wizardPassword)
            }
        }
    }
    
    // Menu Bar
    menuBar: MenuBar {
        Menu {
            title: "&File"
            
            MenuItem {
                text: "Connect..."
                onTriggered: loginDialog.show()
            }
            
            MenuItem {
                text: "Disconnect"
                enabled: isLoggedIn
                onTriggered: {
                    if (session) {
                        session.disconnectFromServer()
                    }
                }
            }
            
            MenuSeparator {}
            
            MenuItem {
                text: "Exit"
                onTriggered: Qt.quit()
            }
        }
        
        Menu {
            title: "&Rooms"
            enabled: isLoggedIn
            
            MenuItem {
                text: "Room List..."
                onTriggered: {
                    if (session) {
                        session.requestRoomList()
                        roomListDialog.show()
                    }
                }
            }
            
            MenuItem {
                text: "Go to Main Hall"
                onTriggered: {
                    if (session) {
                        session.goToRoom(1) // Room ID 1 = Main Hall
                    }
                }
            }
        }
        
        Menu {
            title: "&Help"
            
            MenuItem {
                text: "About Palace"
                onTriggered: aboutDialog.show()
            }
        }
    }
    
    // Status Bar
    footer: ToolBar {
        height: 30
        
        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 10
            anchors.rightMargin: 10
            spacing: 10
            
            Label {
                text: isLoggedIn ? 
                    (session ? "Connected: " + session.username : "Connected") : 
                    "Not Connected"
                font.pixelSize: 12
                color: isLoggedIn ? "#4caf50" : "#f44336"
            }
            
            Item { Layout.fillWidth: true }
            
            Label {
                text: session && isLoggedIn ? 
                    "Room: " + session.currentRoomName : 
                    ""
                font.pixelSize: 12
                color: "#ffffff"
            }
        }
    }
    
    // Room List Dialog
    Window {
        id: roomListDialog
        title: "Room List"
        width: 450
        height: 550
        minimumWidth: 400
        minimumHeight: 500
        modality: Qt.ApplicationModal
        flags: Qt.Dialog
        color: "#1e1e1e"
        
        Rectangle {
            anchors.fill: parent
            gradient: Gradient {
                GradientStop { position: 0.0; color: "#2d2d30" }
                GradientStop { position: 1.0; color: "#1e1e1e" }
            }
        }
        
        ColumnLayout {
            anchors.fill: parent
            spacing: 0
            
            // Header
            Rectangle {
                Layout.fillWidth: true
                height: 60
                gradient: Gradient {
                    GradientStop { position: 0.0; color: "#3e3e42" }
                    GradientStop { position: 1.0; color: "#2d2d30" }
                }
                
                RowLayout {
                    anchors.fill: parent
                    anchors.margins: 15
                    spacing: 12
                    
                    Rectangle {
                        width: 32
                        height: 32
                        radius: 16
                        color: "#007acc"
                        border.color: "#4fc3f7"
                        border.width: 2
                        
                        Label {
                            anchors.centerIn: parent
                            text: "🚪"
                            font.pixelSize: 18
                        }
                    }
                    
                    Label {
                        text: "Available Rooms"
                        font.pixelSize: 18
                        font.bold: true
                        color: "#e0e0e0"
                        Layout.fillWidth: true
                    }
                }
            }
            
            // Room list content
            Item {
                Layout.fillWidth: true
                Layout.fillHeight: true
                
                Rectangle {
                    anchors.fill: parent
                    anchors.margins: 16
                    color: "#252526"
                    border.color: "#3e3e42"
                    radius: 6
                    
                    RoomList {
                        anchors.fill: parent
                        anchors.margins: 1
                        session: root.session
                        
                        onRoomSelected: function(roomId) {
                            if (session) {
                                session.goToRoom(roomId)
                            }
                            roomListDialog.close()
                        }
                    }
                }
            }
            
            // Button bar
            Rectangle {
                Layout.fillWidth: true
                height: 60
                color: "#2d2d30"
                border.color: "#3e3e42"
                border.width: 1
                
                Button {
                    anchors.centerIn: parent
                    text: "Close"
                    width: 100
                    font.pixelSize: 12
                    
                    background: Rectangle {
                        color: parent.pressed ? "#3e3e42" : parent.hovered ? "#2d2d30" : "#252526"
                        border.color: "#3e3e42"
                        border.width: 1
                        radius: 4
                    }
                    
                    contentItem: Label {
                        text: parent.text
                        font: parent.font
                        color: "#c0c0c0"
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                    
                    onClicked: roomListDialog.close()
                }
            }
        }
    }
    
    // About Dialog
    Window {
        id: aboutDialog
        title: "About Palace Client"
        width: 450
        height: 380
        minimumWidth: 400
        minimumHeight: 350
        modality: Qt.ApplicationModal
        flags: Qt.Dialog
        color: "#1e1e1e"
        
        Rectangle {
            anchors.fill: parent
            gradient: Gradient {
                GradientStop { position: 0.0; color: "#2d2d30" }
                GradientStop { position: 1.0; color: "#1e1e1e" }
            }
        }
        
        ColumnLayout {
            anchors.fill: parent
            spacing: 0
            
            // Header with icon
            Rectangle {
                Layout.fillWidth: true
                height: 80
                gradient: Gradient {
                    GradientStop { position: 0.0; color: "#3e3e42" }
                    GradientStop { position: 1.0; color: "#2d2d30" }
                }
                
                ColumnLayout {
                    anchors.centerIn: parent
                    spacing: 8
                    
                    Label {
                        text: "🏰"
                        font.pixelSize: 48
                        Layout.alignment: Qt.AlignHCenter
                    }
                }
            }
            
            // Content area
            Item {
                Layout.fillWidth: true
                Layout.fillHeight: true
                
                ColumnLayout {
                    anchors.centerIn: parent
                    spacing: 20
                    width: parent.width - 40
                    
                    Label {
                        text: "Palace Client"
                        font.pixelSize: 28
                        font.bold: true
                        Layout.alignment: Qt.AlignHCenter
                        color: "#e0e0e0"
                    }
                    
                    Label {
                        text: "Version 0.1.0"
                        font.pixelSize: 16
                        Layout.alignment: Qt.AlignHCenter
                        color: "#4fc3f7"
                    }
                    
                    Rectangle {
                        Layout.fillWidth: true
                        height: 1
                        color: "#3e3e42"
                        Layout.topMargin: 10
                        Layout.bottomMargin: 10
                    }
                    
                    Label {
                        text: "A modern implementation of\nThe Palace visual chat system"
                        font.pixelSize: 13
                        Layout.alignment: Qt.AlignHCenter
                        horizontalAlignment: Text.AlignHCenter
                        color: "#c0c0c0"
                        lineHeight: 1.4
                    }
                    
                    Label {
                        text: "Built with Qt 6.10 and C++23"
                        font.pixelSize: 11
                        Layout.alignment: Qt.AlignHCenter
                        color: "#909090"
                        Layout.topMargin: 10
                    }
                    
                    Label {
                        text: "© 2026 Palace Project"
                        font.pixelSize: 10
                        Layout.alignment: Qt.AlignHCenter
                        color: "#6a6a6a"
                    }
                }
            }
            
            // Button bar
            Rectangle {
                Layout.fillWidth: true
                height: 60
                color: "#2d2d30"
                border.color: "#3e3e42"
                border.width: 1
                
                Button {
                    anchors.centerIn: parent
                    text: "Close"
                    width: 100
                    font.pixelSize: 12
                    
                    background: Rectangle {
                        color: parent.pressed ? "#005a9e" : parent.hovered ? "#1f7bb8" : "#007acc"
                        border.color: "#4fc3f7"
                        border.width: 1
                        radius: 4
                    }
                    
                    contentItem: Label {
                        text: parent.text
                        font: parent.font
                        color: "#ffffff"
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                    
                    onClicked: aboutDialog.close()
                }
            }
        }
    }
    
    // Connection error handler
    Connections {
        target: session
        
        function onConnectionError(error) {
            errorDialog.text = error
            errorDialog.open()
        }
        
        function onConnectedChanged() {
            if (!session.connected && isLoggedIn) {
                // Connection lost
                statusMessage.show("Connection lost", 5000)
            }
        }
    }
    
    // Error Dialog
    Dialog {
        id: errorDialog
        title: "Connection Error"
        width: 350
        height: 150
        modal: true
        standardButtons: Dialog.Ok
        
        property alias text: errorText.text
        
        contentItem: Item {
            Label {
                id: errorText
                anchors.centerIn: parent
                width: parent.width - 40
                wrapMode: Text.Wrap
                horizontalAlignment: Text.AlignHCenter
            }
        }
    }
    
    // Status Message (toast-style notification)
    Rectangle {
        id: statusMessage
        anchors.bottom: parent.bottom
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottomMargin: 60
        width: messageText.width + 40
        height: 40
        color: "#3c3c3c"
        radius: 8
        opacity: 0
        visible: opacity > 0
        
        property alias text: messageText.text
        
        Label {
            id: messageText
            anchors.centerIn: parent
            color: "#ffffff"
        }
        
        function show(message, duration) {
            text = message
            showAnimation.start()
            hideTimer.interval = duration || 3000
            hideTimer.start()
        }
        
        PropertyAnimation {
            id: showAnimation
            target: statusMessage
            property: "opacity"
            to: 0.9
            duration: 200
        }
        
        PropertyAnimation {
            id: hideAnimation
            target: statusMessage
            property: "opacity"
            to: 0
            duration: 200
        }
        
        Timer {
            id: hideTimer
            onTriggered: hideAnimation.start()
        }
    }
}
