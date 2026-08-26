import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components" as Components

Item {
    id: root

    required property var controller
    required property var sessionController

    Components.OgsTheme { id: theme }

    Connections {
        target: root.controller
        function onReportSubmitted() {
            reportHandleField.clear()
            reportDetail.clear()
            reportCategory.currentIndex = 0
        }
    }

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
        anchors.margins: theme.spaceLg
        contentWidth: availableWidth

        ColumnLayout {
            width: scroll.availableWidth
            spacing: theme.spaceMd

            Components.OgsScreenHeader {
                Layout.fillWidth: true
                screenKey: "social"
                title: "SOCIAL LINK"
                statusText: controller.statusText
                statusTone: controller.busy || controller.loadState === "loading"
                            ? "working" : "success"
                errorText: controller.errorText
                navigationHint: "ENTER CONNECT // ESC HOME"
            }

            RowLayout {
                Layout.alignment: Qt.AlignRight
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

            Components.OgsSectionLabel {
                Layout.fillWidth: true
                text: "REPORT PERSONA"
            }

            Text {
                Layout.fillWidth: true
                text: "Reports go to this server's operator. Include only the detail needed for review."
                textFormat: Text.PlainText
                wrapMode: Text.Wrap
                color: theme.textMuted
                font.family: theme.fontFamily
                font.pixelSize: theme.bodySize
            }

            Components.OgsTextField {
                id: reportHandleField
                objectName: "reportHandleField"
                Layout.fillWidth: true
                accessibleName: "Exact persona handle to report"
                placeholderText: "exact_handle"
                maximumLength: 24
                enabled: !controller.busy
            }

            ComboBox {
                id: reportCategory
                objectName: "reportCategoryBox"
                Layout.fillWidth: true
                Accessible.name: "Report category"
                activeFocusOnTab: true
                focusPolicy: Qt.StrongFocus
                enabled: !controller.busy
                model: ["HARASSMENT", "SPAM", "CHEATING", "OTHER"]
                property var categoryValues: ["harassment", "spam", "cheating", "other"]
            }

            Components.OgsTextArea {
                id: reportDetail
                objectName: "reportDetailField"
                Layout.fillWidth: true
                accessibleName: "Report details"
                placeholderText: "Briefly describe what happened"
                maximumLength: 1000
                enabled: !controller.busy
            }

            Components.OgsButton {
                objectName: "reportSubmitButton"
                Layout.alignment: Qt.AlignRight
                text: "REPORT"
                accessibleName: "Submit persona report for operator review"
                enabled: !controller.busy
                onClicked: controller.reportPersonaByHandle(
                    reportHandleField.text,
                    reportCategory.categoryValues[reportCategory.currentIndex],
                    reportDetail.text)
            }

            Components.OgsSectionLabel {
                Layout.fillWidth: true
                text: "INCOMING REQUESTS (" + controller.incomingRequests.length + ")"
            }
            Text {
                Layout.fillWidth: true
                visible: controller.loadState !== "loading" && controller.incomingRequests.length === 0
                text: "No incoming requests."
                textFormat: Text.PlainText
                color: theme.textMuted
                font.family: theme.fontFamily
                font.pixelSize: theme.bodySize
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

            Components.OgsSectionLabel {
                Layout.fillWidth: true
                text: "OUTGOING REQUESTS (" + controller.outgoingRequests.length + ")"
            }
            Text {
                Layout.fillWidth: true
                visible: controller.loadState !== "loading" && controller.outgoingRequests.length === 0
                text: "No outgoing requests."
                textFormat: Text.PlainText
                color: theme.textMuted
                font.family: theme.fontFamily
                font.pixelSize: theme.bodySize
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

            Components.OgsSectionLabel {
                Layout.fillWidth: true
                text: "CONNECTIONS (" + controller.connections.length + ")"
            }
            Text {
                Layout.fillWidth: true
                visible: controller.loadState !== "loading" && controller.connections.length === 0
                text: "No accepted connections."
                textFormat: Text.PlainText
                color: theme.textMuted
                font.family: theme.fontFamily
                font.pixelSize: theme.bodySize
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

            Components.OgsSectionLabel {
                Layout.fillWidth: true
                text: "PRIVATE BLOCKS (" + controller.blocks.length + ")"
            }
            Text {
                Layout.fillWidth: true
                visible: controller.loadState !== "loading" && controller.blocks.length === 0
                text: "No personas blocked by this persona."
                textFormat: Text.PlainText
                color: theme.textMuted
                font.family: theme.fontFamily
                font.pixelSize: theme.bodySize
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
