import QtQuick
import QtQuick.Controls

TextArea {
    id: root

    required property string accessibleName
    property string accessibleDescription: ""
    property int maximumLength: 32767

    Accessible.name: accessibleName
    Accessible.description: accessibleDescription
    activeFocusOnTab: true
    focusPolicy: Qt.StrongFocus
    selectByMouse: true
    wrapMode: TextEdit.Wrap
    implicitHeight: theme.textAreaHeight
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

    onTextChanged: {
        if (text.length > maximumLength)
            text = text.slice(0, maximumLength)
    }

    background: Rectangle {
        radius: theme.radius
        color: theme.surface
        border.color: root.activeFocus ? theme.focus : theme.border
        border.width: root.activeFocus ? theme.focusWidth : theme.borderWidth
    }
}
