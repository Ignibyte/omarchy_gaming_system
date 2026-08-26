import QtQuick

Rectangle {
    id: root

    property string tone: "info"
    property bool highlighted: false

    radius: theme.radius
    color: highlighted ? theme.surfaceRaised : theme.surface
    border.color: highlighted ? theme.toneColor(tone) : theme.borderMuted
    border.width: theme.borderWidth

    OgsTheme { id: theme }
}
