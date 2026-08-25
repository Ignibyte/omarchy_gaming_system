import QtQuick

Rectangle {
    property var nodeData: ({"message": "", "accessible_label": ""})
    property string assetRoot: ""
    property real scaleFactor: 1
    property bool highContrast: false
    property bool reducedMotion: false
    property bool mutedAudio: false

    width: parent ? parent.width : 640
    height: 54 * scaleFactor
    color: highContrast ? "#000000" : "#19151d"
    border.color: highContrast ? "#ffffff" : "#f4c95d"
    Accessible.role: Accessible.StaticText
    Accessible.name: nodeData.accessible_label

    Text {
        anchors.fill: parent
        anchors.margins: 10
        text: parent.nodeData.message
        textFormat: Text.PlainText
        color: parent.highContrast ? "#ffffff" : "#f4c95d"
        font.family: "monospace"
        font.pixelSize: 13 * parent.scaleFactor
        wrapMode: Text.Wrap
    }
}
