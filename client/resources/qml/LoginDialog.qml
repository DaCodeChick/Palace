import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Dialog {
    id: root
    title: "Connect to Palace Server"
    width: 450
    height: 350
    modal: true
    standardButtons: Dialog.Ok | Dialog.Cancel
    
    property var session: null
    
    // Expose connection parameters
    property string serverHost: serverHostField.text
    property int serverPort: parseInt(serverPortField.text)
    property string username: usernameField.text
    property string wizardPassword: wizardPasswordField.text
    
    // Persist settings
    Settings {
        id: settings
        property alias lastHost: serverHostField.text
        property alias lastPort: serverPortField.text
        property alias lastUsername: usernameField.text
    }
    
    contentItem: Item {
        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 20
            spacing: 15
            
            Label {
                text: "Server Connection"
                font.pixelSize: 16
                font.bold: true
            }
            
            // Server Host
            RowLayout {
                spacing: 10
                Layout.fillWidth: true
                
                Label {
                    text: "Host:"
                    Layout.preferredWidth: 100
                }
                
                TextField {
                    id: serverHostField
                    text: "localhost"
                    placeholderText: "server.palace.com"
                    Layout.fillWidth: true
                    selectByMouse: true
                    
                    Keys.onReturnPressed: {
                        if (root.standardButton(Dialog.Ok).enabled) {
                            root.accept()
                        }
                    }
                }
            }
            
            // Server Port
            RowLayout {
                spacing: 10
                Layout.fillWidth: true
                
                Label {
                    text: "Port:"
                    Layout.preferredWidth: 100
                }
                
                TextField {
                    id: serverPortField
                    text: "9998"
                    placeholderText: "9998"
                    Layout.preferredWidth: 100
                    selectByMouse: true
                    validator: IntValidator { bottom: 1; top: 65535 }
                    
                    Keys.onReturnPressed: {
                        if (root.standardButton(Dialog.Ok).enabled) {
                            root.accept()
                        }
                    }
                }
                
                Item { Layout.fillWidth: true }
            }
            
            Rectangle {
                Layout.fillWidth: true
                height: 1
                color: "#cccccc"
                Layout.topMargin: 5
                Layout.bottomMargin: 5
            }
            
            Label {
                text: "User Information"
                font.pixelSize: 16
                font.bold: true
            }
            
            // Username
            RowLayout {
                spacing: 10
                Layout.fillWidth: true
                
                Label {
                    text: "Username:"
                    Layout.preferredWidth: 100
                }
                
                TextField {
                    id: usernameField
                    text: "Guest"
                    placeholderText: "Enter your name"
                    Layout.fillWidth: true
                    selectByMouse: true
                    maximumLength: 31
                    
                    Keys.onReturnPressed: {
                        if (root.standardButton(Dialog.Ok).enabled) {
                            root.accept()
                        }
                    }
                }
            }
            
            // Wizard Password (optional)
            RowLayout {
                spacing: 10
                Layout.fillWidth: true
                
                Label {
                    text: "Wizard Pass:"
                    Layout.preferredWidth: 100
                    color: "#888888"
                }
                
                TextField {
                    id: wizardPasswordField
                    placeholderText: "Optional - for operators"
                    Layout.fillWidth: true
                    selectByMouse: true
                    echoMode: TextInput.Password
                    maximumLength: 31
                    
                    Keys.onReturnPressed: {
                        if (root.standardButton(Dialog.Ok).enabled) {
                            root.accept()
                        }
                    }
                }
            }
            
            Item { Layout.fillHeight: true }
            
            // Connection status/tips
            Rectangle {
                Layout.fillWidth: true
                height: 60
                color: "#f5f5f5"
                radius: 4
                
                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 10
                    spacing: 5
                    
                    Label {
                        text: "💡 Tips:"
                        font.pixelSize: 11
                        font.bold: true
                    }
                    
                    Label {
                        text: "• Default port is 9998\n• Wizard password is optional (for server operators only)"
                        font.pixelSize: 10
                        color: "#666666"
                        wrapMode: Text.Wrap
                        Layout.fillWidth: true
                    }
                }
            }
        }
    }
    
    // Validate fields
    function validate() {
        return serverHostField.text.length > 0 && 
               serverPortField.text.length > 0 &&
               usernameField.text.length > 0
    }
    
    // Enable/disable OK button based on validation
    Component.onCompleted: {
        standardButton(Dialog.Ok).enabled = Qt.binding(validate)
    }
    
    // Focus username field when opened
    onOpened: {
        usernameField.forceActiveFocus()
        usernameField.selectAll()
    }
}
