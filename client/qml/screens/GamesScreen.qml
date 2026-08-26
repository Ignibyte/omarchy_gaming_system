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
        refreshButton.forceActiveFocus()
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
                screenKey: "games"
                title: "GAME CARTRIDGES"
                statusText: controller.statusText
                statusTone: controller.busy || controller.loadState === "loading"
                            ? "working" : "success"
                errorText: controller.errorText
                navigationHint: "ESC HOME // ENTER START OR OPEN"
            }

            RowLayout {
                Layout.alignment: Qt.AlignRight
                Components.OgsButton {
                    id: refreshButton
                    objectName: "gamesRefreshButton"
                    text: "REFRESH"
                    accessibleName: "Refresh game cartridges and sessions"
                    enabled: !controller.busy
                    onClicked: controller.refreshGames()
                }

                Components.OgsButton {
                    objectName: "gamesChallengesButton"
                    text: "CHALLENGES"
                    accessibleName: "Open game challenges"
                    enabled: !controller.busy
                    onClicked: sessionController.showPlayerScreen("challenges")
                }

                Components.OgsButton {
                    objectName: "gamesHomeButton"
                    text: "HOME"
                    accessibleName: "Return to player home"
                    enabled: !controller.busy
                    onClicked: sessionController.showPlayerScreen("home")
                }
            }

            Components.OgsButton {
                objectName: "gamesRetryButton"
                visible: controller.hasRetryableMutation
                text: "RETRY SAME OPERATION"
                accessibleName: "Retry the same game operation identity"
                enabled: !controller.busy
                onClicked: controller.retryPendingMutation()
            }

            Components.OgsSectionLabel {
                Layout.fillWidth: true
                text: "SOLO CARTRIDGES"
            }

            Text {
                Layout.fillWidth: true
                visible: controller.loadState !== "loading" && controller.soloGames().length === 0
                text: "No solo cartridges are available."
                textFormat: Text.PlainText
                color: theme.textMuted
                font.family: theme.fontFamily
                font.pixelSize: theme.bodySize
            }

            Repeater {
                model: controller.soloGames()
                delegate: Components.OgsButton {
                    required property var modelData
                    Layout.fillWidth: true
                    text: "START " + modelData.display_name.toUpperCase()
                    accessibleName: "Start " + modelData.display_name + " version " + modelData.version
                    enabled: !controller.busy
                    onClicked: controller.startSolo(modelData)
                }
            }

            Components.OgsSectionLabel {
                Layout.fillWidth: true
                text: "INSTALLED CATALOG (" + controller.catalog.length + ")"
            }

            Repeater {
                model: controller.catalog
                delegate: Components.OgsCard {
                    required property var modelData
                    Layout.fillWidth: true
                    Layout.preferredHeight: 52
                    tone: "info"

                    Text {
                        anchors.fill: parent
                        anchors.margins: 10
                        text: modelData.display_name + " // v" + modelData.version + " // "
                              + modelData.min_human_players + "–" + modelData.max_human_players
                              + " PLAYERS // " + modelData.authority.toUpperCase()
                        textFormat: Text.PlainText
                        color: theme.textSecondary
                        font.family: theme.fontFamily
                        font.pixelSize: theme.bodySize
                        wrapMode: Text.Wrap
                        verticalAlignment: Text.AlignVCenter
                    }
                }
            }

            Components.OgsSectionLabel {
                Layout.fillWidth: true
                text: "YOUR SESSIONS (" + controller.sessions.length + ")"
            }

            Text {
                Layout.fillWidth: true
                visible: controller.loadState !== "loading" && controller.sessions.length === 0
                text: "No matches yet. Start a solo cartridge or accept a challenge."
                textFormat: Text.PlainText
                color: theme.textMuted
                font.family: theme.fontFamily
                font.pixelSize: theme.bodySize
                wrapMode: Text.Wrap
            }

            Repeater {
                model: controller.sessions
                delegate: Components.OgsButton {
                    required property var modelData
                    Layout.fillWidth: true
                    text: controller.gameName(modelData.game_key, modelData.game_version).toUpperCase()
                          + " // " + modelData.status.toUpperCase() + " // REV " + modelData.revision
                    accessibleName: "Open " + controller.gameName(modelData.game_key, modelData.game_version)
                                    + " " + modelData.status + " session"
                    enabled: !controller.busy
                    onClicked: controller.openSession(modelData)
                }
            }
        }
    }
}
