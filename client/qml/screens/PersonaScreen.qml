import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components" as Components

Item {
    id: root

    required property var controller
    property bool createMode: controller.personas.length === 0

    Components.OgsTheme { id: theme }

    Timer {
        interval: 50
        running: root.visible && !root.createMode && personaList.count > 0
        onTriggered: root.focusInitial()
    }

    function focusInitial() {
        if (createMode || controller.personas.length === 0) {
            handleField.forceActiveFocus()
        } else if (personaList.count > 0) {
            const first = personaList.itemAtIndex(0)
            if (first)
                first.forceActiveFocus()
        }
    }

    function submitCreation() {
        controller.createPersona(handleField.text, displayNameField.text,
                                 bioField.text, statusField.text)
    }

    Keys.onEscapePressed: function(event) {
        if (createMode && controller.personas.length > 0) {
            createMode = false
            controller.clearError()
            Qt.callLater(root.focusInitial)
            event.accepted = true
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
                screenKey: "personas"
                title: createMode ? "CREATE PERSONA" : "CHOOSE PERSONA"
                statusText: controller.statusText
                statusTone: controller.busy ? "working" : "success"
                errorText: controller.errorText
                navigationHint: createMode && controller.personas.length > 0
                                ? "ESC PERSONA LIST" : "ENTER SELECT // TAB MOVE"
            }

            ListView {
                id: personaList
                objectName: "personaList"
                Layout.fillWidth: true
                Layout.maximumWidth: 620
                Layout.alignment: Qt.AlignHCenter
                Layout.preferredHeight: Math.min(contentHeight, 190)
                visible: !root.createMode && controller.personas.length > 0
                spacing: theme.spaceSm
                clip: true
                model: controller.personas
                currentIndex: count > 0 ? 0 : -1
                onCurrentItemChanged: {
                    if (currentItem && !root.createMode)
                        Qt.callLater(root.focusInitial)
                }

                delegate: Components.OgsButton {
                    required property int index
                    required property var modelData
                    width: ListView.view.width
                    objectName: "personaOption" + index
                    text: modelData.display_name + "  //  @" + modelData.handle
                    accessibleName: "Select persona " + modelData.display_name
                    onClicked: controller.selectPersona(modelData)
                    Component.onCompleted: {
                        if (index === 0 && !root.createMode)
                            Qt.callLater(root.focusInitial)
                    }
                }
            }

            Components.OgsButton {
                id: createModeButton
                objectName: "createPersonaModeButton"
                Layout.alignment: Qt.AlignHCenter
                visible: !root.createMode
                text: "CREATE ANOTHER PERSONA"
                accessibleName: "Open persona creation form"
                enabled: !controller.busy
                onClicked: {
                    root.createMode = true
                    controller.clearError()
                    Qt.callLater(root.focusInitial)
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                Layout.maximumWidth: 620
                Layout.alignment: Qt.AlignHCenter
                visible: root.createMode
                spacing: 8

                Components.OgsSectionLabel {
                    text: "HANDLE"
                    tone: "info"
                }

                Components.OgsTextField {
                    id: handleField
                    objectName: "personaHandleField"
                    Layout.fillWidth: true
                    accessibleName: "Persona handle"
                    placeholderText: "player_one"
                    maximumLength: 24
                    enabled: !controller.busy
                    onAccepted: displayNameField.forceActiveFocus()
                }

                Components.OgsSectionLabel {
                    text: "DISPLAY NAME"
                    tone: "info"
                }

                Components.OgsTextField {
                    id: displayNameField
                    objectName: "personaDisplayNameField"
                    Layout.fillWidth: true
                    accessibleName: "Persona display name"
                    placeholderText: "Player One"
                    maximumLength: 64
                    enabled: !controller.busy
                    onAccepted: statusField.forceActiveFocus()
                }

                Components.OgsSectionLabel {
                    text: "STATUS"
                    tone: "info"
                }

                Components.OgsTextField {
                    id: statusField
                    objectName: "personaStatusField"
                    Layout.fillWidth: true
                    accessibleName: "Persona status message"
                    placeholderText: "Ready to play"
                    maximumLength: 160
                    enabled: !controller.busy
                }

                Components.OgsSectionLabel {
                    text: "BIO"
                    tone: "info"
                }

                Components.OgsTextArea {
                    id: bioField
                    objectName: "personaBioField"
                    Layout.fillWidth: true
                    Layout.preferredHeight: 90
                    accessibleName: "Persona bio"
                    placeholderText: "A short public introduction"
                    maximumLength: 1000
                    enabled: !controller.busy
                }

                RowLayout {
                    Layout.alignment: Qt.AlignHCenter
                    spacing: 12

                    Components.OgsButton {
                        id: createButton
                        objectName: "createPersonaButton"
                        text: controller.busy ? "CREATING..." : "CREATE PERSONA"
                        accessibleName: "Submit persona creation"
                        enabled: !controller.busy
                        onClicked: root.submitCreation()
                    }

                    Components.OgsButton {
                        id: cancelCreateButton
                        objectName: "cancelPersonaCreationButton"
                        visible: controller.personas.length > 0
                        text: "CANCEL"
                        accessibleName: "Cancel persona creation"
                        enabled: !controller.busy
                        onClicked: {
                            root.createMode = false
                            controller.clearError()
                            Qt.callLater(root.focusInitial)
                        }
                    }
                }
            }

            Components.OgsButton {
                id: logoutButton
                objectName: "personaLogoutButton"
                Layout.alignment: Qt.AlignHCenter
                text: "SIGN OUT"
                accessibleName: "Sign out before selecting a persona"
                enabled: !controller.busy
                onClicked: controller.logout()
            }
        }
    }
}
