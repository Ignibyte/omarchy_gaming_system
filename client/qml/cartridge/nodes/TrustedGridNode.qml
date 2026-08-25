import QtQuick

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
    property int selectedIndex: 0
    signal actionRequested(string action, var payload)

    width: parent ? parent.width : 640
    height: Math.max(120 * scaleFactor, nodeData.rows * 34 * scaleFactor + 16)
    color: highContrast ? "#000000" : "#0b1420"
    border.color: highContrast ? "#ffffff" : "#263950"
    border.width: activeFocus ? 3 : 1
    activeFocusOnTab: true
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
        actionRequested(nodeData.action, {
            "row": Math.floor(selectedIndex / nodeData.columns),
            "column": selectedIndex % nodeData.columns
        })
    }

    Keys.onLeftPressed: moveSelection(0, -1)
    Keys.onRightPressed: moveSelection(0, 1)
    Keys.onUpPressed: moveSelection(-1, 0)
    Keys.onDownPressed: moveSelection(1, 0)
    Keys.onReturnPressed: triggerSelected()
    Keys.onEnterPressed: triggerSelected()
    Keys.onSpacePressed: triggerSelected()

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
                    ? (root.highContrast ? "#ffffff" : "#1d5c72")
                    : (root.highContrast ? "#000000" : "#132537")
                border.color: root.highContrast ? "#ffffff" : "#365572"

                Text {
                    anchors.centerIn: parent
                    text: parent.modelData
                    textFormat: Text.PlainText
                    color: root.highContrast && root.selectedIndex === parent.index
                        ? "#000000" : "#eef7ff"
                    font.family: "monospace"
                    font.pixelSize: 13 * root.scaleFactor
                    elide: Text.ElideRight
                }

                MouseArea {
                    anchors.fill: parent
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
