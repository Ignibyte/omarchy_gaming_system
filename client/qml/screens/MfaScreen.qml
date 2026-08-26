import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components" as Components

Item {
    id: root

    required property var controller

    Components.OgsTheme { id: theme }

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
        anchors.margins: theme.space2Xl
        contentWidth: availableWidth

        ColumnLayout {
            width: scroll.availableWidth
            spacing: theme.spaceLg

            Item { Layout.fillHeight: true; Layout.minimumHeight: 16 }

            Components.OgsScreenHeader {
                Layout.fillWidth: true
                screenKey: "mfa"
                title: "SECOND FACTOR"
                titleTone: "warning"
                statusText: controller.statusText
                statusTone: controller.busy ? "working" : "warning"
                errorText: controller.errorText
                navigationHint: "ENTER VERIFY // ESC CANCEL"
            }

            Components.OgsSectionLabel {
                Layout.fillWidth: true
                Layout.maximumWidth: 560
                Layout.alignment: Qt.AlignHCenter
                text: "AUTHENTICATOR OR RECOVERY CODE"
                tone: "info"
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
                color: theme.textMuted
                font.family: theme.fontFamily
                font.pixelSize: theme.captionSize
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
