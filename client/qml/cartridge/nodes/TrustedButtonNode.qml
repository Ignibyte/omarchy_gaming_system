import QtQuick
import QtQuick.Controls
import "../../components" as Components

Button {
    id: root

    property var nodeData: ({"label": "", "action": "", "accessible_label": ""})
    property string assetRoot: ""
    property real scaleFactor: 1
    property bool highContrast: false
    property bool reducedMotion: false
    property bool mutedAudio: false
    property bool actionsEnabled: true
    property bool accessibilityReady: false
    signal actionRequested(string action, var payload)

    Components.OgsTheme { id: theme }

    width: parent ? parent.width : 640
    height: 46 * scaleFactor
    text: nodeData.label
    activeFocusOnTab: true
    focusPolicy: Qt.StrongFocus
    hoverEnabled: true
    enabled: actionsEnabled
    opacity: actionsEnabled ? 1 : 0.6
    Accessible.name: nodeData.accessible_label
    Accessible.ignored: !accessibilityReady
    Keys.onReturnPressed: function(event) { root.triggerFromKey(event) }
    Keys.onEnterPressed: function(event) { root.triggerFromKey(event) }
    Keys.onSpacePressed: function(event) { root.triggerFromKey(event) }
    onClicked: root.trigger()

    function trigger() {
        if (actionsEnabled)
            actionRequested(nodeData.action, {})
    }

    function triggerFromKey(event) {
        event.accepted = true
        if (event.isAutoRepeat || !actionsEnabled)
            return false
        trigger()
        return true
    }

    contentItem: Text {
        text: root.nodeData.label
        textFormat: Text.PlainText
        color: root.highContrast
               ? (root.activeFocus ? theme.highContrastBackground
                                   : theme.highContrastForeground)
               : theme.textPrimary
        font.family: theme.fontFamily
        font.bold: true
        font.pixelSize: theme.controlSize * root.scaleFactor
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
    }

    background: Rectangle {
        radius: theme.radius
        color: root.activeFocus || root.hovered
               ? (root.highContrast ? theme.highContrastForeground
                                    : theme.surfaceHover)
               : (root.highContrast ? theme.highContrastBackground
                                    : theme.surfaceRaised)
        border.color: root.highContrast ? theme.highContrastForeground
                                        : theme.warning
        border.width: root.activeFocus ? theme.focusWidth : theme.borderWidth
    }
}
