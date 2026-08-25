import QtQuick

QtObject {
    id: root

    property string baseUrl: ""
    property int timeoutMilliseconds: 10000
    property int maximumResponseBytes: 262144
    readonly property bool requestInFlight: root._activeGeneration !== 0
    readonly property bool hasBearer: root._bearerToken !== ""

    property string _bearerToken: ""
    property var _request: null
    property int _generation: 0
    property int _activeGeneration: 0
    property string _activeOperation: ""
    property var _retiredRequests: []

    signal finished(int generation, string operation, int status, string body,
                    string transportError)

    property Timer _timeoutTimer: Timer {
        interval: root.timeoutMilliseconds
        repeat: false
        onTriggered: {
            if (root._activeGeneration === 0)
                return
            const generation = root._activeGeneration
            const operation = root._activeOperation
            root._discardActiveRequest()
            root.finished(generation, operation, 0, "", "timeout")
        }
    }

    property Timer _retiredCleanupTimer: Timer {
        interval: 2000
        repeat: false
        onTriggered: root._retiredRequests = []
    }

    function exactKeys(value, expected) {
        if (!value || typeof value !== "object" || Array.isArray(value))
            return false
        const actualKeys = Object.keys(value).sort()
        const expectedKeys = expected.slice().sort()
        return JSON.stringify(actualKeys) === JSON.stringify(expectedKeys)
    }

    function normalizeEndpoint(candidate) {
        const value = String(candidate === undefined ? "" : candidate).trim()
        const match = value.match(/^(https?):\/\/(\[[0-9A-Fa-f:]+\]|[A-Za-z0-9.-]+)(?::([0-9]{1,5}))?\/?$/)
        if (!match)
            return {"ok": false, "error": "Enter an HTTPS server URL or a loopback HTTP URL."}

        const scheme = match[1].toLowerCase()
        const host = match[2].toLowerCase()
        const portText = match[3] || ""
        if (host !== "localhost" && !host.startsWith("[")
                && (host.startsWith(".") || host.endsWith(".") || host.indexOf("..") !== -1))
            return {"ok": false, "error": "The server host is not valid."}
        if (portText !== "") {
            const port = Number(portText)
            if (!Number.isInteger(port) || port < 1 || port > 65535)
                return {"ok": false, "error": "The server port must be between 1 and 65535."}
        }

        const loopback = host === "localhost" || host === "127.0.0.1" || host === "[::1]"
        if (scheme !== "https" && !loopback)
            return {"ok": false, "error": "Remote servers must use HTTPS."}

        return {
            "ok": true,
            "url": scheme + "://" + host + (portText === "" ? "" : ":" + portText)
        }
    }

    function configure(candidate) {
        const normalized = normalizeEndpoint(candidate)
        if (!normalized.ok)
            return normalized
        cancel()
        clearBearer()
        baseUrl = normalized.url
        return normalized
    }

    function installBearer(token) {
        _bearerToken = String(token)
    }

    function clearBearer() {
        _bearerToken = ""
    }

    function cancel() {
        if (_activeGeneration === 0)
            return
        _discardActiveRequest()
    }

    function request(operation, method, path, document, authenticated) {
        if (baseUrl === "")
            return 0
        if (authenticated && _bearerToken === "")
            return 0

        cancel()
        const generation = ++_generation
        _activeGeneration = generation
        _activeOperation = operation

        const request = new XMLHttpRequest()
        _request = request
        request.onreadystatechange = function() {
            if (generation !== root._activeGeneration)
                return

            if (request.readyState >= XMLHttpRequest.HEADERS_RECEIVED) {
                const contentLength = Number(request.getResponseHeader("Content-Length") || "0")
                if (Number.isFinite(contentLength) && contentLength > root.maximumResponseBytes) {
                    root._finish(generation, operation, 0, "", "response_too_large")
                    return
                }
            }
            if (request.readyState >= XMLHttpRequest.LOADING
                    && request.responseText.length > root.maximumResponseBytes) {
                root._finish(generation, operation, 0, "", "response_too_large")
                return
            }
            if (request.readyState !== XMLHttpRequest.DONE)
                return

            const expectedUrl = root.baseUrl + path
            if (request.responseURL && request.responseURL !== expectedUrl) {
                root._finish(generation, operation, 0, "", "unexpected_redirect")
                return
            }
            if (request.status === 0) {
                root._finish(generation, operation, 0, "", "network_error")
                return
            }
            root._finish(generation, operation, request.status, request.responseText, "")
        }

        request.open(method, baseUrl + path)
        request.setRequestHeader("Accept", "application/json")
        if (authenticated)
            request.setRequestHeader("Authorization", "Bearer " + _bearerToken)
        if (document !== undefined && document !== null)
            request.setRequestHeader("Content-Type", "application/json")

        _timeoutTimer.restart()
        request.send(document === undefined || document === null
                     ? null : JSON.stringify(document))
        return generation
    }

    function _finish(generation, operation, status, body, transportError) {
        if (generation !== _activeGeneration)
            return
        const request = _request
        _activeGeneration = 0
        _activeOperation = ""
        _request = null
        _timeoutTimer.stop()
        if (transportError !== "" && request)
            _retireRequest(request)
        finished(generation, operation, status, body, transportError)
    }

    function _discardActiveRequest() {
        const request = _request
        _activeGeneration = 0
        _activeOperation = ""
        _request = null
        _timeoutTimer.stop()
        if (request)
            _retireRequest(request)
    }

    function _retireRequest(request) {
        request.onreadystatechange = function() {}
        _retiredRequests = _retiredRequests.concat([request])
        _retiredCleanupTimer.restart()
        Qt.callLater(function() {
            try {
                request.abort()
            } catch (error) {
                // The generation is already invalidated; abort is best-effort cleanup.
            }
        })
    }
}
