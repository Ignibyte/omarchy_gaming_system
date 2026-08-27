import QtQuick
import "../../components" as Components

Rectangle {
    id: root

    property var nodeData: ({"label": "", "action": "", "accessible_label": ""})
    property string assetRoot: ""
    property real scaleFactor: 1
    property bool highContrast: false
    property bool reducedMotion: false
    property bool mutedAudio: false
    property bool actionsEnabled: true
    signal actionRequested(string action, var payload)

    Components.OgsTheme { id: theme }

    width: parent ? parent.width : 640
    height: 46 * scaleFactor
    radius: theme.radius
    color: activeFocus || pointer.containsMouse
           ? (highContrast ? theme.highContrastForeground : theme.surfaceHover)
           : (highContrast ? theme.highContrastBackground : theme.surfaceRaised)
    border.color: highContrast ? theme.highContrastForeground : theme.warning
    border.width: activeFocus ? theme.focusWidth : theme.borderWidth
    activeFocusOnTab: true
    opacity: actionsEnabled ? 1 : 0.6
    Accessible.role: Accessible.Button
    Accessible.name: nodeData.accessible_label
    Accessible.focused: activeFocus
    Accessible.onPressAction: root.trigger()
    Keys.onReturnPressed: root.trigger()
    Keys.onEnterPressed: root.trigger()
    Keys.onSpacePressed: root.trigger()

    function trigger() {
        if (actionsEnabled)
            actionRequested(nodeData.action, {})
    }

    Text {
        anchors.centerIn: parent
        text: root.nodeData.label
        textFormat: Text.PlainText
        color: root.highContrast
               ? (root.activeFocus ? theme.highContrastBackground
                                   : theme.highContrastForeground)
               : theme.textPrimary
        font.family: theme.fontFamily
        font.bold: true
        font.pixelSize: theme.controlSize * root.scaleFactor
        elide: Text.ElideRight
    }

    MouseArea {
        id: pointer
        anchors.fill: parent
        hoverEnabled: true
        enabled: root.actionsEnabled
        onClicked: {
            root.forceActiveFocus()
            root.trigger()
        }
    }
}
