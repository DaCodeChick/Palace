import QtQuick
import QtQuick.Controls

ListView {
    id: root
    clip: true
    
    property var session: null
    
    model: ListModel {
        id: userModel
    }
    
    delegate: ItemDelegate {
        width: root.width
        height: 40
        
        contentItem: Row {
            spacing: 10
            
            // Avatar placeholder
            Rectangle {
                width: 30
                height: 30
                radius: 15
                color: "#4caf50"
                anchors.verticalCenter: parent.verticalCenter
                
                Label {
                    anchors.centerIn: parent
                    text: model.name ? model.name[0].toUpperCase() : "?"
                    color: "#ffffff"
                    font.bold: true
                }
            }
            
            // Username
            Label {
                text: model.name || "Unknown"
                anchors.verticalCenter: parent.verticalCenter
                color: "#ffffff"
                font.pixelSize: 12
            }
        }
        
        background: Rectangle {
            color: parent.hovered ? "#3a3a3a" : "transparent"
        }
    }
    
    // Handle session user list updates
    Connections {
        target: session
        
        function onUsersUpdated(users) {
            userModel.clear()
            for (var i = 0; i < users.length; i++) {
                userModel.append({
                    userId: users[i].userId,
                    name: users[i].name,
                    roomId: users[i].roomId
                })
            }
        }
    }
    
    // Empty state
    Rectangle {
        anchors.centerIn: parent
        width: parent.width - 20
        height: 80
        visible: userModel.count === 0
        color: "transparent"
        
        Label {
            anchors.centerIn: parent
            text: "No users in room"
            color: "#888888"
            font.pixelSize: 11
            font.italic: true
        }
    }
}
