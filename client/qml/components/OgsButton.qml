import QtQuick
import QtQuick.Controls

Button {
    id: root

    property string accessibleName: text

    Accessible.name: accessibleName
    activeFocusOnTab: true
    focusPolicy: Qt.StrongFocus
    font.family: "monospace"
    font.bold: true
    font.pixelSize: 14
    leftPadding: 18
    rightPadding: 18
    topPadding: 11
    bottomPadding: 11

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
        text: root.text
        textFormat: Text.PlainText
        color: root.enabled ? "#eef7ff" : "#728397"
        font: root.font
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
    }

    background: Rectangle {
        radius: 3
        color: root.down ? "#294966"
                         : root.hovered ? "#1d3348" : "#132537"
        border.color: root.activeFocus ? "#ffffff"
                                       : root.enabled ? "#5ee6a8" : "#415066"
        border.width: root.activeFocus ? 2 : 1
    }
}
