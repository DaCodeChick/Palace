import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window
import QtCore

Window {
    id: root
    title: "Connect to Palace Server"
    width: 480
    height: 520
    minimumWidth: 400
    minimumHeight: 480
    maximumWidth: 600
    maximumHeight: 700
    modality: Qt.ApplicationModal
    flags: Qt.Dialog
    
    property var session: null
    
    // Expose connection parameters
    property string serverHost: serverHostField.text
    property int serverPort: parseInt(serverPortField.text)
    property string username: usernameField.text
    property string wizardPassword: wizardPasswordField.text
    
    signal accepted()
    signal rejected()
    
    // Persist settings
    Settings {
        id: settings
        property alias lastHost: serverHostField.text
        property alias lastPort: serverPortField.text
        property alias lastUsername: usernameField.text
    }
    
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
                    width: 36
                    height: 36
                    radius: 18
                    color: "#007acc"
                    border.color: "#4fc3f7"
                    border.width: 2
                    
                    Label {
                        anchors.centerIn: parent
                        text: "🏰"
                        font.pixelSize: 20
                    }
                }
                
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 2
                    
                    Label {
                        text: "Palace Chat"
                        font.pixelSize: 18
                        font.bold: true
                        color: "#e0e0e0"
                    }
                    
                    Label {
                        text: "Enter server connection details"
                        font.pixelSize: 11
                        color: "#a0a0a0"
                    }
                }
            }
        }
        
        // Main content with padding
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
            
            ScrollView {
                anchors.fill: parent
                anchors.margins: 24
                clip: true
                ScrollBar.vertical.policy: ScrollBar.AsNeeded
                ScrollBar.horizontal.policy: ScrollBar.AlwaysOff
                
                ColumnLayout {
                    width: parent.width
                    spacing: 20
                    
                    // Server Connection Section
                    GroupBox {
                        Layout.fillWidth: true
                        title: "Server Connection"
                        font.pixelSize: 13
                        font.bold: true
                        
                        label: Label {
                            text: parent.title
                            font: parent.font
                            color: "#4fc3f7"
                        }
                        
                        background: Rectangle {
                            y: parent.topPadding - parent.bottomPadding
                            width: parent.width
                            height: parent.height - parent.topPadding + parent.bottomPadding
                            color: "#252526"
                            border.color: "#3e3e42"
                            radius: 6
                        }
                        
                        ColumnLayout {
                            width: parent.width
                            spacing: 12
                            
                            // Server Host
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 6
                                
                                Label {
                                    text: "Server Address"
                                    font.pixelSize: 11
                                    font.bold: true
                                    color: "#c0c0c0"
                                }
                                
                                TextField {
                                    id: serverHostField
                                    text: "localhost"
                                    placeholderText: "server.palace.com"
                                    Layout.fillWidth: true
                                    selectByMouse: true
                                    font.pixelSize: 12
                                    
                                    background: Rectangle {
                                        color: serverHostField.activeFocus ? "#1e1e1e" : "#2d2d30"
                                        border.color: serverHostField.activeFocus ? "#007acc" : "#3e3e42"
                                        border.width: serverHostField.activeFocus ? 2 : 1
                                        radius: 4
                                    }
                                    
                                    color: "#e0e0e0"
                                    placeholderTextColor: "#6a6a6a"
                                    
                                    Keys.onReturnPressed: {
                                        if (validate()) {
                                            root.accepted()
                                            root.close()
                                        }
                                    }
                                }
                            }
                            
                            // Server Port
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 6
                                
                                Label {
                                    text: "Port Number"
                                    font.pixelSize: 11
                                    font.bold: true
                                    color: "#c0c0c0"
                                }
                                
                                TextField {
                                    id: serverPortField
                                    text: "9998"
                                    placeholderText: "9998"
                                    Layout.preferredWidth: 120
                                    selectByMouse: true
                                    font.pixelSize: 12
                                    validator: IntValidator { bottom: 1; top: 65535 }
                                    
                                    background: Rectangle {
                                        color: serverPortField.activeFocus ? "#1e1e1e" : "#2d2d30"
                                        border.color: serverPortField.activeFocus ? "#007acc" : "#3e3e42"
                                        border.width: serverPortField.activeFocus ? 2 : 1
                                        radius: 4
                                    }
                                    
                                    color: "#e0e0e0"
                                    placeholderTextColor: "#6a6a6a"
                                    
                                    Keys.onReturnPressed: {
                                        if (validate()) {
                                            root.accepted()
                                            root.close()
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // User Information Section
                    GroupBox {
                        Layout.fillWidth: true
                        title: "User Information"
                        font.pixelSize: 13
                        font.bold: true
                        
                        label: Label {
                            text: parent.title
                            font: parent.font
                            color: "#4fc3f7"
                        }
                        
                        background: Rectangle {
                            y: parent.topPadding - parent.bottomPadding
                            width: parent.width
                            height: parent.height - parent.topPadding + parent.bottomPadding
                            color: "#252526"
                            border.color: "#3e3e42"
                            radius: 6
                        }
                        
                        ColumnLayout {
                            width: parent.width
                            spacing: 12
                            
                            // Username
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 6
                                
                                Label {
                                    text: "Your Name"
                                    font.pixelSize: 11
                                    font.bold: true
                                    color: "#c0c0c0"
                                }
                                
                                TextField {
                                    id: usernameField
                                    text: "Guest"
                                    placeholderText: "Enter your name"
                                    Layout.fillWidth: true
                                    selectByMouse: true
                                    font.pixelSize: 12
                                    maximumLength: 31
                                    
                                    background: Rectangle {
                                        color: usernameField.activeFocus ? "#1e1e1e" : "#2d2d30"
                                        border.color: usernameField.activeFocus ? "#007acc" : "#3e3e42"
                                        border.width: usernameField.activeFocus ? 2 : 1
                                        radius: 4
                                    }
                                    
                                    color: "#e0e0e0"
                                    placeholderTextColor: "#6a6a6a"
                                    
                                    Keys.onReturnPressed: {
                                        if (validate()) {
                                            root.accepted()
                                            root.close()
                                        }
                                    }
                                }
                            }
                            
                            // Wizard Password (optional)
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 6
                                
                                Label {
                                    text: "Wizard Password (Optional)"
                                    font.pixelSize: 11
                                    font.bold: true
                                    color: "#909090"
                                }
                                
                                TextField {
                                    id: wizardPasswordField
                                    placeholderText: "For server operators only"
                                    Layout.fillWidth: true
                                    selectByMouse: true
                                    font.pixelSize: 12
                                    echoMode: TextInput.Password
                                    maximumLength: 31
                                    
                                    background: Rectangle {
                                        color: wizardPasswordField.activeFocus ? "#1e1e1e" : "#2d2d30"
                                        border.color: wizardPasswordField.activeFocus ? "#007acc" : "#3e3e42"
                                        border.width: wizardPasswordField.activeFocus ? 2 : 1
                                        radius: 4
                                    }
                                    
                                    color: "#e0e0e0"
                                    placeholderTextColor: "#6a6a6a"
                                    
                                    Keys.onReturnPressed: {
                                        if (validate()) {
                                            root.accepted()
                                            root.close()
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Info/Tips Section
                    Rectangle {
                        Layout.fillWidth: true
                        height: 70
                        color: "#1a3a52"
                        border.color: "#2d5f7f"
                        border.width: 1
                        radius: 6
                        
                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: 12
                            spacing: 12
                            
                            Label {
                                text: "💡"
                                font.pixelSize: 24
                                Layout.alignment: Qt.AlignTop
                            }
                            
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 4
                                
                                Label {
                                    text: "Quick Tips"
                                    font.pixelSize: 11
                                    font.bold: true
                                    color: "#4fc3f7"
                                }
                                
                                Label {
                                    text: "• Default Palace port is 9998\n• Wizard password is only needed for server operators"
                                    font.pixelSize: 10
                                    color: "#a0c4d4"
                                    wrapMode: Text.WordWrap
                                    Layout.fillWidth: true
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Button Bar
        Rectangle {
            Layout.fillWidth: true
            height: 64
            color: "#2d2d30"
            border.color: "#3e3e42"
            border.width: 1
            
            RowLayout {
                anchors.fill: parent
                anchors.margins: 16
                spacing: 12
                
                Item { Layout.fillWidth: true }
                
                Button {
                    text: "Cancel"
                    Layout.preferredWidth: 100
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
                    
                    onClicked: {
                        root.rejected()
                        root.close()
                    }
                }
                
                Button {
                    text: "Connect"
                    Layout.preferredWidth: 100
                    enabled: validate()
                    font.pixelSize: 12
                    font.bold: true
                    
                    background: Rectangle {
                        color: {
                            if (!parent.enabled) return "#3e3e42"
                            if (parent.pressed) return "#005a9e"
                            if (parent.hovered) return "#1f7bb8"
                            return "#007acc"
                        }
                        border.color: parent.enabled ? "#4fc3f7" : "#3e3e42"
                        border.width: 1
                        radius: 4
                    }
                    
                    contentItem: Label {
                        text: parent.text
                        font: parent.font
                        color: parent.enabled ? "#ffffff" : "#6a6a6a"
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                    
                    onClicked: {
                        root.accepted()
                        root.close()
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
    
    // Focus username field when opened
    onVisibilityChanged: {
        if (visible) {
            usernameField.forceActiveFocus()
            usernameField.selectAll()
        }
    }
}
