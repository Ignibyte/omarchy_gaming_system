import QtQuick
import "../../components" as Components

Rectangle {
    id: root

    property var nodeData: ({"message": "", "accessible_label": ""})
    property string assetRoot: ""
    property real scaleFactor: 1
    property bool highContrast: false
    property bool reducedMotion: false
    property bool mutedAudio: false

    Components.OgsTheme { id: theme }

    width: parent ? parent.width : 640
    height: 54 * scaleFactor
    color: highContrast ? theme.highContrastBackground : theme.surfaceRaised
    border.color: highContrast ? theme.highContrastForeground : theme.warning
    Accessible.role: Accessible.StaticText
    Accessible.name: nodeData.accessible_label

    Text {
        anchors.fill: parent
        anchors.margins: 10
        text: root.nodeData.message
        textFormat: Text.PlainText
        color: root.highContrast ? theme.highContrastForeground : theme.warning
        font.family: theme.fontFamily
        font.pixelSize: theme.bodySize * root.scaleFactor
        wrapMode: Text.Wrap
    }
}
