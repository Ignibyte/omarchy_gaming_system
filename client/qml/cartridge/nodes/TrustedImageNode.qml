import QtQuick

Rectangle {
    id: root

    property var nodeData: ({"asset_token": "", "accessible_label": ""})
    property string assetRoot: ""
    property real scaleFactor: 1
    property bool highContrast: false
    property bool reducedMotion: false
    property bool mutedAudio: false

    width: parent ? parent.width : 640
    height: 160 * scaleFactor
    color: highContrast ? "#000000" : "#0b1420"
    border.color: highContrast ? "#ffffff" : "#263950"
    Accessible.role: Accessible.Graphic
    Accessible.name: nodeData.accessible_label

    Image {
        anchors.fill: parent
        anchors.margins: 8
        source: root.assetRoot !== "" && root.nodeData.asset_token !== ""
            ? "file://" + encodeURI(root.assetRoot + "/" + root.nodeData.asset_token) : ""
        sourceSize.width: Math.min(2048, Math.max(1, Math.ceil(width * 2)))
        sourceSize.height: Math.min(2048, Math.max(1, Math.ceil(height * 2)))
        fillMode: Image.PreserveAspectFit
        asynchronous: true
        cache: true
    }
}
