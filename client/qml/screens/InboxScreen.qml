import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components" as Components

Item {
    id: root

    required property var controller
    required property var sessionController

    function focusInitial() {
        if (controller.selectedConversation)
            composer.forceActiveFocus()
        else
            inboxRefreshButton.forceActiveFocus()
    }

    Keys.onEscapePressed: function(event) {
        if (controller.selectedConversation)
            controller.closeConversation()
        else
            sessionController.showPlayerScreen("home")
        event.accepted = true
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 18
        spacing: 10

        RowLayout {
            Layout.fillWidth: true

            Text {
                Layout.fillWidth: true
                text: controller.selectedConversation
                      ? "PRIVATE LINK // @" + controller.selectedConversation.other_persona.handle
                      : "PRIVATE INBOX"
                textFormat: Text.PlainText
                color: "#5ee6a8"
                font.family: "monospace"
                font.bold: true
                font.pixelSize: 22
                elide: Text.ElideRight
            }

            Components.OgsButton {
                id: inboxRefreshButton
                objectName: "inboxRefreshButton"
                text: controller.selectedConversation ? "INBOX" : "REFRESH"
                accessibleName: controller.selectedConversation
                                ? "Return to conversation list" : "Refresh private inbox"
                enabled: !controller.busy
                onClicked: {
                    if (controller.selectedConversation)
                        controller.closeConversation()
                    else
                        controller.refreshInbox()
                }
            }

            Components.OgsButton {
                objectName: "inboxHomeButton"
                text: "HOME"
                accessibleName: "Return to player home"
                enabled: !controller.busy
                onClicked: sessionController.showPlayerScreen("home")
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

        ListView {
            id: conversationList
            objectName: "conversationList"
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: !controller.selectedConversation
            spacing: 8
            clip: true
            model: controller.conversations

            delegate: Components.OgsButton {
                required property int index
                required property var modelData
                width: ListView.view.width
                objectName: "conversationButton" + index
                text: "@" + modelData.other_persona.handle + "  //  "
                      + modelData.unread_count + " UNREAD"
                accessibleName: "Open conversation with " + modelData.other_persona.display_name
                                + ", " + modelData.unread_count + " unread"
                enabled: !controller.busy
                onClicked: controller.openConversation(modelData)
            }

            Text {
                anchors.centerIn: parent
                width: parent.width
                visible: controller.loadState !== "loading" && controller.conversations.length === 0
                text: "No private conversations. Accept a connection to create one."
                textFormat: Text.PlainText
                color: "#607890"
                font.family: "monospace"
                font.pixelSize: 14
                wrapMode: Text.Wrap
                horizontalAlignment: Text.AlignHCenter
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: !!controller.selectedConversation
            spacing: 8

            Components.OgsButton {
                objectName: "loadOlderMessagesButton"
                Layout.alignment: Qt.AlignHCenter
                visible: controller.nextBefore !== null
                text: "LOAD OLDER"
                accessibleName: "Load older private messages"
                enabled: !controller.busy
                onClicked: controller.loadOlderMessages()
            }

            ListView {
                id: messageList
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: 6
                clip: true
                model: controller.messages

                delegate: Rectangle {
                    required property var modelData
                    width: ListView.view.width
                    height: messageText.implicitHeight + 18
                    radius: 3
                    color: modelData.type === "system" ? "#172333" : "#0c1825"
                    border.color: "#29445e"

                    Text {
                        id: messageText
                        anchors.fill: parent
                        anchors.margins: 9
                        text: controller.messageText(modelData)
                        textFormat: Text.PlainText
                        color: modelData.type === "system" ? "#f4c95d" : "#eef7ff"
                        font.family: "monospace"
                        font.pixelSize: 13
                        wrapMode: Text.Wrap
                    }
                }
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: 10

                ScrollView {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 82

                    Components.OgsTextArea {
                        id: composer
                        objectName: "messageComposer"
                        width: parent.width
                        accessibleName: "Private message body"
                        placeholderText: "Write a private message"
                        maximumLength: 4000
                        enabled: !controller.busy
                    }
                }

                Components.OgsButton {
                    objectName: "sendMessageButton"
                    text: "SEND"
                    accessibleName: "Send private message"
                    enabled: !controller.busy
                    onClicked: {
                        if (controller.sendMessage(composer.text))
                            composer.clear()
                    }
                }
            }
        }
    }
}
