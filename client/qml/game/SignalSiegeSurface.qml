import QtQuick
import QtQuick.Layouts
import "../cartridge/nodes" as Nodes
import "../components" as Components

Item {
    id: root

    required property var presentation
    signal actionRequested(string action)

    Components.OgsTheme { id: theme }

    implicitHeight: battlefield.implicitHeight

    function focusInitial() {
        if (strikeButton.enabled)
            strikeButton.forceActiveFocus()
        else if (chargeButton.enabled)
            chargeButton.forceActiveFocus()
    }

    ColumnLayout {
        id: battlefield
        width: parent.width
        spacing: theme.spaceMd

        Components.OgsSectionLabel {
            Layout.fillWidth: true
            text: root.presentation.title
            tone: "success"
            font.pixelSize: theme.titleSize
            horizontalAlignment: Text.AlignHCenter
        }

        Text {
            Layout.fillWidth: true
            text: root.presentation.turn_label
            textFormat: Text.PlainText
            color: theme.textMuted
            font.family: theme.fontFamily
            font.pixelSize: theme.bodySize
            horizontalAlignment: Text.AlignHCenter
        }

        Nodes.TrustedStatusNode {
            Layout.fillWidth: true
            nodeData: {
                "text": root.presentation.status,
                "accessible_label": "Authoritative game status"
            }
        }

        Text {
            Layout.fillWidth: true
            text: root.presentation.actor_label + " // CORE " + root.presentation.actor_core
                  + " // ENERGY " + root.presentation.actor_energy
                  + (root.presentation.actor_guard > 0 ? " // GUARD " + root.presentation.actor_guard : "")
            textFormat: Text.PlainText
            color: theme.textPrimary
            font.family: theme.fontFamily
            font.pixelSize: theme.bodySize
            font.bold: true
            wrapMode: Text.Wrap
        }

        Nodes.TrustedMeterNode {
            Layout.fillWidth: true
            nodeData: {
                "value": root.presentation.actor_core,
                "minimum": 0,
                "maximum": 8,
                "accessible_label": root.presentation.actor_label + " core"
            }
        }

        Nodes.TrustedMeterNode {
            Layout.fillWidth: true
            nodeData: {
                "value": root.presentation.actor_energy,
                "minimum": 0,
                "maximum": 4,
                "accessible_label": root.presentation.actor_label + " energy"
            }
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 1
            color: theme.borderMuted
        }

        Text {
            Layout.fillWidth: true
            text: root.presentation.opponent_label + " // CORE " + root.presentation.opponent_core
                  + " // ENERGY " + root.presentation.opponent_energy
                  + (root.presentation.opponent_guard > 0 ? " // GUARD " + root.presentation.opponent_guard : "")
            textFormat: Text.PlainText
            color: theme.warning
            font.family: theme.fontFamily
            font.pixelSize: theme.bodySize
            font.bold: true
            wrapMode: Text.Wrap
        }

        Nodes.TrustedMeterNode {
            Layout.fillWidth: true
            nodeData: {
                "value": root.presentation.opponent_core,
                "minimum": 0,
                "maximum": 8,
                "accessible_label": root.presentation.opponent_label + " core"
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 8

            Nodes.TrustedButtonNode {
                id: strikeButton
                objectName: "gameStrikeButton"
                Layout.fillWidth: true
                enabled: root.presentation.can_strike
                opacity: enabled ? 1 : 0.4
                nodeData: {
                    "label": "STRIKE [2 DMG]",
                    "action": "strike",
                    "accessible_label": "Strike for two damage; costs one energy"
                }
                onActionRequested: function(action) { root.actionRequested(action) }
            }

            Nodes.TrustedButtonNode {
                id: guardButton
                objectName: "gameGuardButton"
                Layout.fillWidth: true
                enabled: root.presentation.can_guard
                opacity: enabled ? 1 : 0.4
                nodeData: {
                    "label": "GUARD [2]",
                    "action": "guard",
                    "accessible_label": "Guard two damage; costs one energy"
                }
                onActionRequested: function(action) { root.actionRequested(action) }
            }

            Nodes.TrustedButtonNode {
                id: chargeButton
                objectName: "gameChargeButton"
                Layout.fillWidth: true
                enabled: root.presentation.can_charge
                opacity: enabled ? 1 : 0.4
                nodeData: {
                    "label": "CHARGE [+2]",
                    "action": "charge",
                    "accessible_label": "Charge two energy"
                }
                onActionRequested: function(action) { root.actionRequested(action) }
            }
        }
    }
}
