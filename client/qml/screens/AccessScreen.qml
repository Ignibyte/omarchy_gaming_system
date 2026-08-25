import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components" as Components

Item {
    id: root

    required property var controller

    function focusInitial() {
        usernameField.forceActiveFocus()
    }

    function submit() {
        const username = usernameField.text
        const password = passwordField.text
        passwordField.clear()
        if (controller.accessMode === "register")
            controller.registerAccount(username, password)
        else
            controller.signIn(username, password, deviceField.text)
    }

    Keys.onEscapePressed: function(event) {
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
    }

    ScrollView {
        id: scroll
        anchors.fill: parent
        anchors.margins: 24
        contentWidth: availableWidth

        ColumnLayout {
            width: scroll.availableWidth
            spacing: 12

            RowLayout {
                Layout.alignment: Qt.AlignHCenter
                spacing: 10

                Components.OgsButton {
                    id: signInModeButton
                    objectName: "signInModeButton"
                    text: "SIGN IN"
                    accessibleName: "Show sign in form"
                    enabled: !controller.busy
                    opacity: controller.accessMode === "sign_in" ? 1 : 0.65
                    onClicked: controller.chooseAccessMode("sign_in")
                }

                Components.OgsButton {
                    id: registerModeButton
                    objectName: "registerModeButton"
                    text: "CREATE ACCOUNT"
                    accessibleName: "Show account registration form"
                    enabled: !controller.busy
                    opacity: controller.accessMode === "register" ? 1 : 0.65
                    onClicked: controller.chooseAccessMode("register")
                }
            }

            Text {
                Layout.fillWidth: true
                text: controller.accessMode === "register" ? "NEW PLAYER LINK" : "PLAYER ACCESS"
                textFormat: Text.PlainText
                color: "#5ee6a8"
                font.family: "monospace"
                font.bold: true
                font.pixelSize: 26
                horizontalAlignment: Text.AlignHCenter
            }

            Text {
                Layout.fillWidth: true
                text: controller.statusText
                textFormat: Text.PlainText
                color: "#d5e2ef"
                font.family: "monospace"
                font.pixelSize: 14
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
                text: "USERNAME"
                textFormat: Text.PlainText
                color: "#8aa4c0"
                font.family: "monospace"
                font.pixelSize: 12
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

            Text {
                Layout.fillWidth: true
                Layout.maximumWidth: 560
                Layout.alignment: Qt.AlignHCenter
                text: "PASSWORD"
                textFormat: Text.PlainText
                color: "#8aa4c0"
                font.family: "monospace"
                font.pixelSize: 12
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

            Text {
                Layout.fillWidth: true
                Layout.maximumWidth: 560
                Layout.alignment: Qt.AlignHCenter
                visible: controller.accessMode === "sign_in"
                text: "DEVICE LABEL"
                textFormat: Text.PlainText
                color: "#8aa4c0"
                font.family: "monospace"
                font.pixelSize: 12
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
                    passwordField.clear()
                    controller.showServerConfiguration()
                }
            }

            Text {
                Layout.fillWidth: true
                text: controller.serverUrl
                textFormat: Text.PlainText
                color: "#607890"
                font.family: "monospace"
                font.pixelSize: 11
                elide: Text.ElideMiddle
                horizontalAlignment: Text.AlignHCenter
            }
        }
    }
}
