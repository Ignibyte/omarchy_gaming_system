import QtQuick

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

    width: parent ? parent.width : 640
    height: 38 * scaleFactor
    color: highContrast ? "#000000" : "#132537"
    border.color: highContrast ? "#ffffff" : "#365572"
    Accessible.role: Accessible.ProgressBar
    Accessible.name: nodeData.accessible_label + ": " + String(nodeData.value)

    Rectangle {
        anchors.left: parent.left
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.margins: 4
        width: (parent.width - 8) * root.ratio
        color: root.highContrast ? "#ffffff" : "#5ee6a8"
    }

    Text {
        anchors.centerIn: parent
        text: root.nodeData.accessible_label + " " + root.nodeData.value
        textFormat: Text.PlainText
        color: root.highContrast && root.ratio > 0.45 ? "#000000" : "#eef7ff"
        font.family: "monospace"
        font.bold: true
        font.pixelSize: 13 * root.scaleFactor
    }
}
