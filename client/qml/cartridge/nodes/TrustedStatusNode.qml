import QtQuick
import "../../components" as Components

Rectangle {
    id: root

    property var nodeData: ({"text": "", "accessible_label": ""})
    property string assetRoot: ""
    property real scaleFactor: 1
    property bool highContrast: false
    property bool reducedMotion: false
    property bool mutedAudio: false

    Components.OgsTheme { id: theme }

    width: parent ? parent.width : 640
    height: 40 * scaleFactor
    radius: theme.radius
    color: highContrast ? theme.highContrastBackground : theme.surface
    border.color: highContrast ? theme.highContrastForeground : theme.accent
    Accessible.role: Accessible.StaticText
    Accessible.name: nodeData.accessible_label + ": " + nodeData.text

    Text {
        anchors.fill: parent
        anchors.margins: 9
        text: root.nodeData.text
        textFormat: Text.PlainText
        color: root.highContrast ? theme.highContrastForeground : theme.accent
        font.family: theme.fontFamily
        font.bold: true
        font.pixelSize: theme.bodySize * root.scaleFactor
        elide: Text.ElideRight
    }
}
