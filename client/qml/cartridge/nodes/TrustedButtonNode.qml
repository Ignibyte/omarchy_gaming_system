import QtQuick

Rectangle {
    id: root

    property var nodeData: ({"label": "", "action": "", "accessible_label": ""})
    property string assetRoot: ""
    property real scaleFactor: 1
    property bool highContrast: false
    property bool reducedMotion: false
    property bool mutedAudio: false
    signal actionRequested(string action, var payload)

    width: parent ? parent.width : 640
    height: 46 * scaleFactor
    radius: 3
    color: activeFocus || pointer.containsMouse ? (highContrast ? "#ffffff" : "#1d3348")
        : (highContrast ? "#000000" : "#132537")
    border.color: highContrast ? "#ffffff" : "#f4c95d"
    border.width: activeFocus ? 3 : 1
    activeFocusOnTab: true
    Accessible.role: Accessible.Button
    Accessible.name: nodeData.accessible_label
    Accessible.focused: activeFocus
    Accessible.onPressAction: root.trigger()
    Keys.onReturnPressed: root.trigger()
    Keys.onEnterPressed: root.trigger()
    Keys.onSpacePressed: root.trigger()

    function trigger() {
        actionRequested(nodeData.action, {})
    }

    Text {
        anchors.centerIn: parent
        text: root.nodeData.label
        textFormat: Text.PlainText
        color: root.highContrast && root.activeFocus ? "#000000" : "#eef7ff"
        font.family: "monospace"
        font.bold: true
        font.pixelSize: 14 * root.scaleFactor
        elide: Text.ElideRight
    }

    MouseArea {
        id: pointer
        anchors.fill: parent
        hoverEnabled: true
        onClicked: {
            root.forceActiveFocus()
            root.trigger()
        }
    }
}
