import QtQuick
import "../../components" as Components

Rectangle {
    id: root

    property var nodeData: ({"asset_token": "", "accessible_label": ""})
    property string assetRoot: ""
    property real scaleFactor: 1
    property bool highContrast: false
    property bool reducedMotion: false
    property bool mutedAudio: false

    Components.OgsTheme { id: theme }

    width: parent ? parent.width : 640
    height: 160 * scaleFactor
    color: highContrast ? theme.highContrastBackground : theme.background
    border.color: highContrast ? theme.highContrastForeground : theme.borderMuted
    Accessible.role: Accessible.Graphic
    Accessible.name: nodeData.accessible_label

    Image {
        anchors.fill: parent
        anchors.margins: 8
        source: root.assetRoot !== "" && root.nodeData.asset_token !== ""
            ? (root.assetRoot.startsWith("http://") || root.assetRoot.startsWith("https://")
               ? root.assetRoot + "/" + root.nodeData.asset_token
               : "file://" + encodeURI(root.assetRoot + "/" + root.nodeData.asset_token)) : ""
        sourceSize.width: Math.min(2048, Math.max(1, Math.ceil(width * 2)))
        sourceSize.height: Math.min(2048, Math.max(1, Math.ceil(height * 2)))
        fillMode: Image.PreserveAspectFit
        asynchronous: true
        cache: true
    }
}
