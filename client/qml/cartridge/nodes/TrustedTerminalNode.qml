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
    height: Math.max(72 * scaleFactor, content.implicitHeight + 20)
    color: highContrast ? theme.highContrastBackground : theme.background
    border.color: highContrast ? theme.highContrastForeground : theme.borderMuted
    Accessible.role: Accessible.StaticText
    Accessible.name: nodeData.accessible_label

    Text {
        id: content
        anchors.fill: parent
        anchors.margins: 10
        text: root.nodeData.text
        textFormat: Text.PlainText
        color: root.highContrast ? theme.highContrastForeground : theme.textSecondary
        font.family: theme.fontFamily
        font.pixelSize: theme.bodySize * root.scaleFactor
        wrapMode: Text.WrapAnywhere
    }
}
