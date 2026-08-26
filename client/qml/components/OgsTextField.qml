import QtQuick
import QtQuick.Controls

TextField {
    id: root

    required property string accessibleName
    property string accessibleDescription: ""

    Accessible.name: accessibleName
    Accessible.description: accessibleDescription
    activeFocusOnTab: true
    focusPolicy: Qt.StrongFocus
    selectByMouse: true
    implicitHeight: theme.controlHeight
    color: theme.textPrimary
    placeholderTextColor: theme.textMuted
    selectionColor: theme.surfacePressed
    selectedTextColor: theme.focus
    font.family: theme.fontFamily
    font.pixelSize: theme.bodySize
    leftPadding: 12
    rightPadding: 12
    topPadding: 10
    bottomPadding: 10

    OgsTheme { id: theme }

    background: Rectangle {
        radius: theme.radius
        color: theme.surface
        border.color: root.activeFocus ? theme.focus : theme.border
        border.width: root.activeFocus ? theme.focusWidth : theme.borderWidth
    }
}
