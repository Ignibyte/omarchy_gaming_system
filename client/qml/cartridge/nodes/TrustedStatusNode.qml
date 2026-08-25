import QtQuick

Rectangle {
    property var nodeData: ({"text": "", "accessible_label": ""})
    property string assetRoot: ""
    property real scaleFactor: 1
    property bool highContrast: false
    property bool reducedMotion: false
    property bool mutedAudio: false

    width: parent ? parent.width : 640
    height: 40 * scaleFactor
    radius: 3
    color: highContrast ? "#000000" : "#122435"
    border.color: highContrast ? "#ffffff" : "#5ee6a8"
    Accessible.role: Accessible.StaticText
    Accessible.name: nodeData.accessible_label + ": " + nodeData.text

    Text {
        anchors.fill: parent
        anchors.margins: 9
        text: nodeData.text
        textFormat: Text.PlainText
        color: parent.highContrast ? "#ffffff" : "#5ee6a8"
        font.family: "monospace"
        font.bold: true
        font.pixelSize: 14 * parent.scaleFactor
        elide: Text.ElideRight
    }
}
