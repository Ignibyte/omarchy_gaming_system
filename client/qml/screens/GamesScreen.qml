import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components" as Components

Item {
    id: root

    required property var controller
    required property var sessionController

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
        anchors.margins: 18
        contentWidth: availableWidth

        ColumnLayout {
            width: scroll.availableWidth
            spacing: 12

            RowLayout {
                Layout.fillWidth: true

                Text {
                    Layout.fillWidth: true
                    text: "GAME CARTRIDGES"
                    textFormat: Text.PlainText
                    color: "#5ee6a8"
                    font.family: "monospace"
                    font.bold: true
                    font.pixelSize: 24
                }

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

            Text {
                Layout.fillWidth: true
                text: controller.errorText !== "" ? controller.errorText : controller.statusText
                textFormat: Text.PlainText
                color: controller.errorText !== "" ? "#ff6b7a" : "#8aa4c0"
                font.family: "monospace"
                font.pixelSize: 12
                wrapMode: Text.Wrap
            }

            Components.OgsButton {
                objectName: "gamesRetryButton"
                visible: controller.hasRetryableMutation
                text: "RETRY SAME OPERATION"
                accessibleName: "Retry the same game operation identity"
                enabled: !controller.busy
                onClicked: controller.retryPendingMutation()
            }

            Text {
                Layout.fillWidth: true
                text: "SOLO CARTRIDGES"
                textFormat: Text.PlainText
                color: "#f4c95d"
                font.family: "monospace"
                font.bold: true
            }

            Text {
                Layout.fillWidth: true
                visible: controller.loadState !== "loading" && controller.soloGames().length === 0
                text: "No solo cartridges are available."
                textFormat: Text.PlainText
                color: "#607890"
                font.family: "monospace"
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

            Text {
                Layout.fillWidth: true
                text: "INSTALLED CATALOG (" + controller.catalog.length + ")"
                textFormat: Text.PlainText
                color: "#f4c95d"
                font.family: "monospace"
                font.bold: true
            }

            Repeater {
                model: controller.catalog
                delegate: Rectangle {
                    required property var modelData
                    Layout.fillWidth: true
                    Layout.preferredHeight: 52
                    color: "#0c1825"
                    border.color: "#36516b"

                    Text {
                        anchors.fill: parent
                        anchors.margins: 10
                        text: modelData.display_name + " // v" + modelData.version + " // "
                              + modelData.min_human_players + "–" + modelData.max_human_players
                              + " PLAYERS // " + modelData.authority.toUpperCase()
                        textFormat: Text.PlainText
                        color: "#b7c9da"
                        font.family: "monospace"
                        wrapMode: Text.Wrap
                        verticalAlignment: Text.AlignVCenter
                    }
                }
            }

            Text {
                Layout.fillWidth: true
                text: "YOUR SESSIONS (" + controller.sessions.length + ")"
                textFormat: Text.PlainText
                color: "#f4c95d"
                font.family: "monospace"
                font.bold: true
            }

            Text {
                Layout.fillWidth: true
                visible: controller.loadState !== "loading" && controller.sessions.length === 0
                text: "No matches yet. Start a solo cartridge or accept a challenge."
                textFormat: Text.PlainText
                color: "#607890"
                font.family: "monospace"
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
