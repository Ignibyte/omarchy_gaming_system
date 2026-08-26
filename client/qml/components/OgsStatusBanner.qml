import QtQuick
import QtQuick.Layouts

Rectangle {
    id: root

    property string message: ""
    property string tone: "info"
    property string prefix: theme.tonePrefix(tone)
    property string accessibleDescription: ""

    visible: message !== ""
    implicitHeight: bannerText.implicitHeight + theme.spaceLg
    radius: theme.radius
    color: theme.surface
    border.color: theme.toneColor(tone)
    border.width: theme.borderWidth

    Accessible.role: tone === "error" ? Accessible.AlertMessage : Accessible.StatusBar
    Accessible.name: prefix + ": " + message
    Accessible.description: accessibleDescription

    OgsTheme { id: theme }

    RowLayout {
        anchors.fill: parent
        anchors.margins: theme.spaceSm
        spacing: theme.spaceSm

        Rectangle {
            Layout.fillHeight: true
            Layout.preferredWidth: 5
            radius: 2
            color: theme.toneColor(root.tone)
        }

        Text {
            id: bannerText
            Layout.fillWidth: true
            text: root.prefix + " // " + root.message
            textFormat: Text.PlainText
            color: theme.textPrimary
            font.family: theme.fontFamily
            font.pixelSize: theme.bodySize
            wrapMode: Text.Wrap
        }
    }
}
