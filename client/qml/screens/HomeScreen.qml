import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components" as Components

Item {
    id: root

    required property var controller

    function focusInitial() {
        socialButton.forceActiveFocus()
    }

    ScrollView {
        id: scroll
        anchors.fill: parent
        anchors.margins: 28
        contentWidth: availableWidth

        ColumnLayout {
            width: scroll.availableWidth
            spacing: 18

            Item { Layout.fillHeight: true; Layout.minimumHeight: 16 }

            Text {
                Layout.fillWidth: true
                text: "PLAYER LINK READY"
                textFormat: Text.PlainText
                color: "#5ee6a8"
                font.family: "monospace"
                font.bold: true
                font.pixelSize: 30
                horizontalAlignment: Text.AlignHCenter
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.maximumWidth: 620
                Layout.preferredHeight: personaColumn.implicitHeight + 36
                Layout.alignment: Qt.AlignHCenter
                radius: 4
                color: "#0c1825"
                border.color: "#36516b"

                ColumnLayout {
                    id: personaColumn
                    anchors.fill: parent
                    anchors.margins: 18
                    spacing: 8

                    Text {
                        Layout.fillWidth: true
                        text: controller.selectedPersona
                              ? controller.selectedPersona.display_name : "No persona selected"
                        textFormat: Text.PlainText
                        color: "#eef7ff"
                        font.family: "monospace"
                        font.bold: true
                        font.pixelSize: 24
                        wrapMode: Text.Wrap
                    }

                    Text {
                        Layout.fillWidth: true
                        text: controller.selectedPersona
                              ? "@" + controller.selectedPersona.handle : ""
                        textFormat: Text.PlainText
                        color: "#f4c95d"
                        font.family: "monospace"
                        font.pixelSize: 15
                    }

                    Text {
                        Layout.fillWidth: true
                        text: controller.selectedPersona
                              ? controller.selectedPersona.status_message : ""
                        textFormat: Text.PlainText
                        color: "#b7c9da"
                        font.family: "monospace"
                        font.pixelSize: 14
                        wrapMode: Text.Wrap
                    }
                }
            }

            Text {
                Layout.fillWidth: true
                Layout.maximumWidth: 620
                Layout.alignment: Qt.AlignHCenter
                text: "Your persona can now manage connections and exchange durable private messages. Challenges and games arrive in the next client slice."
                textFormat: Text.PlainText
                color: "#8aa4c0"
                font.family: "monospace"
                font.pixelSize: 14
                wrapMode: Text.Wrap
                horizontalAlignment: Text.AlignHCenter
            }

            RowLayout {
                Layout.alignment: Qt.AlignHCenter
                spacing: 12

                Components.OgsButton {
                    id: socialButton
                    objectName: "homeSocialButton"
                    text: "SOCIAL"
                    accessibleName: "Open persona social link"
                    onClicked: controller.showPlayerScreen("social")
                }

                Components.OgsButton {
                    id: inboxButton
                    objectName: "homeInboxButton"
                    text: "INBOX"
                    accessibleName: "Open private inbox"
                    onClicked: controller.showPlayerScreen("inbox")
                }

                Components.OgsButton {
                    id: logoutButton
                    objectName: "homeLogoutButton"
                    text: "SIGN OUT"
                    accessibleName: "Sign out of OmarchyGS"
                    onClicked: controller.logout()
                }

                Components.OgsButton {
                    id: changeServerButton
                    objectName: "homeChangeServerButton"
                    text: "CHANGE SERVER"
                    accessibleName: "Sign out and change server"
                    onClicked: controller.showServerConfiguration()
                }
            }

            Text {
                Layout.fillWidth: true
                text: controller.serverUrl
                textFormat: Text.PlainText
                color: "#607890"
                font.family: "monospace"
                font.pixelSize: 11
                elide: Text.ElideMiddle
                horizontalAlignment: Text.AlignHCenter
            }

            Item { Layout.fillHeight: true; Layout.minimumHeight: 16 }
        }
    }
}
