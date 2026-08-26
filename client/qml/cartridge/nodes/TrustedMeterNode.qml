import QtQuick
import "../../components" as Components

Rectangle {
    id: root

    property var nodeData: ({"value": 0, "minimum": 0, "maximum": 1, "accessible_label": ""})
    property string assetRoot: ""
    property real scaleFactor: 1
    property bool highContrast: false
    property bool reducedMotion: false
    property bool mutedAudio: false
    readonly property real ratio: (nodeData.value - nodeData.minimum)
        / (nodeData.maximum - nodeData.minimum)

    Components.OgsTheme { id: theme }

    width: parent ? parent.width : 640
    height: 38 * scaleFactor
    color: highContrast ? theme.highContrastBackground : theme.surfaceRaised
    border.color: highContrast ? theme.highContrastForeground : theme.border
    Accessible.role: Accessible.ProgressBar
    Accessible.name: nodeData.accessible_label + ": " + String(nodeData.value)

    Rectangle {
        anchors.left: parent.left
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.margins: 4
        width: (parent.width - 8) * root.ratio
        color: root.highContrast ? theme.highContrastForeground : theme.accent
    }

    Text {
        anchors.centerIn: parent
        text: root.nodeData.accessible_label + " " + root.nodeData.value
        textFormat: Text.PlainText
        color: root.highContrast && root.ratio > 0.45
               ? theme.highContrastBackground
               : root.highContrast ? theme.highContrastForeground : theme.textPrimary
        font.family: theme.fontFamily
        font.bold: true
        font.pixelSize: theme.bodySize * root.scaleFactor
    }
}
