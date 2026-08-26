import QtQuick
import QtQuick.Particles
import "../../components" as Components

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

    Components.OgsTheme { id: theme }

    width: parent ? parent.width : 640
    height: 140 * scaleFactor
    color: highContrast ? theme.highContrastBackground : theme.background
    border.color: highContrast ? theme.highContrastForeground : theme.borderMuted
    Accessible.role: Accessible.Animation
    Accessible.name: nodeData.accessible_label

    ParticleSystem {
        id: system
        running: root.nodeData.running && !root.reducedMotion
    }

    ImageParticle {
        system: system
        color: root.highContrast ? theme.highContrastForeground
            : root.nodeData.preset === "sparks" ? theme.warning
            : root.nodeData.preset === "snow" ? theme.textSecondary : theme.accent
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
