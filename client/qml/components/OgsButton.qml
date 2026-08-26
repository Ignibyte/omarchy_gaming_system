import QtQuick
import QtQuick.Controls

Button {
    id: root

    property string accessibleName: text
    property string accessibleDescription: ""

    Accessible.name: accessibleName
    Accessible.description: accessibleDescription
    activeFocusOnTab: true
    focusPolicy: Qt.StrongFocus
    hoverEnabled: true
    opacity: enabled ? 1 : 0.68
    implicitHeight: theme.controlHeight
    font.family: theme.fontFamily
    font.bold: true
    font.pixelSize: theme.controlSize
    leftPadding: 18
    rightPadding: 18
    topPadding: theme.spaceSm
    bottomPadding: theme.spaceSm

    OgsTheme { id: theme }

    Keys.onReturnPressed: function(event) {
        if (root.enabled)
            root.clicked()
        event.accepted = true
    }
    Keys.onEnterPressed: function(event) {
        if (root.enabled)
            root.clicked()
        event.accepted = true
    }

    contentItem: Text {
        text: (root.checkable && root.checked ? "● " : "") + root.text
        textFormat: Text.PlainText
        color: root.enabled ? theme.textPrimary : theme.textMuted
        font: root.font
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
    }

    background: Rectangle {
        radius: theme.radius
        color: root.down ? theme.surfacePressed
                         : root.hovered ? theme.surfaceHover : theme.surfaceRaised
        border.color: root.activeFocus ? theme.focus
                                       : root.checked ? theme.accent
                                       : root.enabled ? theme.border : theme.borderMuted
        border.width: root.activeFocus ? theme.focusWidth : theme.borderWidth
    }
}
