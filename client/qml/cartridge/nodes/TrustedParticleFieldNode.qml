import QtQuick
import QtQuick.Particles

Rectangle {
    id: root

    property var nodeData: ({
        "particle_count": 0, "preset": "stars", "running": false, "accessible_label": ""
    })
    property string assetRoot: ""
    property real scaleFactor: 1
    property bool highContrast: false
    property bool reducedMotion: false
    property bool mutedAudio: false

    width: parent ? parent.width : 640
    height: 140 * scaleFactor
    color: highContrast ? "#000000" : "#081320"
    border.color: highContrast ? "#ffffff" : "#263950"
    Accessible.role: Accessible.Animation
    Accessible.name: nodeData.accessible_label

    ParticleSystem {
        id: system
        running: root.nodeData.running && !root.reducedMotion
    }

    ImageParticle {
        system: system
        color: root.highContrast ? "#ffffff"
            : root.nodeData.preset === "sparks" ? "#f4c95d"
            : root.nodeData.preset === "snow" ? "#d5e2ef" : "#5ee6a8"
        alpha: 0.8
    }

    Emitter {
        anchors.fill: parent
        system: system
        emitRate: Math.min(root.nodeData.particle_count, 240)
        lifeSpan: Math.max(1000, Math.ceil(root.nodeData.particle_count
            / Math.max(1, emitRate)) * 1000)
        size: root.nodeData.preset === "sparks" ? 3 : 2
        endSize: root.nodeData.preset === "snow" ? 3 : 1
    }
}
