import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components" as Components

Item {
    id: root

    required property var controller

    Components.OgsTheme { id: theme }

    function focusInitial() {
        endpointField.forceActiveFocus()
    }

    Keys.onEscapePressed: function(event) {
        endpointField.text = controller.serverUrl
        event.accepted = true
    }

    ScrollView {
        id: scroll
        anchors.fill: parent
        anchors.margins: theme.space2Xl
        contentWidth: availableWidth

        ColumnLayout {
            width: scroll.availableWidth
            spacing: theme.spaceLg

            Item { Layout.fillHeight: true; Layout.minimumHeight: 12 }

            Components.OgsScreenHeader {
                Layout.fillWidth: true
                screenKey: "connection"
                title: "LINK A SERVER"
                statusText: controller.statusText
                statusTone: controller.busy ? "working"
                            : controller.connectionState === "ready" ? "success" : "info"
                errorText: controller.errorText
                navigationHint: "ENTER CONNECT // ESC RESTORE SAVED SERVER"
            }

            Components.OgsSectionLabel {
                Layout.fillWidth: true
                Layout.maximumWidth: 620
                Layout.alignment: Qt.AlignHCenter
                text: "SERVER URL"
                tone: "info"
            }

            Components.OgsTextField {
                id: endpointField
                objectName: "serverUrlField"
                Layout.fillWidth: true
                Layout.maximumWidth: 620
                Layout.alignment: Qt.AlignHCenter
                accessibleName: "OmarchyGS server URL"
                text: controller.serverUrl
                placeholderText: "https://games.example.net"
                enabled: !controller.busy
                onAccepted: connectButton.clicked()
            }

            Components.OgsButton {
                id: connectButton
                objectName: "connectButton"
                Layout.alignment: Qt.AlignHCenter
                Layout.minimumWidth: 220
                text: controller.busy ? "CHECKING..." : "CONNECT"
                accessibleName: "Connect to OmarchyGS server"
                enabled: !controller.busy
                onClicked: controller.connectToServer(endpointField.text)
            }

            Text {
                Layout.fillWidth: true
                Layout.maximumWidth: 620
                Layout.alignment: Qt.AlignHCenter
                text: "Remote servers require HTTPS. Plain HTTP is limited to localhost."
                textFormat: Text.PlainText
                color: theme.textMuted
                font.family: theme.fontFamily
                font.pixelSize: theme.captionSize
                wrapMode: Text.Wrap
                horizontalAlignment: Text.AlignHCenter
            }

            Item { Layout.fillHeight: true; Layout.minimumHeight: 12 }
        }
    }
}
