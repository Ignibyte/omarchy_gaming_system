import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components" as Components
import "../game" as Game
import "../cartridge" as Cartridge

Item {
    id: root

    required property var controller
    required property var sessionController

    Components.OgsTheme { id: theme }

    function focusInitial() {
        if (cartridgeSurface.visible)
            cartridgeSurface.focusInitial()
        else if (signalSurface.visible)
            signalSurface.focusInitial()
        else
            refreshButton.forceActiveFocus()
    }

    Keys.onEscapePressed: function(event) {
        controller.closeSession()
        sessionController.showPlayerScreen("games")
        event.accepted = true
    }

    Connections {
        target: controller
        function onBusyChanged() {
            if (!controller.busy)
                Qt.callLater(root.focusInitial)
        }
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
                screenKey: "gameplay"
                title: "AUTHORITATIVE GAME LINK"
                statusText: controller.statusText
                statusTone: controller.busy || controller.loadState === "loading"
                            ? "working" : "success"
                errorText: controller.errorText
                navigationHint: "ESC GAMES // ENTER ACTION"
            }

            RowLayout {
                Layout.alignment: Qt.AlignRight
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

            Components.OgsButton {
                visible: controller.hasRetryableMutation
                text: "RETRY SAME COMMAND"
                accessibleName: "Retry the same game command identity"
                enabled: !controller.busy
                onClicked: controller.retryPendingMutation()
            }

            Components.OgsStatusBanner {
                Layout.fillWidth: true
                visible: controller.selectedSession !== null && !controller.presentation.supported
                         && controller.cartridgeRenderPlan === null
                message: "This cartridge is listed safely, but this client has no trusted presenter for it."
                tone: "warning"
            }

            Cartridge.TrustedCartridgeSurface {
                id: cartridgeSurface
                objectName: "trustedCartridgeGameplaySurface"
                Layout.fillWidth: true
                Layout.preferredHeight: 360
                visible: controller.selectedSession !== null
                         && controller.cartridgeRenderPlan !== null
                assetRoot: controller.cartridgeAssetRoot
                renderPlan: controller.cartridgeRenderPlan
                actionsEnabled: !controller.busy
                                && controller.selectedSession !== null
                                && controller.selectedSession.status === "active"
                                && controller.selectedSession.presentation !== null
                                && controller.selectedSession.presentation.active_session_policy === "continue"
                opacity: actionsEnabled ? 1 : 0.65
                onActionRequested: function(action, payload) {
                    controller.submitCartridgeAction(action, payload)
                }
            }

            Game.SignalSiegeSurface {
                id: signalSurface
                objectName: "signalSiegeSurface"
                Layout.fillWidth: true
                visible: controller.selectedSession !== null && controller.presentation.supported
                         && controller.cartridgeRenderPlan === null
                presentation: controller.presentation
                enabled: !controller.busy
                opacity: enabled ? 1 : 0.65
                onActionRequested: function(action) { controller.submitAction(action) }
            }
        }
    }
}
