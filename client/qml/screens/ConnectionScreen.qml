import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components" as Components

Item {
    id: root

    required property var controller

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
        anchors.margins: 28
        contentWidth: availableWidth

        ColumnLayout {
            width: scroll.availableWidth
            spacing: 16

            Item { Layout.fillHeight: true; Layout.minimumHeight: 12 }

            Text {
                Layout.fillWidth: true
                text: "LINK A SERVER"
                textFormat: Text.PlainText
                color: "#5ee6a8"
                font.family: "monospace"
                font.bold: true
                font.pixelSize: 30
                horizontalAlignment: Text.AlignHCenter
            }

            Text {
                Layout.fillWidth: true
                text: controller.statusText
                textFormat: Text.PlainText
                color: "#d5e2ef"
                font.family: "monospace"
                font.pixelSize: 16
                wrapMode: Text.Wrap
                horizontalAlignment: Text.AlignHCenter
            }

            Text {
                Layout.fillWidth: true
                visible: controller.errorText !== ""
                text: controller.errorText
                textFormat: Text.PlainText
                color: "#ff8b98"
                font.family: "monospace"
                font.pixelSize: 14
                wrapMode: Text.Wrap
                horizontalAlignment: Text.AlignHCenter
            }

            Text {
                Layout.fillWidth: true
                Layout.maximumWidth: 620
                Layout.alignment: Qt.AlignHCenter
                text: "SERVER URL"
                textFormat: Text.PlainText
                color: "#8aa4c0"
                font.family: "monospace"
                font.pixelSize: 12
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
                color: "#6f879f"
                font.family: "monospace"
                font.pixelSize: 12
                wrapMode: Text.Wrap
                horizontalAlignment: Text.AlignHCenter
            }

            Item { Layout.fillHeight: true; Layout.minimumHeight: 12 }
        }
    }
}
