import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components" as Components

Item {
    id: root

    required property var controller

    Components.OgsTheme { id: theme }

    function focusInitial() {
        endpointField.forceActiveFocus()
    }

    Keys.onEscapePressed: function(event) {
        endpointField.text = controller.serverUrl
        event.accepted = true
    }

    ScrollView {
        id: scroll
        anchors.fill: parent
        anchors.margins: theme.space2Xl
        contentWidth: availableWidth

        ColumnLayout {
            width: scroll.availableWidth
            spacing: theme.spaceLg

            Item { Layout.fillHeight: true; Layout.minimumHeight: 12 }

            Components.OgsScreenHeader {
                Layout.fillWidth: true
                screenKey: "connection"
                title: "LINK A SERVER"
                statusText: controller.statusText
                statusTone: controller.busy ? "working"
                            : controller.connectionState === "ready" ? "success"
                            : controller.connectionState === "identity_mismatch" ? "danger"
                            : controller.connectionState === "incompatible" ? "warning" : "info"
                errorText: controller.errorText
                navigationHint: "ENTER SAVE + CONNECT // TAB CHOOSE // ESC RESTORE URL"
            }

            Components.OgsSectionLabel {
                Layout.fillWidth: true
                Layout.maximumWidth: 620
                Layout.alignment: Qt.AlignHCenter
                text: "SAVED SERVERS"
                tone: "success"
            }

            Text {
                visible: controller.serverProfiles.length === 0
                Layout.fillWidth: true
                Layout.maximumWidth: 620
                Layout.alignment: Qt.AlignHCenter
                text: "No saved servers. Connect once or save a compatible community below."
                textFormat: Text.PlainText
                color: theme.textMuted
                font.family: theme.fontFamily
                font.pixelSize: theme.captionSize
                wrapMode: Text.Wrap
                horizontalAlignment: Text.AlignHCenter
                Accessible.role: Accessible.StaticText
                Accessible.name: text
            }

            Repeater {
                model: controller.serverProfiles

                delegate: Components.OgsCard {
                    required property int index
                    required property var modelData

                    Layout.fillWidth: true
                    Layout.maximumWidth: 620
                    Layout.alignment: Qt.AlignHCenter
                    implicitHeight: savedRow.implicitHeight + theme.spaceMd * 2
                    highlighted: controller.selectedProfileId === modelData.server_id
                    tone: "success"

                    RowLayout {
                        id: savedRow
                        anchors.fill: parent
                        anchors.margins: theme.spaceMd
                        spacing: theme.spaceSm

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 2

                            Text {
                                Layout.fillWidth: true
                                text: modelData.server_name
                                textFormat: Text.PlainText
                                color: theme.textPrimary
                                font.family: theme.fontFamily
                                font.bold: true
                                font.pixelSize: theme.controlSize
                                elide: Text.ElideRight
                            }

                            Text {
                                Layout.fillWidth: true
                                text: modelData.origin
                                textFormat: Text.PlainText
                                color: theme.textMuted
                                font.family: theme.fontFamily
                                font.pixelSize: theme.captionSize
                                elide: Text.ElideMiddle
                            }
                        }

                        Components.OgsButton {
                            objectName: "savedProfileConnect_" + index
                            Layout.preferredWidth: 112
                            text: controller.busy ? "WAIT" : "CONNECT"
                            accessibleName: "Connect to saved server " + modelData.server_name
                            accessibleDescription: "Require pinned server identity "
                                                   + modelData.server_id
                            enabled: !controller.busy
                            onClicked: controller.connectSavedProfile(index)
                        }

                        Components.OgsButton {
                            objectName: "savedProfileRemove_" + index
                            Layout.preferredWidth: 92
                            text: "REMOVE"
                            accessibleName: "Remove saved server " + modelData.server_name
                            enabled: !controller.busy
                            onClicked: controller.removeServerProfile(index)
                        }
                    }
                }
            }

            Components.OgsSectionLabel {
                Layout.fillWidth: true
                Layout.maximumWidth: 620
                Layout.alignment: Qt.AlignHCenter
                text: "SERVER URL"
                tone: "info"
            }

            Components.OgsTextField {
                id: endpointField
                objectName: "serverUrlField"
                Layout.fillWidth: true
                Layout.maximumWidth: 620
                Layout.alignment: Qt.AlignHCenter
                accessibleName: "OmarchyGS server URL"
                text: controller.serverUrl
                placeholderText: "https://games.example.net"
                enabled: !controller.busy
                onAccepted: connectButton.clicked()
            }

            RowLayout {
                Layout.alignment: Qt.AlignHCenter
                spacing: theme.spaceMd

                Components.OgsButton {
                    id: connectOnceButton
                    objectName: "connectOnceButton"
                    Layout.minimumWidth: 180
                    text: controller.busy ? "CHECKING..." : "CONNECT ONCE"
                    accessibleName: "Connect once without saving this server"
                    enabled: !controller.busy
                    onClicked: controller.connectToServer(endpointField.text, false)
                }

                Components.OgsButton {
                    id: connectButton
                    objectName: "connectButton"
                    Layout.minimumWidth: 200
                    text: controller.busy ? "CHECKING..." : "SAVE + CONNECT"
                    accessibleName: "Save and connect to this OmarchyGS server"
                    enabled: !controller.busy
                    onClicked: controller.connectToServer(endpointField.text, true)
                }
            }

            Text {
                Layout.fillWidth: true
                Layout.maximumWidth: 620
                Layout.alignment: Qt.AlignHCenter
                text: "Remote servers require HTTPS. Plain HTTP is limited to localhost."
                textFormat: Text.PlainText
                color: theme.textMuted
                font.family: theme.fontFamily
                font.pixelSize: theme.captionSize
                wrapMode: Text.Wrap
                horizontalAlignment: Text.AlignHCenter
            }

            Item { Layout.fillHeight: true; Layout.minimumHeight: 12 }
        }
    }
}
