import QtQuick

Window {
    id: root

    width: 920
    height: 600
    minimumWidth: 640
    minimumHeight: 420
    visible: true
    title: "OmarchyGS // Game Cartridge Proof"
    color: "#070b12"

    property string brokerUrl: argumentValue("--broker-url=", "http://127.0.0.1:19092")
        + "/v1/proof"
    property string connectionState: "LOADING"
    property string connectionDetail: "Verifying cartridge and contacting provider..."
    property color statusColor: "#f4c95d"
    property bool requestInFlight: false
    property bool smokeTest: Qt.application.arguments.indexOf("--smoke-test") !== -1
    property var presentation: ({"screens": []})
    property var gameView: ({"headline": "WAITING", "board": ["·"], "turn": 0, "status": "loading"})
    property int measuredFrames: 0
    property double frameTotalMs: 0
    property double frameMaxMs: 0
    property double priorFrameMs: 0

    function argumentValue(prefix, fallback) {
        const applicationArguments = Qt.application.arguments
        for (let index = 0; index < applicationArguments.length; index += 1) {
            if (applicationArguments[index].indexOf(prefix) === 0)
                return applicationArguments[index].substring(prefix.length)
        }
        return fallback
    }

    function bindingValue(name) {
        if (name === "headline")
            return gameView.headline
        if (name === "board")
            return gameView.board
        if (name === "status")
            return gameView.status
        return ""
    }

    function componentFor(kind) {
        if (kind === "terminal")
            return terminalNode
        if (kind === "grid")
            return gridNode
        if (kind === "status")
            return statusNode
        return unsupportedNode
    }

    function loadProof() {
        if (requestInFlight)
            return
        requestInFlight = true
        connectionState = "LOADING"
        connectionDetail = "Verifying signed presentation and remote authority..."
        statusColor = "#f4c95d"

        const request = new XMLHttpRequest()
        request.onreadystatechange = function() {
            if (request.readyState !== XMLHttpRequest.DONE)
                return
            requestInFlight = false
            if (request.status === 200) {
                try {
                    const response = JSON.parse(request.responseText)
                    if (response.status !== "ready"
                            || response.revision !== 1
                            || !response.idempotent_replay
                            || !response.duplicate_event_rejected
                            || !response.pairwise_subject_verified
                            || response.raw_persona_disclosed
                            || response.device_token_disclosed
                            || response.database_access_disclosed
                            || !response.presentation
                            || !response.presentation.screens
                            || response.presentation.screens.length < 1)
                        throw new Error("proof invariants failed")
                    presentation = response.presentation
                    gameView = response.view
                    connectionState = "READY"
                    connectionDetail = response.detail
                    statusColor = "#5ee6a8"
                } catch (error) {
                    connectionState = "PROTOCOL ERROR"
                    connectionDetail = "The trusted broker returned invalid cartridge data."
                    statusColor = "#ff6b7a"
                    if (smokeTest) {
                        console.error("OGS_CARTRIDGE_PROTOCOL_ERROR " + error)
                        Qt.exit(2)
                    }
                }
            } else {
                connectionState = "OFFLINE"
                connectionDetail = request.status > 0
                    ? "Provider path returned HTTP " + request.status + ". Press R to retry."
                    : "No answer from the trusted broker. Press R to retry."
                statusColor = "#ff6b7a"
                if (smokeTest) {
                    console.error("OGS_CARTRIDGE_OFFLINE status=" + request.status)
                    Qt.exit(2)
                }
            }
        }
        request.open("POST", brokerUrl)
        request.setRequestHeader("Content-Type", "application/json")
        request.send("{}")
    }

    Component.onCompleted: loadProof()

    FrameAnimation {
        running: root.smokeTest && root.connectionState === "READY"
        onTriggered: {
            const now = Date.now()
            if (root.priorFrameMs > 0) {
                const delta = now - root.priorFrameMs
                root.frameTotalMs += delta
                root.frameMaxMs = Math.max(root.frameMaxMs, delta)
                root.measuredFrames += 1
            }
            root.priorFrameMs = now
            if (root.measuredFrames >= 120) {
                const average = root.frameTotalMs / root.measuredFrames
                console.log("OGS_CARTRIDGE_FRAME_METRICS frames=" + root.measuredFrames
                    + " average_ms=" + average.toFixed(2)
                    + " max_ms=" + root.frameMaxMs.toFixed(2))
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 12000
        repeat: false
        running: root.smokeTest
        onTriggered: {
            console.error("OGS_CARTRIDGE_FRAME_METRICS timeout state=" + root.connectionState)
            Qt.exit(2)
        }
    }

    NumberAnimation {
        target: scanLine
        property: "y"
        from: 8
        to: root.height - 8
        duration: 2200
        loops: Animation.Infinite
        running: root.connectionState === "READY"
    }

    Rectangle {
        anchors.fill: parent
        color: "transparent"
        border.color: "#182538"
        border.width: 1

        Rectangle {
            width: parent.width
            height: 6
            color: root.statusColor
        }

        Rectangle {
            id: scanLine
            width: parent.width
            height: 1
            color: "#183f4f"
            opacity: 0.45
        }

        FocusScope {
            anchors.fill: parent
            focus: true
            Keys.onReturnPressed: root.loadProof()
            Keys.onPressed: function(event) {
                if (event.key === Qt.Key_R) {
                    root.loadProof()
                    event.accepted = true
                }
            }

            Column {
                anchors.centerIn: parent
                width: Math.min(parent.width - 80, 760)
                spacing: 18

                Text {
                    width: parent.width
                    text: "OMARCHYGS // INSERTED CARTRIDGE"
                    color: "#8aa4c0"
                    font.family: "monospace"
                    font.pixelSize: 15
                    font.letterSpacing: 2
                    horizontalAlignment: Text.AlignHCenter
                }

                Text {
                    width: parent.width
                    text: root.connectionState
                    color: root.statusColor
                    font.family: "monospace"
                    font.bold: true
                    font.pixelSize: 30
                    horizontalAlignment: Text.AlignHCenter
                    Accessible.name: "Cartridge connection state " + text
                }

                Column {
                    width: parent.width
                    spacing: 12
                    visible: root.connectionState === "READY"

                    Repeater {
                        model: root.presentation.screens.length > 0
                            ? root.presentation.screens[0].nodes : []
                        delegate: Loader {
                            required property var modelData
                            width: parent.width
                            sourceComponent: root.componentFor(modelData.kind)
                            onLoaded: item.node = modelData
                        }
                    }
                }

                Text {
                    width: parent.width
                    visible: root.connectionState !== "READY"
                    text: root.connectionDetail
                    color: "#d5e2ef"
                    font.family: "monospace"
                    font.pixelSize: 16
                    horizontalAlignment: Text.AlignHCenter
                    wrapMode: Text.Wrap
                }

                Text {
                    width: parent.width
                    text: "R / ENTER  RETRY   •   ESC  EXIT"
                    color: "#546b82"
                    font.family: "monospace"
                    font.pixelSize: 12
                    horizontalAlignment: Text.AlignHCenter
                }
            }
        }
    }

    Shortcut {
        sequence: "Escape"
        onActivated: Qt.quit()
    }

    Component {
        id: terminalNode
        Rectangle {
            property var node: ({})
            implicitHeight: 64
            radius: 3
            color: "#0d1723"
            border.color: "#28415a"
            Accessible.name: node.accessible_label
            Text {
                anchors.centerIn: parent
                text: root.bindingValue(parent.node.text_binding)
                textFormat: Text.PlainText
                color: "#f4c95d"
                font.family: "monospace"
                font.bold: true
                font.pixelSize: 20
            }
        }
    }

    Component {
        id: gridNode
        Rectangle {
            property var node: ({})
            implicitHeight: 222
            radius: 3
            color: "#0b131d"
            border.color: "#28415a"
            Accessible.name: node.accessible_label
            Grid {
                anchors.centerIn: parent
                rows: parent.node.rows
                columns: parent.node.columns
                spacing: 6
                Repeater {
                    model: root.bindingValue(parent.parent.node.cells_binding)
                    delegate: Rectangle {
                        required property string modelData
                        width: 62
                        height: 62
                        radius: 3
                        color: "#132537"
                        border.color: "#3a6685"
                        Text {
                            anchors.centerIn: parent
                            text: modelData
                            textFormat: Text.PlainText
                            color: "#5ee6a8"
                            font.family: "monospace"
                            font.bold: true
                            font.pixelSize: 28
                        }
                    }
                }
            }
        }
    }

    Component {
        id: statusNode
        Rectangle {
            property var node: ({})
            implicitHeight: 44
            radius: 3
            color: "#102219"
            border.color: "#2f7050"
            Accessible.name: node.accessible_label
            Text {
                anchors.centerIn: parent
                text: "TURN " + root.gameView.turn + " // "
                    + root.bindingValue(parent.node.text_binding).toUpperCase()
                textFormat: Text.PlainText
                color: "#5ee6a8"
                font.family: "monospace"
                font.pixelSize: 14
            }
        }
    }

    Component {
        id: unsupportedNode
        Rectangle {
            property var node: ({})
            implicitHeight: 44
            color: "#2a1117"
            border.color: "#ff6b7a"
            Text {
                anchors.centerIn: parent
                text: "UNSUPPORTED TRUSTED NODE"
                color: "#ff6b7a"
                font.family: "monospace"
            }
        }
    }
}
