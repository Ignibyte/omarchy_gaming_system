import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components" as Components

Item {
    id: root

    required property var controller

    function focusInitial() {
        factorField.forceActiveFocus()
    }

    function submit() {
        const factor = factorField.text
        factorField.clear()
        controller.completeMfa(factor)
    }

    Keys.onEscapePressed: function(event) {
        factorField.clear()
        controller.cancelMfa()
        event.accepted = true
    }

    ScrollView {
        id: scroll
        anchors.fill: parent
        anchors.margins: 28
        contentWidth: availableWidth

        ColumnLayout {
            width: scroll.availableWidth
            spacing: 16

            Item { Layout.fillHeight: true; Layout.minimumHeight: 16 }

            Text {
                Layout.fillWidth: true
                text: "SECOND FACTOR"
                textFormat: Text.PlainText
                color: "#f4c95d"
                font.family: "monospace"
                font.bold: true
                font.pixelSize: 30
                horizontalAlignment: Text.AlignHCenter
            }

            Text {
                Layout.fillWidth: true
                text: controller.statusText
                textFormat: Text.PlainText
                color: "#d5e2ef"
                font.family: "monospace"
                font.pixelSize: 15
                wrapMode: Text.Wrap
                horizontalAlignment: Text.AlignHCenter
            }

            Text {
                Layout.fillWidth: true
                visible: controller.errorText !== ""
                text: controller.errorText
                textFormat: Text.PlainText
                color: "#ff8b98"
                font.family: "monospace"
                font.pixelSize: 14
                wrapMode: Text.Wrap
                horizontalAlignment: Text.AlignHCenter
            }

            Text {
                Layout.fillWidth: true
                Layout.maximumWidth: 560
                Layout.alignment: Qt.AlignHCenter
                text: "AUTHENTICATOR OR RECOVERY CODE"
                textFormat: Text.PlainText
                color: "#8aa4c0"
                font.family: "monospace"
                font.pixelSize: 12
            }

            Components.OgsTextField {
                id: factorField
                objectName: "mfaFactorField"
                Layout.fillWidth: true
                Layout.maximumWidth: 560
                Layout.alignment: Qt.AlignHCenter
                accessibleName: "Authenticator or recovery code"
                echoMode: TextInput.Password
                passwordCharacter: "●"
                maximumLength: 64
                enabled: !controller.busy
                onAccepted: root.submit()
            }

            Text {
                Layout.fillWidth: true
                text: controller.mfaExpiresAt === "" ? "" : "Challenge expires " + controller.mfaExpiresAt
                textFormat: Text.PlainText
                color: "#6f879f"
                font.family: "monospace"
                font.pixelSize: 11
                horizontalAlignment: Text.AlignHCenter
            }

            RowLayout {
                Layout.alignment: Qt.AlignHCenter
                spacing: 12

                Components.OgsButton {
                    id: verifyButton
                    objectName: "verifyMfaButton"
                    text: controller.busy ? "VERIFYING..." : "VERIFY"
                    accessibleName: "Verify second factor"
                    enabled: !controller.busy
                    onClicked: root.submit()
                }

                Components.OgsButton {
                    id: cancelButton
                    objectName: "cancelMfaButton"
                    text: "CANCEL"
                    accessibleName: "Cancel second-factor sign in"
                    enabled: !controller.busy
                    onClicked: {
                        factorField.clear()
                        controller.cancelMfa()
                    }
                }
            }

            Item { Layout.fillHeight: true; Layout.minimumHeight: 16 }
        }
    }
}
