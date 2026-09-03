import QtQuick
import "../../components" as Components

Rectangle {
    id: root

    property var nodeData: ({
        "rows": 1, "columns": 1, "cells": [""], "action": "", "accessible_label": ""
    })
    property string assetRoot: ""
    property real scaleFactor: 1
    property bool highContrast: false
    property bool reducedMotion: false
    property bool mutedAudio: false
    property bool actionsEnabled: true
    property int selectedIndex: 0
    signal actionRequested(string action, var payload)

    Components.OgsTheme { id: theme }

    width: parent ? parent.width : 640
    height: Math.max(120 * scaleFactor, nodeData.rows * 34 * scaleFactor + 16)
    color: highContrast ? theme.highContrastBackground : theme.background
    border.color: highContrast ? theme.highContrastForeground : theme.borderMuted
    border.width: activeFocus ? theme.focusWidth : theme.borderWidth
    activeFocusOnTab: true
    opacity: actionsEnabled ? 1 : 0.6
    Accessible.role: Accessible.List
    Accessible.name: nodeData.accessible_label
    Accessible.focused: activeFocus
    Accessible.onPressAction: triggerSelected()

    function moveSelection(deltaRow, deltaColumn) {
        const row = Math.floor(selectedIndex / nodeData.columns)
        const column = selectedIndex % nodeData.columns
        const nextRow = Math.max(0, Math.min(nodeData.rows - 1, row + deltaRow))
        const nextColumn = Math.max(0, Math.min(nodeData.columns - 1, column + deltaColumn))
        selectedIndex = nextRow * nodeData.columns + nextColumn
    }

    function triggerSelected() {
        if (!actionsEnabled)
            return
        actionRequested(nodeData.action, {
            "row": Math.floor(selectedIndex / nodeData.columns),
            "column": selectedIndex % nodeData.columns
        })
    }

    Keys.onLeftPressed: moveSelection(0, -1)
    Keys.onRightPressed: moveSelection(0, 1)
    Keys.onUpPressed: moveSelection(-1, 0)
    Keys.onDownPressed: moveSelection(1, 0)
    Keys.onReturnPressed: function(event) { root.triggerFromKey(event) }
    Keys.onEnterPressed: function(event) { root.triggerFromKey(event) }
    Keys.onSpacePressed: function(event) { root.triggerFromKey(event) }

    function triggerFromKey(event) {
        event.accepted = true
        if (event.isAutoRepeat || !actionsEnabled)
            return false
        triggerSelected()
        return true
    }

    Grid {
        anchors.fill: parent
        anchors.margins: 8
        columns: root.nodeData.columns
        spacing: 3

        Repeater {
            model: root.nodeData.cells

            delegate: Rectangle {
                required property int index
                required property string modelData
                width: (root.width - 16 - (root.nodeData.columns - 1) * 3) / root.nodeData.columns
                height: (root.height - 16 - (root.nodeData.rows - 1) * 3) / root.nodeData.rows
                color: root.selectedIndex === index
                    ? (root.highContrast ? theme.highContrastForeground : theme.surfacePressed)
                    : (root.highContrast ? theme.highContrastBackground : theme.surfaceRaised)
                border.color: root.highContrast ? theme.highContrastForeground : theme.border

                Text {
                    anchors.centerIn: parent
                    text: parent.modelData
                    textFormat: Text.PlainText
                    color: root.highContrast && root.selectedIndex === parent.index
                        ? theme.highContrastBackground
                        : root.highContrast ? theme.highContrastForeground : theme.textPrimary
                    font.family: theme.fontFamily
                    font.pixelSize: theme.bodySize * root.scaleFactor
                    elide: Text.ElideRight
                }

                MouseArea {
                    anchors.fill: parent
                    enabled: root.actionsEnabled
                    onClicked: {
                        root.selectedIndex = parent.index
                        root.forceActiveFocus()
                        root.triggerSelected()
                    }
                }
            }
        }
    }
}
