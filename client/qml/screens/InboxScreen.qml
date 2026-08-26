import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components" as Components

Item {
    id: root

    required property var controller
    required property var sessionController

    Components.OgsTheme { id: theme }

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
        anchors.margins: theme.spaceLg
        spacing: theme.spaceSm

        Components.OgsScreenHeader {
            Layout.fillWidth: true
            screenKey: "inbox"
            title: controller.selectedConversation
                   ? "PRIVATE LINK // @" + controller.selectedConversation.other_persona.handle
                   : "PRIVATE INBOX"
            statusText: controller.statusText
            statusTone: controller.busy || controller.loadState === "loading"
                        ? "working" : "success"
            errorText: controller.errorText
            navigationHint: controller.selectedConversation ? "ESC INBOX // TAB MESSAGE ACTIONS"
                                                            : "ESC HOME // ENTER OPEN THREAD"
        }

        RowLayout {
            Layout.alignment: Qt.AlignRight
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
                color: theme.textMuted
                font.family: theme.fontFamily
                font.pixelSize: theme.bodySize
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

                delegate: Components.OgsCard {
                    required property var modelData
                    width: ListView.view.width
                    height: messageText.implicitHeight + 18
                    tone: modelData.type === "system" ? "warning" : "info"
                    highlighted: modelData.type === "system"

                    Text {
                        id: messageText
                        anchors.fill: parent
                        anchors.margins: 9
                        text: controller.messageText(modelData)
                        textFormat: Text.PlainText
                        color: modelData.type === "system" ? theme.warning : theme.textPrimary
                        font.family: theme.fontFamily
                        font.pixelSize: theme.bodySize
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
