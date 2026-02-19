import QtQuick
import QtQuick.Controls

ListView {
    id: root
    clip: true
    
    property var session: null
    signal roomSelected(int roomId)
    
    model: ListModel {
        id: roomModel
    }
    
    delegate: ItemDelegate {
        width: root.width
        height: 50
        
        contentItem: Row {
            spacing: 15
            
            // Room icon
            Rectangle {
                width: 35
                height: 35
                radius: 4
                color: "#4a4a4a"
                anchors.verticalCenter: parent.verticalCenter
                
                Label {
                    anchors.centerIn: parent
                    text: "🏠"
                    font.pixelSize: 18
                }
            }
            
            // Room info
            Column {
                anchors.verticalCenter: parent.verticalCenter
                spacing: 3
                
                Label {
                    text: model.name || "Unknown Room"
                    color: "#ffffff"
                    font.pixelSize: 13
                    font.bold: true
                }
                
                Label {
                    text: model.userCount + " user" + (model.userCount !== 1 ? "s" : "")
                    color: "#888888"
                    font.pixelSize: 10
                }
            }
        }
        
        background: Rectangle {
            color: parent.hovered ? "#3a3a3a" : "transparent"
        }
        
        onClicked: {
            root.roomSelected(model.roomId)
        }
    }
    
    // Handle session room list updates
    Connections {
        target: session
        
        function onRoomListReceived(rooms) {
            roomModel.clear()
            for (var i = 0; i < rooms.length; i++) {
                roomModel.append({
                    roomId: rooms[i].roomId,
                    name: rooms[i].name,
                    userCount: rooms[i].userCount
                })
            }
        }
    }
    
    // Empty state
    Rectangle {
        anchors.centerIn: parent
        width: parent.width - 20
        height: 100
        visible: roomModel.count === 0
        color: "transparent"
        
        Column {
            anchors.centerIn: parent
            spacing: 10
            
            Label {
                anchors.horizontalCenter: parent.horizontalCenter
                text: "No rooms available"
                color: "#888888"
                font.pixelSize: 12
                font.italic: true
            }
            
            Button {
                anchors.horizontalCenter: parent.horizontalCenter
                text: "Refresh"
                onClicked: {
                    if (session) {
                        session.requestRoomList()
                    }
                }
            }
        }
    }
}
