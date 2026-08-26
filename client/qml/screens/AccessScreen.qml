import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components" as Components

Item {
    id: root

    required property var controller

    Components.OgsTheme { id: theme }

    function focusInitial() {
        if (controller.accessMode === "register")
            inviteCodeField.forceActiveFocus()
        else
            usernameField.forceActiveFocus()
    }

    function submit() {
        const username = usernameField.text
        const password = passwordField.text
        const inviteCode = inviteCodeField.text
        inviteCodeField.clear()
        passwordField.clear()
        if (controller.accessMode === "register")
            controller.registerAccount(inviteCode, username, password)
        else
            controller.signIn(username, password, deviceField.text)
    }

    Keys.onEscapePressed: function(event) {
        inviteCodeField.clear()
        passwordField.clear()
        if (controller.accessMode === "register")
            controller.chooseAccessMode("sign_in")
        else
            controller.showServerConfiguration()
        event.accepted = true
    }

    Connections {
        target: controller
        function onSuggestedUsernameChanged() {
            if (controller.suggestedUsername !== "")
                usernameField.text = controller.suggestedUsername
        }
        function onAccessModeChanged() {
            inviteCodeField.clear()
            passwordField.clear()
            Qt.callLater(root.focusInitial)
        }
    }

    ScrollView {
        id: scroll
        anchors.fill: parent
        anchors.margins: theme.spaceXl
        contentWidth: availableWidth

        ColumnLayout {
            width: scroll.availableWidth
            spacing: theme.spaceMd

            Components.OgsScreenHeader {
                Layout.fillWidth: true
                screenKey: "access"
                title: controller.accessMode === "register" ? "NEW PLAYER LINK" : "PLAYER ACCESS"
                statusText: controller.statusText
                statusTone: controller.busy ? "working" : "info"
                errorText: controller.errorText
                navigationHint: "TAB MOVE // ENTER SUBMIT // ESC SERVER"
            }

            RowLayout {
                Layout.alignment: Qt.AlignHCenter
                spacing: 10

                Components.OgsButton {
                    id: signInModeButton
                    objectName: "signInModeButton"
                    text: "SIGN IN"
                    accessibleName: "Show sign in form"
                    accessibleDescription: controller.accessMode === "sign_in"
                                           ? "Selected access mode" : "Switch access mode"
                    checkable: true
                    checked: controller.accessMode === "sign_in"
                    enabled: !controller.busy
                    onClicked: controller.chooseAccessMode("sign_in")
                }

                Components.OgsButton {
                    id: registerModeButton
                    objectName: "registerModeButton"
                    text: "CREATE ACCOUNT"
                    accessibleName: "Show account registration form"
                    accessibleDescription: controller.accessMode === "register"
                                           ? "Selected access mode" : "Switch access mode"
                    checkable: true
                    checked: controller.accessMode === "register"
                    enabled: !controller.busy
                    onClicked: controller.chooseAccessMode("register")
                }
            }

            Components.OgsSectionLabel {
                Layout.fillWidth: true
                Layout.maximumWidth: 560
                Layout.alignment: Qt.AlignHCenter
                visible: controller.accessMode === "register"
                text: "INVITATION CODE"
                tone: "info"
            }

            Components.OgsTextField {
                id: inviteCodeField
                objectName: "inviteCodeField"
                Layout.fillWidth: true
                Layout.maximumWidth: 560
                Layout.alignment: Qt.AlignHCenter
                visible: controller.accessMode === "register"
                accessibleName: "Registration invitation code"
                echoMode: TextInput.Password
                passwordCharacter: "●"
                placeholderText: "ogsi_..."
                maximumLength: 48
                enabled: !controller.busy
                onAccepted: usernameField.forceActiveFocus()
            }

            Components.OgsSectionLabel {
                Layout.fillWidth: true
                Layout.maximumWidth: 560
                Layout.alignment: Qt.AlignHCenter
                text: "USERNAME"
                tone: "info"
            }

            Components.OgsTextField {
                id: usernameField
                objectName: "usernameField"
                Layout.fillWidth: true
                Layout.maximumWidth: 560
                Layout.alignment: Qt.AlignHCenter
                accessibleName: "Account username"
                placeholderText: "player_one"
                maximumLength: 32
                enabled: !controller.busy
                onAccepted: passwordField.forceActiveFocus()
            }

            Components.OgsSectionLabel {
                Layout.fillWidth: true
                Layout.maximumWidth: 560
                Layout.alignment: Qt.AlignHCenter
                text: "PASSWORD"
                tone: "info"
            }

            Components.OgsTextField {
                id: passwordField
                objectName: "passwordField"
                Layout.fillWidth: true
                Layout.maximumWidth: 560
                Layout.alignment: Qt.AlignHCenter
                accessibleName: "Account password"
                echoMode: TextInput.Password
                passwordCharacter: "●"
                maximumLength: 128
                enabled: !controller.busy
                onAccepted: root.submit()
            }

            Components.OgsSectionLabel {
                Layout.fillWidth: true
                Layout.maximumWidth: 560
                Layout.alignment: Qt.AlignHCenter
                visible: controller.accessMode === "sign_in"
                text: "DEVICE LABEL"
                tone: "info"
            }

            Components.OgsTextField {
                id: deviceField
                objectName: "deviceNameField"
                Layout.fillWidth: true
                Layout.maximumWidth: 560
                Layout.alignment: Qt.AlignHCenter
                visible: controller.accessMode === "sign_in"
                accessibleName: "Device label"
                text: "Omarchy QML"
                maximumLength: 64
                enabled: !controller.busy
                onAccepted: root.submit()
            }

            Components.OgsButton {
                id: submitButton
                objectName: "accessSubmitButton"
                Layout.alignment: Qt.AlignHCenter
                Layout.minimumWidth: 240
                text: controller.busy ? "WORKING..."
                                      : controller.accessMode === "register"
                                        ? "CREATE ACCOUNT" : "SIGN IN"
                accessibleName: controller.accessMode === "register"
                                ? "Submit account registration" : "Submit sign in"
                enabled: !controller.busy
                onClicked: root.submit()
            }

            Components.OgsButton {
                id: changeServerButton
                objectName: "changeServerButton"
                Layout.alignment: Qt.AlignHCenter
                text: "CHANGE SERVER"
                accessibleName: "Change OmarchyGS server"
                enabled: !controller.busy
                onClicked: {
                    inviteCodeField.clear()
                    passwordField.clear()
                    controller.showServerConfiguration()
                }
            }

            Text {
                Layout.fillWidth: true
                text: controller.serverUrl
                textFormat: Text.PlainText
                color: theme.textMuted
                font.family: theme.fontFamily
                font.pixelSize: theme.captionSize
                elide: Text.ElideMiddle
                horizontalAlignment: Text.AlignHCenter
            }
        }
    }
}
