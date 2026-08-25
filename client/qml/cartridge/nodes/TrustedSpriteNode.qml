import QtQuick

Rectangle {
    id: root

    property var nodeData: ({
        "asset_token": "", "frame_width": 1, "frame_height": 1,
        "frame_count": 1, "frames_per_second": 1, "animated": false,
        "accessible_label": ""
    })
    property string assetRoot: ""
    property real scaleFactor: 1
    property bool highContrast: false
    property bool reducedMotion: false
    property bool mutedAudio: false

    width: parent ? parent.width : 640
    height: 160 * scaleFactor
    color: highContrast ? "#000000" : "#0b1420"
    border.color: highContrast ? "#ffffff" : "#263950"
    Accessible.role: Accessible.Animation
    Accessible.name: nodeData.accessible_label

    AnimatedSprite {
        anchors.centerIn: parent
        width: Math.min(parent.width - 16, root.nodeData.frame_width * root.scaleFactor * 4)
        height: Math.min(parent.height - 16, root.nodeData.frame_height * root.scaleFactor * 4)
        source: root.assetRoot !== "" && root.nodeData.asset_token !== ""
            ? "file://" + encodeURI(root.assetRoot + "/" + root.nodeData.asset_token) : ""
        frameWidth: root.nodeData.frame_width
        frameHeight: root.nodeData.frame_height
        frameCount: root.nodeData.frame_count
        frameRate: root.nodeData.frames_per_second
        running: root.nodeData.animated && !root.reducedMotion
        interpolate: false
    }
}
