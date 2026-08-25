import QtQuick

Rectangle {
    property var nodeData: ({"text": "", "accessible_label": ""})
    property string assetRoot: ""
    property real scaleFactor: 1
    property bool highContrast: false
    property bool reducedMotion: false
    property bool mutedAudio: false

    width: parent ? parent.width : 640
    height: Math.max(72 * scaleFactor, content.implicitHeight + 20)
    color: highContrast ? "#000000" : "#0b1420"
    border.color: highContrast ? "#ffffff" : "#263950"
    Accessible.role: Accessible.StaticText
    Accessible.name: nodeData.accessible_label

    Text {
        id: content
        anchors.fill: parent
        anchors.margins: 10
        text: nodeData.text
        textFormat: Text.PlainText
        color: parent.highContrast ? "#ffffff" : "#d5e2ef"
        font.family: "monospace"
        font.pixelSize: 14 * parent.scaleFactor
        wrapMode: Text.WrapAnywhere
    }
}
