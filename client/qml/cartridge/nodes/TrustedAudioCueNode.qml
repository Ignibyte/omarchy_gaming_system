import QtQuick
import QtMultimedia

Item {
    id: root

    property var nodeData: ({
        "asset_token": "", "looped": false, "muted": true, "accessible_label": ""
    })
    property string assetRoot: ""
    property real scaleFactor: 1
    property bool highContrast: false
    property bool reducedMotion: false
    property bool mutedAudio: false

    width: parent ? parent.width : 640
    height: 1
    visible: false
    Accessible.role: Accessible.Sound
    Accessible.name: nodeData.accessible_label

    MediaPlayer {
        id: player
        source: root.assetRoot !== "" && root.nodeData.asset_token !== ""
            ? (root.assetRoot.startsWith("http://") || root.assetRoot.startsWith("https://")
               ? root.assetRoot + "/" + root.nodeData.asset_token
               : "file://" + encodeURI(root.assetRoot + "/" + root.nodeData.asset_token)) : ""
        loops: root.nodeData.looped ? MediaPlayer.Infinite : 1
        audioOutput: AudioOutput {
            muted: root.mutedAudio || root.nodeData.muted
        }
    }

    Timer {
        id: playback
        interval: 0
        repeat: false
        onTriggered: {
            if (!root.mutedAudio && !root.nodeData.muted)
                player.play()
        }
    }

    onNodeDataChanged: playback.restart()
}
