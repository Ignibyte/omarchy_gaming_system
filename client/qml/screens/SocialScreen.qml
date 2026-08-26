import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components" as Components

Item {
    id: root

    required property var controller
    required property var sessionController

    function focusInitial() {
        handleField.forceActiveFocus()
    }

    Keys.onEscapePressed: function(event) {
        sessionController.showPlayerScreen("home")
        event.accepted = true
    }

    ScrollView {
        id: scroll
        anchors.fill: parent
        anchors.margins: 18
        contentWidth: availableWidth

        ColumnLayout {
            width: scroll.availableWidth
            spacing: 12

            RowLayout {
                Layout.fillWidth: true

                Text {
                    Layout.fillWidth: true
                    text: "SOCIAL LINK"
                    textFormat: Text.PlainText
                    color: "#5ee6a8"
                    font.family: "monospace"
                    font.bold: true
                    font.pixelSize: 24
                }

                Components.OgsButton {
                    objectName: "socialRefreshButton"
                    text: "REFRESH"
                    accessibleName: "Refresh social state"
                    enabled: !controller.busy
                    onClicked: controller.refreshSocial()
                }

                Components.OgsButton {
                    objectName: "socialBackButton"
                    text: "HOME"
                    accessibleName: "Return to player home"
                    enabled: !controller.busy
                    onClicked: sessionController.showPlayerScreen("home")
                }
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: 10

                Components.OgsTextField {
                    id: handleField
                    objectName: "socialHandleField"
                    Layout.fillWidth: true
                    accessibleName: "Exact persona handle"
                    placeholderText: "exact_handle"
                    maximumLength: 24
                    enabled: !controller.busy
                    onAccepted: requestButton.clicked()
                }

                Components.OgsButton {
                    id: requestButton
                    objectName: "socialRequestButton"
                    text: "CONNECT"
                    accessibleName: "Request connection by exact handle"
                    enabled: !controller.busy
                    onClicked: {
                        if (controller.requestConnectionByHandle(handleField.text))
                            handleField.clear()
                    }
                }
            }

            Text {
                Layout.fillWidth: true
                text: controller.errorText !== "" ? controller.errorText : controller.statusText
                textFormat: Text.PlainText
                color: controller.errorText !== "" ? "#ff6b7a" : "#8aa4c0"
                font.family: "monospace"
                font.pixelSize: 12
                wrapMode: Text.Wrap
            }

            Text {
                Layout.fillWidth: true
                text: "INCOMING REQUESTS (" + controller.incomingRequests.length + ")"
                textFormat: Text.PlainText
                color: "#f4c95d"
                font.family: "monospace"
                font.bold: true
                font.pixelSize: 14
            }
            Text {
                Layout.fillWidth: true
                visible: controller.loadState !== "loading" && controller.incomingRequests.length === 0
                text: "No incoming requests."
                textFormat: Text.PlainText
                color: "#607890"
                font.family: "monospace"
            }
            ListView {
                id: incomingList
                objectName: "incomingRequestList"
                Layout.fillWidth: true
                Layout.preferredHeight: contentHeight
                interactive: false
                spacing: 6
                model: controller.incomingRequests
                delegate: Components.SocialRow {
                    required property int index
                    required property var modelData
                    property var entry: modelData
                    width: ListView.view.width
                    persona: entry.persona
                    detail: "received " + entry.created_at
                    primaryText: "ACCEPT"
                    secondaryText: "DECLINE"
                    actionsEnabled: !controller.busy
                    onPrimaryTriggered: controller.acceptRequest(entry.persona)
                    onSecondaryTriggered: controller.removeRelationship(entry.persona)
                }
            }

            Text {
                Layout.fillWidth: true
                text: "OUTGOING REQUESTS (" + controller.outgoingRequests.length + ")"
                textFormat: Text.PlainText
                color: "#f4c95d"
                font.family: "monospace"
                font.bold: true
                font.pixelSize: 14
            }
            Text {
                Layout.fillWidth: true
                visible: controller.loadState !== "loading" && controller.outgoingRequests.length === 0
                text: "No outgoing requests."
                textFormat: Text.PlainText
                color: "#607890"
                font.family: "monospace"
            }
            ListView {
                id: outgoingList
                objectName: "outgoingRequestList"
                Layout.fillWidth: true
                Layout.preferredHeight: contentHeight
                interactive: false
                spacing: 6
                model: controller.outgoingRequests
                delegate: Components.SocialRow {
                    required property int index
                    required property var modelData
                    property var entry: modelData
                    width: ListView.view.width
                    persona: entry.persona
                    detail: "sent " + entry.created_at
                    primaryText: "CANCEL"
                    actionsEnabled: !controller.busy
                    onPrimaryTriggered: controller.removeRelationship(entry.persona)
                }
            }

            Text {
                Layout.fillWidth: true
                text: "CONNECTIONS (" + controller.connections.length + ")"
                textFormat: Text.PlainText
                color: "#f4c95d"
                font.family: "monospace"
                font.bold: true
                font.pixelSize: 14
            }
            Text {
                Layout.fillWidth: true
                visible: controller.loadState !== "loading" && controller.connections.length === 0
                text: "No accepted connections."
                textFormat: Text.PlainText
                color: "#607890"
                font.family: "monospace"
            }
            ListView {
                id: connectionList
                objectName: "connectionList"
                Layout.fillWidth: true
                Layout.preferredHeight: contentHeight
                interactive: false
                spacing: 6
                model: controller.connections
                delegate: Components.SocialRow {
                    required property int index
                    required property var modelData
                    property var entry: modelData
                    width: ListView.view.width
                    persona: entry.persona
                    detail: "connected " + entry.connected_at
                    primaryText: "REMOVE"
                    secondaryText: "BLOCK"
                    actionsEnabled: !controller.busy
                    onPrimaryTriggered: controller.removeRelationship(entry.persona)
                    onSecondaryTriggered: controller.blockPersona(entry.persona)
                }
            }

            Text {
                Layout.fillWidth: true
                text: "PRIVATE BLOCKS (" + controller.blocks.length + ")"
                textFormat: Text.PlainText
                color: "#f4c95d"
                font.family: "monospace"
                font.bold: true
                font.pixelSize: 14
            }
            Text {
                Layout.fillWidth: true
                visible: controller.loadState !== "loading" && controller.blocks.length === 0
                text: "No personas blocked by this persona."
                textFormat: Text.PlainText
                color: "#607890"
                font.family: "monospace"
            }
            ListView {
                id: blockList
                objectName: "blockList"
                Layout.fillWidth: true
                Layout.preferredHeight: contentHeight
                interactive: false
                spacing: 6
                model: controller.blocks
                delegate: Components.SocialRow {
                    required property int index
                    required property var modelData
                    property var entry: modelData
                    width: ListView.view.width
                    persona: entry.persona
                    detail: "blocked " + entry.created_at
                    primaryText: "UNBLOCK"
                    actionsEnabled: !controller.busy
                    onPrimaryTriggered: controller.unblockPersona(entry.persona)
                }
            }
        }
    }
}
