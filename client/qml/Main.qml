import QtQuick

Window {
    id: root

    width: 920
    height: 600
    minimumWidth: 640
    minimumHeight: 420
    visible: true
    title: "Omarchy Gaming System"
    color: "#070b12"

    property string healthUrl: "http://127.0.0.1:8080/health"
    property string connectionState: "CONNECTING"
    property string connectionDetail: "Dialing the local game service..."
    property color statusColor: "#f4c95d"
    property bool requestInFlight: false
    property bool smokeTest: Qt.application.arguments.indexOf("--smoke-test") !== -1

    function refreshHealth() {
        if (requestInFlight)
            return

        requestInFlight = true
        connectionState = "CONNECTING"
        connectionDetail = "Calling " + healthUrl
        statusColor = "#f4c95d"

        const request = new XMLHttpRequest()
        request.onreadystatechange = function() {
            if (request.readyState !== XMLHttpRequest.DONE)
                return

            requestInFlight = false

            if (request.status === 200) {
                try {
                    const response = JSON.parse(request.responseText)
                    connectionState = "CONNECTED"
                    connectionDetail = response.service + " v" + response.version
                        + "  /  database " + response.database
                    statusColor = "#5ee6a8"
                } catch (error) {
                    connectionState = "PROTOCOL ERROR"
                    connectionDetail = "The server returned invalid JSON."
                    statusColor = "#ff6b7a"
                }
            } else {
                connectionState = "OFFLINE"
                connectionDetail = request.status > 0
                    ? "The server returned HTTP " + request.status + "."
                    : "No answer from the local server."
                statusColor = "#ff6b7a"
            }

            if (smokeTest)
                Qt.quit()
        }

        request.open("GET", healthUrl)
        request.send()
    }

    Component.onCompleted: refreshHealth()

    Timer {
        interval: 15000
        repeat: !root.smokeTest
        running: !root.smokeTest
        onTriggered: root.refreshHealth()
    }

    Timer {
        interval: 5000
        repeat: false
        running: root.smokeTest
        onTriggered: Qt.quit()
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

        Column {
            anchors.centerIn: parent
            width: Math.min(parent.width - 80, 700)
            spacing: 24

            Text {
                width: parent.width
                text: "OMARCHY // GAMES"
                color: "#8aa4c0"
                font.family: "monospace"
                font.pixelSize: 18
                font.letterSpacing: 3
                horizontalAlignment: Text.AlignHCenter
            }

            Text {
                width: parent.width
                text: root.connectionState
                color: root.statusColor
                font.family: "monospace"
                font.bold: true
                font.pixelSize: 42
                horizontalAlignment: Text.AlignHCenter
            }

            Text {
                width: parent.width
                text: root.connectionDetail
                color: "#d5e2ef"
                font.family: "monospace"
                font.pixelSize: 16
                horizontalAlignment: Text.AlignHCenter
                wrapMode: Text.Wrap
            }

            Rectangle {
                anchors.horizontalCenter: parent.horizontalCenter
                width: 190
                height: 48
                radius: 3
                color: refreshArea.containsMouse ? "#1d3348" : "#132537"
                border.color: root.statusColor

                Text {
                    anchors.centerIn: parent
                    text: root.requestInFlight ? "DIALING..." : "RECONNECT"
                    color: "#eef7ff"
                    font.family: "monospace"
                    font.bold: true
                    font.pixelSize: 15
                }

                MouseArea {
                    id: refreshArea
                    anchors.fill: parent
                    hoverEnabled: true
                    enabled: !root.requestInFlight
                    onClicked: root.refreshHealth()
                }
            }
        }

        Text {
            anchors.left: parent.left
            anchors.bottom: parent.bottom
            anchors.margins: 22
            text: "FIRST LINK // HEALTH PROBE"
            color: "#546b82"
            font.family: "monospace"
            font.pixelSize: 12
        }
    }
}
