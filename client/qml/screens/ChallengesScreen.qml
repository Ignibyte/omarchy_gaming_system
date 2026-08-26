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
        sessionController.showPlayerScreen("games")
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
                screenKey: "challenges"
                title: "GAME CHALLENGES"
                statusText: controller.statusText
                statusTone: controller.busy || controller.loadState === "loading"
                            ? "working" : "success"
                errorText: controller.errorText
                navigationHint: "ESC GAMES // ENTER CHALLENGE ACTION"
            }

            RowLayout {
                Layout.alignment: Qt.AlignRight
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

            Components.OgsButton {
                visible: controller.hasRetryableMutation
                text: "RETRY SAME OPERATION"
                accessibleName: "Retry the same challenge operation identity"
                enabled: !controller.busy
                onClicked: controller.retryPendingMutation()
            }

            Components.OgsSectionLabel {
                Layout.fillWidth: true
                text: "CHALLENGE A CONNECTION"
            }

            Text {
                Layout.fillWidth: true
                visible: controller.loadState !== "loading"
                         && (controller.connections.length === 0 || controller.challengeGames().length === 0)
                text: controller.connections.length === 0 ? "Connect with another persona first."
                      : "No two-player cartridge is available."
                textFormat: Text.PlainText
                color: theme.textMuted
                font.family: theme.fontFamily
                font.pixelSize: theme.bodySize
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
                        color: theme.textSecondary
                        font.family: theme.fontFamily
                        font.pixelSize: theme.bodySize
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

            Components.OgsSectionLabel {
                Layout.fillWidth: true
                text: "CHALLENGE HISTORY (" + controller.challenges.length + ")"
            }

            Text {
                Layout.fillWidth: true
                visible: controller.loadState !== "loading" && controller.challenges.length === 0
                text: "No challenges yet."
                textFormat: Text.PlainText
                color: theme.textMuted
                font.family: theme.fontFamily
                font.pixelSize: theme.bodySize
            }

            Repeater {
                model: controller.challenges
                delegate: Components.OgsCard {
                    required property var modelData
                    Layout.fillWidth: true
                    Layout.preferredHeight: challengeColumn.implicitHeight + 20
                    tone: modelData.status === "accepted" ? "success"
                          : modelData.status === "pending" ? "warning" : "info"
                    highlighted: modelData.status === "accepted"

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
                            color: theme.textPrimary
                            font.family: theme.fontFamily
                            font.pixelSize: theme.bodySize
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
