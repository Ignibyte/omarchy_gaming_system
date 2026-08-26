import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components" as Components

Item {
    id: root

    required property var controller

    Components.OgsTheme { id: theme }

    function focusInitial() {
        gamesButton.forceActiveFocus()
    }

    ScrollView {
        id: scroll
        anchors.fill: parent
        anchors.margins: theme.space2Xl
        contentWidth: availableWidth

        ColumnLayout {
            width: scroll.availableWidth
            spacing: theme.spaceLg

            Item { Layout.fillHeight: true; Layout.minimumHeight: 16 }

            Components.OgsScreenHeader {
                Layout.fillWidth: true
                screenKey: "home"
                title: "PLAYER LINK READY"
                statusText: controller.statusText
                statusTone: "success"
                errorText: controller.errorText
                navigationHint: "TAB CHOOSE DESTINATION // ENTER OPEN"
            }

            Components.OgsCard {
                Layout.fillWidth: true
                Layout.maximumWidth: 620
                Layout.preferredHeight: personaColumn.implicitHeight + 36
                Layout.alignment: Qt.AlignHCenter
                tone: "success"
                highlighted: true

                ColumnLayout {
                    id: personaColumn
                    anchors.fill: parent
                    anchors.margins: 18
                    spacing: theme.spaceSm

                    Text {
                        Layout.fillWidth: true
                        text: controller.selectedPersona
                              ? controller.selectedPersona.display_name : "No persona selected"
                        textFormat: Text.PlainText
                        color: theme.textPrimary
                        font.family: theme.fontFamily
                        font.bold: true
                        font.pixelSize: theme.titleSize
                        wrapMode: Text.Wrap
                    }

                    Text {
                        Layout.fillWidth: true
                        text: controller.selectedPersona
                              ? "@" + controller.selectedPersona.handle : ""
                        textFormat: Text.PlainText
                        color: theme.warning
                        font.family: theme.fontFamily
                        font.pixelSize: theme.sectionSize
                    }

                    Text {
                        Layout.fillWidth: true
                        text: controller.selectedPersona
                              ? controller.selectedPersona.status_message : ""
                        textFormat: Text.PlainText
                        color: theme.textSecondary
                        font.family: theme.fontFamily
                        font.pixelSize: theme.bodySize
                        wrapMode: Text.Wrap
                    }
                }
            }

            Text {
                Layout.fillWidth: true
                Layout.maximumWidth: 620
                Layout.alignment: Qt.AlignHCenter
                text: "Load a game cartridge, challenge a connection, or continue a durable match. Social and inbox links remain available."
                textFormat: Text.PlainText
                color: theme.textMuted
                font.family: theme.fontFamily
                font.pixelSize: theme.bodySize
                wrapMode: Text.Wrap
                horizontalAlignment: Text.AlignHCenter
            }

            GridLayout {
                Layout.fillWidth: true
                Layout.maximumWidth: 620
                Layout.alignment: Qt.AlignHCenter
                columns: 3
                columnSpacing: 12
                rowSpacing: 12

                Components.OgsButton {
                    id: gamesButton
                    objectName: "homeGamesButton"
                    Layout.fillWidth: true
                    text: "GAMES"
                    accessibleName: "Open game cartridges and matches"
                    onClicked: controller.showPlayerScreen("games")
                }

                Components.OgsButton {
                    id: challengesButton
                    objectName: "homeChallengesButton"
                    Layout.fillWidth: true
                    text: "CHALLENGES"
                    accessibleName: "Open game challenges"
                    onClicked: controller.showPlayerScreen("challenges")
                }

                Components.OgsButton {
                    id: socialButton
                    objectName: "homeSocialButton"
                    Layout.fillWidth: true
                    text: "SOCIAL"
                    accessibleName: "Open persona social link"
                    onClicked: controller.showPlayerScreen("social")
                }

                Components.OgsButton {
                    id: inboxButton
                    objectName: "homeInboxButton"
                    Layout.fillWidth: true
                    text: "INBOX"
                    accessibleName: "Open private inbox"
                    onClicked: controller.showPlayerScreen("inbox")
                }

                Components.OgsButton {
                    id: logoutButton
                    objectName: "homeLogoutButton"
                    Layout.fillWidth: true
                    text: "SIGN OUT"
                    accessibleName: "Sign out of OmarchyGS"
                    onClicked: controller.logout()
                }

                Components.OgsButton {
                    id: changeServerButton
                    objectName: "homeChangeServerButton"
                    Layout.fillWidth: true
                    text: "CHANGE SERVER"
                    accessibleName: "Sign out and change server"
                    onClicked: controller.showServerConfiguration()
                }
            }

            Text {
                Layout.fillWidth: true
                text: controller.serverUrl
                textFormat: Text.PlainText
                color: theme.textMuted
                font.family: theme.fontFamily
                font.pixelSize: theme.captionSize
                elide: Text.ElideMiddle
                horizontalAlignment: Text.AlignHCenter
            }

            Item { Layout.fillHeight: true; Layout.minimumHeight: 16 }
        }
    }
}
