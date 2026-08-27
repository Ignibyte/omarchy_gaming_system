import QtQuick
import "../components" as Components

Window {
    id: root

    Components.OgsTheme { id: theme }

    width: 920
    height: 600
    visible: true
    color: theme.background
    title: "OmarchyGS Cartridge Preview"

    property bool smokeTest: Qt.application.arguments.indexOf("--smoke-test") !== -1
    property int frames: 0
    property int warmupFrames: 0
    property real totalFrameMs: 0
    property real maximumFrameMs: 0
    property int actionRequests: 0
    property bool smokeExerciseComplete: false

    function argument(prefix) {
        for (let index = 0; index < Qt.application.arguments.length; index++) {
            const value = Qt.application.arguments[index]
            if (value.indexOf(prefix) === 0)
                return value.slice(prefix.length)
        }
        return ""
    }

    function localUrl(path) {
        return "file://" + encodeURI(path)
    }

    function loadPlan() {
        const planPath = argument("--plan=")
        if (planPath === "" || surface.assetRoot === "") {
            surface.validationError = "Missing trusted preview paths"
            if (smokeTest)
                Qt.exit(2)
            return
        }
        const request = new XMLHttpRequest()
        request.onreadystatechange = function() {
            if (request.readyState !== XMLHttpRequest.DONE)
                return
            if ((request.status !== 0 && request.status !== 200)
                    || request.responseText.length > 2 * 1024 * 1024) {
                surface.validationError = "Render plan could not be read"
                if (smokeTest)
                    Qt.exit(2)
                return
            }
            try {
                if (!surface.acceptPlan(JSON.parse(request.responseText))) {
                    if (smokeTest)
                        Qt.exit(2)
                    return
                }
                if (smokeTest && surface.acceptedPlan.state !== "ready"
                        && surface.instantiatedNodeCount !== 0)
                    Qt.exit(3)
            } catch (error) {
                surface.validationError = "Render plan JSON is invalid"
                if (smokeTest)
                    Qt.exit(2)
            }
        }
        request.open("GET", localUrl(planPath))
        request.send()
    }

    Component.onCompleted: loadPlan()

    TrustedCartridgeSurface {
        id: surface
        anchors.fill: parent
        assetRoot: root.argument("--asset-root=")
        onActionRequested: function(action, payload) {
            root.actionRequests++
            console.log("OGS_CARTRIDGE_ACTION requested=" + action
                + " confirmed=false payload=" + JSON.stringify(payload))
        }
    }

    Timer {
        interval: 200
        running: root.smokeTest && surface.planAccepted && surface.acceptedPlan.state === "ready"
        repeat: false
        onTriggered: {
            const result = surface.smokeExercise()
            console.log("OGS_CARTRIDGE_INPUT_METRICS expected=" + result.expected
                + " exercised=" + result.exercised + " focus=" + result.focus_observed)
            if (result.expected !== result.exercised || !result.focus_observed)
                Qt.exit(5)
            root.smokeExerciseComplete = true
        }
    }

    FrameAnimation {
        running: root.smokeTest && surface.planAccepted
        onTriggered: {
            if (root.warmupFrames < 60) {
                root.warmupFrames++
                return
            }
            const milliseconds = frameTime * 1000
            root.frames++
            root.totalFrameMs += milliseconds
            root.maximumFrameMs = Math.max(root.maximumFrameMs, milliseconds)
            if (root.frames >= 120) {
                if (surface.acceptedPlan.state === "ready"
                        && surface.instantiatedNodeCount !== surface.acceptedPlan.nodes.length)
                    Qt.exit(3)
                if (surface.acceptedPlan.state === "ready"
                        && (!root.smokeExerciseComplete || root.actionRequests < 2))
                    Qt.exit(5)
                console.log("OGS_CARTRIDGE_RENDER_METRICS state=" + surface.acceptedPlan.state
                    + " nodes=" + surface.instantiatedNodeCount
                    + " frames=" + root.frames
                    + " average_ms=" + (root.totalFrameMs / root.frames).toFixed(3)
                    + " max_ms=" + root.maximumFrameMs.toFixed(3))
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 10000
        running: root.smokeTest
        repeat: false
        onTriggered: Qt.exit(4)
    }
}
