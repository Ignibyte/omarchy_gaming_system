import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components" as Components
import "../game" as Game

Item {
    id: root

    required property var controller
    required property var sessionController

    function focusInitial() {
        if (surface.visible)
            surface.focusInitial()
        else
            refreshButton.forceActiveFocus()
    }

    Keys.onEscapePressed: function(event) {
        controller.closeSession()
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
                    text: "AUTHORITATIVE GAME LINK"
                    textFormat: Text.PlainText
                    color: "#8aa4c0"
                    font.family: "monospace"
                    font.bold: true
                }

                Components.OgsButton {
                    id: refreshButton
                    objectName: "gameRefreshButton"
                    text: "REFRESH"
                    accessibleName: "Refresh authoritative game state"
                    enabled: !controller.busy && controller.selectedSession !== null
                    onClicked: controller.openSessionById(controller.selectedSession.id)
                }

                Components.OgsButton {
                    objectName: "gameBackButton"
                    text: "GAMES"
                    accessibleName: "Return to game cartridges"
                    enabled: !controller.busy
                    onClicked: {
                        controller.closeSession()
                        sessionController.showPlayerScreen("games")
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

            Components.OgsButton {
                visible: controller.hasRetryableMutation
                text: "RETRY SAME COMMAND"
                accessibleName: "Retry the same game command identity"
                enabled: !controller.busy
                onClicked: controller.retryPendingMutation()
            }

            Text {
                Layout.fillWidth: true
                visible: controller.selectedSession !== null && !controller.presentation.supported
                text: "This cartridge is listed safely, but this client has no trusted presenter for it."
                textFormat: Text.PlainText
                color: "#f4c95d"
                font.family: "monospace"
                wrapMode: Text.Wrap
            }

            Game.SignalSiegeSurface {
                id: surface
                objectName: "signalSiegeSurface"
                Layout.fillWidth: true
                visible: controller.selectedSession !== null && controller.presentation.supported
                presentation: controller.presentation
                enabled: !controller.busy
                opacity: enabled ? 1 : 0.65
                onActionRequested: function(action) { controller.submitAction(action) }
            }
        }
    }
}
