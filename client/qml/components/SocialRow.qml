import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

OgsCard {
    id: root

    required property var persona
    property string detail: ""
    property string primaryText: ""
    property string secondaryText: ""
    property bool actionsEnabled: true

    signal primaryTriggered()
    signal secondaryTriggered()

    implicitHeight: row.implicitHeight + 20
    tone: "success"

    OgsTheme { id: theme }

    RowLayout {
        id: row
        anchors.fill: parent
        anchors.margins: 10
        spacing: theme.spaceMd

        ColumnLayout {
            Layout.fillWidth: true
            spacing: theme.spaceXs

            Text {
                Layout.fillWidth: true
                text: root.persona.display_name + "  @" + root.persona.handle
                textFormat: Text.PlainText
                color: theme.textPrimary
                font.family: theme.fontFamily
                font.bold: true
                font.pixelSize: theme.bodySize
                elide: Text.ElideRight
            }

            Text {
                Layout.fillWidth: true
                visible: root.detail !== ""
                text: root.detail
                textFormat: Text.PlainText
                color: theme.textMuted
                font.family: theme.fontFamily
                font.pixelSize: theme.captionSize
                elide: Text.ElideRight
            }
        }

        OgsButton {
            objectName: "socialPrimaryButton"
            visible: root.primaryText !== ""
            enabled: root.actionsEnabled
            text: root.primaryText
            accessibleName: root.primaryText + " @" + root.persona.handle
            onClicked: root.primaryTriggered()
        }

        OgsButton {
            objectName: "socialSecondaryButton"
            visible: root.secondaryText !== ""
            enabled: root.actionsEnabled
            text: root.secondaryText
            accessibleName: root.secondaryText + " @" + root.persona.handle
            onClicked: root.secondaryTriggered()
        }
    }
}
