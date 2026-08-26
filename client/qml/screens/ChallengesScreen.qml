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
        sessionController.showPlayerScreen("games")
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
                    text: "GAME CHALLENGES"
                    textFormat: Text.PlainText
                    color: "#5ee6a8"
                    font.family: "monospace"
                    font.bold: true
                    font.pixelSize: 24
                }

                Components.OgsButton {
                    id: refreshButton
                    objectName: "challengesRefreshButton"
                    text: "REFRESH"
                    accessibleName: "Refresh game challenges"
                    enabled: !controller.busy
                    onClicked: controller.refreshChallenges()
                }

                Components.OgsButton {
                    objectName: "challengesGamesButton"
                    text: "GAMES"
                    accessibleName: "Return to game cartridges"
                    enabled: !controller.busy
                    onClicked: sessionController.showPlayerScreen("games")
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
                visible: controller.hasRetryableMutation
                text: "RETRY SAME OPERATION"
                accessibleName: "Retry the same challenge operation identity"
                enabled: !controller.busy
                onClicked: controller.retryPendingMutation()
            }

            Text {
                Layout.fillWidth: true
                text: "CHALLENGE A CONNECTION"
                textFormat: Text.PlainText
                color: "#f4c95d"
                font.family: "monospace"
                font.bold: true
            }

            Text {
                Layout.fillWidth: true
                visible: controller.loadState !== "loading"
                         && (controller.connections.length === 0 || controller.challengeGames().length === 0)
                text: controller.connections.length === 0 ? "Connect with another persona first."
                      : "No two-player cartridge is available."
                textFormat: Text.PlainText
                color: "#607890"
                font.family: "monospace"
            }

            Repeater {
                model: controller.connections
                delegate: RowLayout {
                    required property var modelData
                    Layout.fillWidth: true

                    Text {
                        Layout.fillWidth: true
                        text: "@" + modelData.persona.handle
                        textFormat: Text.PlainText
                        color: "#b7c9da"
                        font.family: "monospace"
                    }

                    Components.OgsButton {
                        text: controller.challengeGames().length > 0
                              ? "CHALLENGE // " + controller.challengeGames()[0].display_name.toUpperCase()
                              : "NO VERSUS GAME"
                        accessibleName: "Challenge " + modelData.persona.display_name
                        enabled: !controller.busy && controller.challengeGames().length > 0
                        onClicked: controller.createChallenge(modelData, controller.challengeGames()[0])
                    }
                }
            }

            Text {
                Layout.fillWidth: true
                text: "CHALLENGE HISTORY (" + controller.challenges.length + ")"
                textFormat: Text.PlainText
                color: "#f4c95d"
                font.family: "monospace"
                font.bold: true
            }

            Text {
                Layout.fillWidth: true
                visible: controller.loadState !== "loading" && controller.challenges.length === 0
                text: "No challenges yet."
                textFormat: Text.PlainText
                color: "#607890"
                font.family: "monospace"
            }

            Repeater {
                model: controller.challenges
                delegate: Rectangle {
                    required property var modelData
                    Layout.fillWidth: true
                    Layout.preferredHeight: challengeColumn.implicitHeight + 20
                    color: "#0c1825"
                    border.color: "#36516b"

                    ColumnLayout {
                        id: challengeColumn
                        anchors.fill: parent
                        anchors.margins: 10
                        spacing: 7

                        Text {
                            Layout.fillWidth: true
                            text: controller.gameName(modelData.game_key, modelData.game_version)
                                  + " // @" + controller.otherChallengePersona(modelData).handle
                                  + " // " + modelData.direction.toUpperCase()
                                  + " // " + modelData.status.toUpperCase()
                            textFormat: Text.PlainText
                            color: "#eef7ff"
                            font.family: "monospace"
                            font.bold: true
                            wrapMode: Text.Wrap
                        }

                        RowLayout {
                            Layout.fillWidth: true

                            Components.OgsButton {
                                visible: modelData.status === "pending" && modelData.direction === "incoming"
                                text: "ACCEPT"
                                accessibleName: "Accept challenge from "
                                                + controller.otherChallengePersona(modelData).display_name
                                enabled: !controller.busy
                                onClicked: controller.acceptChallenge(modelData)
                            }

                            Components.OgsButton {
                                visible: modelData.status === "pending" && modelData.direction === "incoming"
                                text: "DECLINE"
                                accessibleName: "Decline challenge from "
                                                + controller.otherChallengePersona(modelData).display_name
                                enabled: !controller.busy
                                onClicked: controller.declineChallenge(modelData)
                            }

                            Components.OgsButton {
                                visible: modelData.status === "pending" && modelData.direction === "outgoing"
                                text: "CANCEL"
                                accessibleName: "Cancel challenge to "
                                                + controller.otherChallengePersona(modelData).display_name
                                enabled: !controller.busy
                                onClicked: controller.cancelChallenge(modelData)
                            }

                            Components.OgsButton {
                                visible: modelData.status === "accepted"
                                text: "OPEN MATCH"
                                accessibleName: "Open accepted match with "
                                                + controller.otherChallengePersona(modelData).display_name
                                enabled: !controller.busy
                                onClicked: controller.openChallengeSession(modelData)
                            }
                        }
                    }
                }
            }

            Components.OgsButton {
                objectName: "challengesOlderButton"
                visible: controller.nextChallengeBefore !== null
                text: "LOAD OLDER"
                accessibleName: "Load older game challenges"
                enabled: !controller.busy
                onClicked: controller.loadOlderChallenges()
            }
        }
    }
}
