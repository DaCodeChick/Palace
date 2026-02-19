import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: root
    
    property var session: null
    
    // Main layout
    RowLayout {
        anchors.fill: parent
        spacing: 0
        
        // Left sidebar - User list
        Rectangle {
            Layout.preferredWidth: 200
            Layout.fillHeight: true
            color: "#2a2a2a"
            
            ColumnLayout {
                anchors.fill: parent
                spacing: 0
                
                // Header
                Rectangle {
                    Layout.fillWidth: true
                    height: 40
                    color: "#3a3a3a"
                    
                    Label {
                        anchors.centerIn: parent
                        text: "Users in Room"
                        font.pixelSize: 13
                        font.bold: true
                        color: "#ffffff"
                    }
                }
                
                // User list
                UserList {
                    id: userList
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    session: root.session
                }
            }
        }
        
        // Center - Room canvas
        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: "#1a1a1a"
            
            ColumnLayout {
                anchors.fill: parent
                spacing: 0
                
                // Room display area
                Rectangle {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    Layout.minimumHeight: 400
                    color: "#ffffff"
                    border.color: "#4a4a4a"
                    border.width: 1
                    
                    // Room canvas (will be replaced with PalaceCanvas)
                    Item {
                        id: roomCanvas
                        anchors.fill: parent
                        anchors.margins: 1
                        
                        // Placeholder for MVP - shows room name
                        Rectangle {
                            anchors.centerIn: parent
                            width: 300
                            height: 200
                            color: "#f0f0f0"
                            radius: 8
                            border.color: "#cccccc"
                            border.width: 2
                            
                            ColumnLayout {
                                anchors.centerIn: parent
                                spacing: 20
                                
                                Label {
                                    text: session && session.loggedIn ? 
                                        session.currentRoomName :
                                        "Not Connected"
                                    font.pixelSize: 24
                                    font.bold: true
                                    Layout.alignment: Qt.AlignHCenter
                                    color: "#333333"
                                }
                                
                                Label {
                                    text: session && session.loggedIn ?
                                        "Room ID: " + session.currentRoomId :
                                        ""
                                    font.pixelSize: 14
                                    Layout.alignment: Qt.AlignHCenter
                                    color: "#666666"
                                }
                                
                                Label {
                                    text: "(Room rendering coming soon)"
                                    font.pixelSize: 11
                                    font.italic: true
                                    Layout.alignment: Qt.AlignHCenter
                                    color: "#999999"
                                }
                            }
                        }
                        
                        // User avatars would be rendered here
                        // TODO: Replace with PalaceCanvas for prop rendering
                    }
                }
                
                // Chat panel at bottom
                ChatPanel {
                    id: chatPanel
                    Layout.fillWidth: true
                    Layout.preferredHeight: 200
                    Layout.minimumHeight: 150
                    session: root.session
                }
            }
        }
    }
    
    // Handle session events
    Connections {
        target: session
        
        function onCurrentRoomChanged() {
            // Room changed - canvas will update automatically via binding
            console.log("Room changed to:", session.currentRoomName)
        }
        
        function onUsersUpdated(users) {
            console.log("Users updated, count:", users.length)
        }
    }
}
