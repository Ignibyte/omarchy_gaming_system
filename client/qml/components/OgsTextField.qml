import QtQuick
import QtQuick.Controls

TextField {
    id: root

    required property string accessibleName

    Accessible.name: accessibleName
    activeFocusOnTab: true
    focusPolicy: Qt.StrongFocus
    selectByMouse: true
    color: "#eef7ff"
    placeholderTextColor: "#667f98"
    selectionColor: "#2b6c72"
    selectedTextColor: "#ffffff"
    font.family: "monospace"
    font.pixelSize: 15
    leftPadding: 12
    rightPadding: 12
    topPadding: 10
    bottomPadding: 10

    background: Rectangle {
        radius: 3
        color: "#0c1825"
        border.color: root.activeFocus ? "#ffffff" : "#36516b"
        border.width: root.activeFocus ? 2 : 1
    }
}
