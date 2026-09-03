import QtQuick
import "../components" as Components
import "../cartridge" as Cartridge

Window {
    id: root

    Components.OgsTheme { id: theme }

    width: 920
    height: 640
    visible: true
    color: theme.background
    title: "OmarchyGS Usurper Local Play — Development"

    property bool smokeTest: Qt.application.arguments.indexOf("--smoke-test") !== -1
    property bool busy: true
    property int smokeStep: 0
    property int smokeLoadRetries: 0
    property int confirmedActions: 0
    property int revision: -1
    property string screenId: ""
    property string statusText: "Connecting to the local provider..."
    property string localEndpoint: ""
    property string localCapability: ""
    readonly property var smokeActions: [
        {"screen": "entry", "action": "continue"},
        {"screen": "create-race", "action": "choose_race_human"},
        {"screen": "create-class", "action": "choose_class_alchemist"},
        {"screen": "main-street", "action": "enter_dungeon_level_1"},
        {"screen": "dungeon", "action": "enter_dungeon_level_21"},
        {"screen": "dungeon", "action": "look"},
        {"screen": "combat", "action": "attack"}
    ]

    function argument(prefix) {
        for (let index = 0; index < Qt.application.arguments.length; index++) {
            const value = Qt.application.arguments[index]
            if (value.indexOf(prefix) === 0)
                return value.slice(prefix.length)
        }
        return ""
    }

    function endpoint() {
        return localEndpoint
    }

    function capability() {
        return localCapability
    }

    function startupFile() {
        return argument("--startup-file=")
    }

    function exactKeys(value, expected) {
        if (!value || typeof value !== "object" || Array.isArray(value))
            return false
        return JSON.stringify(Object.keys(value).sort())
                === JSON.stringify(expected.slice().sort())
    }

    function validIdentifier(value) {
        return typeof value === "string" && /^[a-z][a-z0-9._-]{0,95}$/.test(value)
    }

    function validSetup() {
        const match = endpoint().match(/^http:\/\/127\.0\.0\.1:([1-9][0-9]{0,4})$/)
        return match !== null && Number(match[1]) <= 65535
            && /^[0-9a-f]{64}$/.test(capability())
    }

    function fail(message) {
        busy = false
        statusText = message
        if (smokeTest)
            Qt.exit(2)
    }

    function loadStartup() {
        const path = startupFile()
        if (path.length === 0 || path[0] !== "/" || path.indexOf("\0") !== -1) {
            fail("The local provider startup file path is invalid.")
            return
        }
        const request = new XMLHttpRequest()
        request.onreadystatechange = function() {
            if (request.readyState !== XMLHttpRequest.DONE)
                return
            if ((request.status !== 0 && request.status !== 200)
                    || request.responseText.length > 1024) {
                fail("The local provider startup file could not be read.")
                return
            }
            try {
                const document = JSON.parse(request.responseText)
                if (!exactKeys(document, ["format", "endpoint", "capability"])
                        || document.format !== "omarchygs.local-play-startup/v1") {
                    fail("The local provider startup file is invalid.")
                    return
                }
                localEndpoint = document.endpoint
                localCapability = document.capability
                if (!validSetup()) {
                    localEndpoint = ""
                    localCapability = ""
                    fail("The local provider endpoint or capability is invalid.")
                    return
                }
                loadSession()
            } catch (error) {
                fail("The local provider startup file is not valid JSON.")
            }
        }
        request.open("GET", "file://" + path)
        request.send()
    }

    function acceptResponse(document, confirmation) {
        if (!exactKeys(document, ["format", "revision", "screen_id",
                                  "asset_generation", "asset_capability", "plan"])
                || document.format !== "omarchygs.local-play-session/v1"
                || !Number.isInteger(document.revision) || document.revision < 0
                || !validIdentifier(document.screen_id)
                || !Number.isInteger(document.asset_generation)
                || document.asset_generation < 1
                || typeof document.asset_capability !== "string"
                || !/^[0-9a-f]{64}$/.test(document.asset_capability)) {
            fail("The local provider returned an invalid session envelope.")
            return false
        }
        surface.assetRoot = endpoint() + "/v1/assets/" + document.asset_capability
            + "/" + document.asset_generation
        if (!surface.acceptPlan(document.plan)) {
            fail("The trusted renderer rejected the local provider plan.")
            return false
        }
        revision = document.revision
        screenId = document.screen_id
        busy = false
        statusText = confirmation
        smokeLoadRetries = 0
        if (smokeTest)
            smokeTimer.restart()
        else
            Qt.callLater(function() { surface.focusInitial() })
        return true
    }

    function runSmokeStep() {
        if (!smokeTest || busy || smokeStep >= smokeActions.length)
            return
        if (surface.loadedNodeCount() !== surface.instantiatedNodeCount) {
            smokeLoadRetries++
            if (smokeLoadRetries > 30) {
                fail("The trusted renderer did not settle to one loaded item per node.")
                return
            }
            smokeTimer.restart()
            return
        }
        const expected = smokeActions[smokeStep]
        if (screenId !== expected.screen
                || surface.instantiatedNodeCount !== surface.acceptedPlan.nodes.length
                || surface.actionNodeCount(expected.action) !== 1
                || !surface.triggerAction(expected.action)) {
            fail("The trusted renderer exposed duplicate, missing, or inert controls.")
        }
    }

    function send(method, path, body, confirmation) {
        if (!validSetup()) {
            fail("The local provider endpoint or capability is invalid.")
            return
        }
        busy = true
        const request = new XMLHttpRequest()
        request.onreadystatechange = function() {
            if (request.readyState !== XMLHttpRequest.DONE)
                return
            if (request.status !== 200 || request.responseText.length > 2 * 1024 * 1024) {
                fail("The local provider rejected the request; state was not confirmed.")
                return
            }
            try {
                if (acceptResponse(JSON.parse(request.responseText), confirmation)
                        && method === "POST") {
                    confirmedActions++
                    console.log("OGS_LOCAL_PLAY_ACTION confirmed=true revision="
                        + revision + " screen=" + screenId)
                    if (smokeTest) {
                        smokeStep++
                        if (smokeStep === smokeActions.length)
                            Qt.quit()
                    }
                }
            } catch (error) {
                fail("The local provider response was not valid JSON.")
            }
        }
        request.open(method, endpoint() + path)
        request.setRequestHeader("X-OmarchyGS-Local-Capability", capability())
        if (body !== null)
            request.setRequestHeader("Content-Type", "application/json")
        request.send(body === null ? undefined : JSON.stringify(body))
    }

    function loadSession() {
        send("GET", "/v1/session", null, "Provider session ready.")
    }

    function requestAction(action, payload) {
        const selectedAction = String(action)
        if (busy || revision < 0 || !validIdentifier(selectedAction)
                || !payload || typeof payload !== "object" || Array.isArray(payload)
                || Object.keys(payload).length !== 0)
            return false
        send("POST", "/v1/actions", {
            "expected_revision": revision,
            "screen_id": screenId,
            "action": selectedAction,
            "payload": payload
        }, selectedAction.indexOf("navigate.") === 0
           ? "Signed navigation confirmed; provider state unchanged."
           : "Provider action confirmed at revision " + (revision + 1) + ".")
        return true
    }

    Component.onCompleted: loadStartup()

    Column {
        anchors.fill: parent

        Rectangle {
            width: parent.width
            height: 52
            color: theme.surfaceRaised
            border.color: theme.warning
            border.width: theme.borderWidth

            Column {
                anchors.fill: parent
                anchors.margins: 7

                Text {
                    width: parent.width
                    text: "LOCAL PROVIDER PLAY // DEVELOPMENT ONLY"
                    textFormat: Text.PlainText
                    color: theme.warning
                    font.family: theme.fontFamily
                    font.bold: true
                    font.pixelSize: theme.captionSize
                    elide: Text.ElideRight
                }

                Text {
                    width: parent.width
                    text: root.statusText
                    textFormat: Text.PlainText
                    color: theme.textSecondary
                    font.family: theme.fontFamily
                    font.pixelSize: theme.captionSize
                    elide: Text.ElideRight
                }
            }
        }

        Cartridge.TrustedCartridgeSurface {
            id: surface
            width: parent.width
            height: parent.height - 52
            assetRoot: ""
            actionsEnabled: !root.busy && root.revision >= 0
            onActionRequested: function(action, payload) {
                root.requestAction(action, payload)
            }
        }
    }

    Timer {
        id: smokeTimer
        interval: 100
        repeat: false
        onTriggered: root.runSmokeStep()
    }

    Timer {
        interval: 20000
        running: root.smokeTest
        repeat: false
        onTriggered: Qt.exit(4)
    }
}
