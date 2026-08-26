import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: root

    required property var persona
    property string detail: ""
    property string primaryText: ""
    property string secondaryText: ""
    property bool actionsEnabled: true

    signal primaryTriggered()
    signal secondaryTriggered()

    implicitHeight: row.implicitHeight + 20
    radius: 3
    color: "#0c1825"
    border.color: "#29445e"

    RowLayout {
        id: row
        anchors.fill: parent
        anchors.margins: 10
        spacing: 10

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 3

            Text {
                Layout.fillWidth: true
                text: root.persona.display_name + "  @" + root.persona.handle
                textFormat: Text.PlainText
                color: "#eef7ff"
                font.family: "monospace"
                font.bold: true
                font.pixelSize: 14
                elide: Text.ElideRight
            }

            Text {
                Layout.fillWidth: true
                visible: root.detail !== ""
                text: root.detail
                textFormat: Text.PlainText
                color: "#8aa4c0"
                font.family: "monospace"
                font.pixelSize: 11
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
