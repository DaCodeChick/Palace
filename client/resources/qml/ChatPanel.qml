import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: root
    color: "#2c2c2c"
    
    property var session: null
    
    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        
        // Header with resize handle
        Rectangle {
            Layout.fillWidth: true
            height: 30
            color: "#3c3c3c"
            
            Label {
                anchors.left: parent.left
                anchors.leftMargin: 10
                anchors.verticalCenter: parent.verticalCenter
                text: "Chat"
                font.pixelSize: 12
                font.bold: true
                color: "#ffffff"
            }
            
            // Resize handle visual indicator
            Rectangle {
                anchors.horizontalCenter: parent.horizontalCenter
                anchors.top: parent.top
                width: 40
                height: 4
                radius: 2
                color: "#666666"
                
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.SizeVerCursor
                    // TODO: Implement resize drag
                }
            }
        }
        
        // Chat messages area
        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: "#1a1a1a"
            border.color: "#4a4a4a"
            border.width: 1
            
            ScrollView {
                anchors.fill: parent
                anchors.margins: 5
                clip: true
                
                ListView {
                    id: chatListView
                    spacing: 5
                    
                    model: ListModel {
                        id: chatModel
                    }
                    
                    delegate: Item {
                        width: chatListView.width
                        height: messageText.height + 10
                        
                        Rectangle {
                            anchors.fill: parent
                            color: model.isWhisper ? "#2a2a4a" : "transparent"
                            radius: model.isWhisper ? 4 : 0
                        }
                        
                        Row {
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.verticalCenter: parent.verticalCenter
                            anchors.margins: 5
                            spacing: 8
                            
                            // Timestamp
                            Label {
                                text: model.timestamp
                                font.pixelSize: 10
                                color: "#888888"
                                width: 50
                            }
                            
                            // Username
                            Label {
                                text: model.username + ":"
                                font.pixelSize: 11
                                font.bold: true
                                color: model.isWhisper ? "#ffaa00" : "#4caf50"
                            }
                            
                            // Message text
                            Label {
                                id: messageText
                                text: model.message
                                font.pixelSize: 11
                                color: "#ffffff"
                                wrapMode: Text.Wrap
                                width: parent.width - 150
                            }
                        }
                    }
                    
                    // Auto-scroll to bottom on new message
                    onCountChanged: {
                        if (count > 0) {
                            positionViewAtEnd()
                        }
                    }
                }
            }
        }
        
        // Chat input area
        Rectangle {
            Layout.fillWidth: true
            height: 50
            color: "#2c2c2c"
            border.color: "#4a4a4a"
            border.width: 1
            
            RowLayout {
                anchors.fill: parent
                anchors.margins: 5
                spacing: 5
                
                TextField {
                    id: chatInput
                    Layout.fillWidth: true
                    placeholderText: session && session.loggedIn ? 
                        "Type your message..." : 
                        "Not connected"
                    enabled: session && session.loggedIn
                    selectByMouse: true
                    
                    background: Rectangle {
                        color: chatInput.enabled ? "#ffffff" : "#cccccc"
                        radius: 4
                    }
                    
                    Keys.onReturnPressed: {
                        sendMessage()
                    }
                }
                
                Button {
                    text: "Send"
                    enabled: session && session.loggedIn && chatInput.text.length > 0
                    Layout.preferredWidth: 70
                    
                    onClicked: {
                        sendMessage()
                    }
                }
                
                Button {
                    text: "📢"
                    enabled: session && session.loggedIn
                    Layout.preferredWidth: 40
                    ToolTip.text: "Encrypted chat"
                    ToolTip.visible: hovered
                    
                    onClicked: {
                        sendEncryptedMessage()
                    }
                }
            }
        }
    }
    
    // Handle incoming chat messages
    Connections {
        target: session
        
        function onChatReceived(username, text, isWhisper) {
            var now = new Date()
            var timestamp = Qt.formatTime(now, "hh:mm")
            
            chatModel.append({
                timestamp: timestamp,
                username: username,
                message: text,
                isWhisper: isWhisper
            })
            
            // Limit chat history to 200 messages
            if (chatModel.count > 200) {
                chatModel.remove(0)
            }
        }
    }
    
    // Functions
    function sendMessage() {
        if (!session || !session.loggedIn || chatInput.text.length === 0) {
            return
        }
        
        var message = chatInput.text.trim()
        if (message.length === 0) {
            return
        }
        
        session.sendChat(message)
        
        // Add to local chat (echo)
        var now = new Date()
        var timestamp = Qt.formatTime(now, "hh:mm")
        chatModel.append({
            timestamp: timestamp,
            username: session.username,
            message: message,
            isWhisper: false
        })
        
        chatInput.clear()
        chatInput.forceActiveFocus()
    }
    
    function sendEncryptedMessage() {
        if (!session || !session.loggedIn || chatInput.text.length === 0) {
            return
        }
        
        var message = chatInput.text.trim()
        if (message.length === 0) {
            return
        }
        
        session.sendEncryptedChat(message)
        
        // Add to local chat (echo) with whisper styling
        var now = new Date()
        var timestamp = Qt.formatTime(now, "hh:mm")
        chatModel.append({
            timestamp: timestamp,
            username: session.username,
            message: message + " 🔒",
            isWhisper: true
        })
        
        chatInput.clear()
        chatInput.forceActiveFocus()
    }
    
    // Add system message
    function addSystemMessage(message) {
        var now = new Date()
        var timestamp = Qt.formatTime(now, "hh:mm")
        
        chatModel.append({
            timestamp: timestamp,
            username: "System",
            message: message,
            isWhisper: false
        })
    }
    
    Component.onCompleted: {
        addSystemMessage("Welcome to Palace! Type a message to chat.")
    }
}
