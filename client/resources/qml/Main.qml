import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
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
            loginDialog.open()
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
                        onClicked: loginDialog.open()
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
                onTriggered: loginDialog.open()
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
                        roomListDialog.open()
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
                onTriggered: aboutDialog.open()
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
    Dialog {
        id: roomListDialog
        title: "Room List"
        width: 400
        height: 500
        modal: true
        
        contentItem: RoomList {
            session: root.session
            
            onRoomSelected: function(roomId) {
                if (session) {
                    session.goToRoom(roomId)
                }
                roomListDialog.close()
            }
        }
    }
    
    // About Dialog
    Dialog {
        id: aboutDialog
        title: "About Palace Client"
        width: 400
        height: 300
        modal: true
        standardButtons: Dialog.Ok
        
        contentItem: Item {
            ColumnLayout {
                anchors.centerIn: parent
                spacing: 20
                
                Label {
                    text: "Palace Client"
                    font.pixelSize: 24
                    font.bold: true
                    Layout.alignment: Qt.AlignHCenter
                }
                
                Label {
                    text: "Version 0.1.0"
                    font.pixelSize: 14
                    Layout.alignment: Qt.AlignHCenter
                    color: "#888888"
                }
                
                Label {
                    text: "A modern implementation of The Palace\nvisual chat system"
                    font.pixelSize: 12
                    Layout.alignment: Qt.AlignHCenter
                    horizontalAlignment: Text.AlignHCenter
                }
                
                Label {
                    text: "Built with Qt 6.10 and C++23"
                    font.pixelSize: 10
                    Layout.alignment: Qt.AlignHCenter
                    color: "#888888"
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
