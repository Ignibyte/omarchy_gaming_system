import QtQuick

QtObject {
    id: root

    property string state: "connection"
    property string connectionState: "idle"
    property string serverUrl: "http://127.0.0.1:8080"
    property string accessMode: "sign_in"
    property string suggestedUsername: ""
    property string statusText: "Choose a server to begin."
    property string errorText: ""
    property bool busy: api.requestInFlight
    property var personas: []
    property var selectedPersona: null
    readonly property bool hasSession: api.hasBearer
    readonly property bool hasMfaChallenge: _mfaChallengeToken !== ""
    readonly property string mfaExpiresAt: _mfaExpiresAt
    property alias requestTimeoutMilliseconds: api.timeoutMilliseconds
    property alias maximumResponseBytes: api.maximumResponseBytes

    signal playerRequestFinished(int generation, string operation, int status,
                                 string body, string transportError)

    property string _mfaChallengeToken: ""
    property string _mfaExpiresAt: ""
    property string _pendingUsername: ""
    property string _deviceName: "Omarchy QML"
    property int _expectedGeneration: 0

    property ApiClient _api: ApiClient {
        id: api
        onFinished: function(generation, operation, status, body, transportError) {
            root._handleFinished(generation, operation, status, body, transportError)
            root.playerRequestFinished(generation, operation, status, body, transportError)
        }
    }

    property Timer _mfaExpiryTimer: Timer {
        interval: 1000
        repeat: true
        running: root._mfaChallengeToken !== ""
        onTriggered: root._expireMfaIfNeeded()
    }

    function initialize(candidate) {
        connectToServer(candidate || serverUrl)
    }

    function connectToServer(candidate) {
        _clearAuthority()
        errorText = ""
        statusText = "Checking the server..."
        state = "connection"
        connectionState = "connecting"
        const configured = api.configure(candidate)
        if (!configured.ok) {
            connectionState = "configuration_error"
            statusText = "Server configuration needs attention."
            errorText = configured.error
            return false
        }
        serverUrl = configured.url
        _expectedGeneration = api.request("health", "GET", "/health", null, false)
        return _expectedGeneration !== 0
    }

    function retryHealth() {
        return connectToServer(serverUrl)
    }

    function showServerConfiguration() {
        api.cancel()
        _clearAuthority()
        state = "connection"
        connectionState = "idle"
        statusText = "Choose a server to begin."
        errorText = ""
    }

    function chooseAccessMode(mode) {
        if (mode !== "sign_in" && mode !== "register")
            return false
        accessMode = mode
        errorText = ""
        statusText = mode === "register"
            ? "Create a private account." : "Sign in to this device."
        return true
    }

    function registerAccount(username, password) {
        if (state !== "access" || accessMode !== "register" || busy)
            return false
        errorText = ""
        statusText = "Creating the account..."
        _expectedGeneration = api.request(
                    "register", "POST", "/v1/accounts",
                    {"username": String(username), "password": String(password)}, false)
        return _expectedGeneration !== 0
    }

    function signIn(username, password, deviceName) {
        if (state !== "access" || accessMode !== "sign_in" || busy)
            return false
        _pendingUsername = String(username).trim()
        _deviceName = String(deviceName).trim()
        errorText = ""
        statusText = "Verifying account credentials..."
        _expectedGeneration = api.request(
                    "login", "POST", "/v1/sessions",
                    {"username": String(username), "password": String(password),
                     "device_name": _deviceName}, false)
        return _expectedGeneration !== 0
    }

    function completeMfa(factor) {
        if (state !== "mfa" || _mfaChallengeToken === "" || busy)
            return false
        _expireMfaIfNeeded()
        if (_mfaChallengeToken === "")
            return false
        errorText = ""
        statusText = "Verifying the second factor..."
        _expectedGeneration = api.request(
                    "mfa", "POST", "/v1/sessions/mfa",
                    {"challenge_token": _mfaChallengeToken, "code": String(factor)}, false)
        return _expectedGeneration !== 0
    }

    function cancelMfa() {
        if (state !== "mfa")
            return false
        api.cancel()
        _clearMfa()
        state = "access"
        accessMode = "sign_in"
        statusText = "Second-factor sign-in canceled."
        errorText = ""
        return true
    }

    function loadPersonas() {
        if (!api.hasBearer || busy)
            return false
        errorText = ""
        statusText = "Loading owned personas..."
        _expectedGeneration = api.request("list_personas", "GET", "/v1/personas", null, true)
        return _expectedGeneration !== 0
    }

    function createPersona(handle, displayName, bio, statusMessage) {
        if (state !== "personas" || !api.hasBearer || busy)
            return false
        errorText = ""
        statusText = "Creating the persona..."
        _expectedGeneration = api.request(
                    "create_persona", "POST", "/v1/personas",
                    {"handle": String(handle), "display_name": String(displayName),
                     "bio": String(bio), "status_message": String(statusMessage)}, true)
        return _expectedGeneration !== 0
    }

    function selectPersona(persona) {
        if (state !== "personas" || !_validPersona(persona))
            return false
        let owned = false
        for (let index = 0; index < personas.length; index++) {
            if (personas[index].id === persona.id) {
                owned = true
                break
            }
        }
        if (!owned)
            return false
        selectedPersona = persona
        state = "home"
        statusText = "Persona link ready."
        errorText = ""
        return true
    }

    function showPlayerScreen(screen) {
        if (!hasSession || !_validPersona(selectedPersona) || busy)
            return false
        if (screen !== "home" && screen !== "social" && screen !== "inbox"
                && screen !== "games" && screen !== "challenges" && screen !== "gameplay")
            return false
        state = screen
        statusText = screen === "social" ? "Social link ready."
                   : screen === "inbox" ? "Private inbox ready."
                   : screen === "games" ? "Game cartridge link ready."
                   : screen === "challenges" ? "Challenge link ready."
                   : screen === "gameplay" ? "Authoritative game link ready."
                   : "Persona link ready."
        errorText = ""
        return true
    }

    function playerRequest(operation, method, path, document, authenticated) {
        if (!hasSession || !_validPersona(selectedPersona) || busy)
            return 0
        if (state !== "home" && state !== "social" && state !== "inbox"
                && state !== "games" && state !== "challenges" && state !== "gameplay")
            return 0
        if (typeof operation !== "string" || !operation.startsWith("player_")
                || typeof path !== "string" || !path.startsWith("/v1/"))
            return 0
        return api.request(operation, method, path, document, authenticated !== false)
    }

    function cancelPlayerRequest() {
        api.cancel()
    }

    function invalidatePlayerSession(message) {
        _returnToAccess(String(message || "This device session is no longer valid."))
    }

    function logout() {
        api.cancel()
        _clearAuthority()
        state = "access"
        accessMode = "sign_in"
        statusText = "Signed out on this device."
        errorText = ""
    }

    function clearError() {
        errorText = ""
    }

    function _handleFinished(generation, operation, status, body, transportError) {
        if (generation !== _expectedGeneration)
            return
        _expectedGeneration = 0

        if (transportError !== "") {
            _handleTransportFailure(operation, transportError)
            return
        }

        const parsed = _parseDocument(body)
        if (!parsed.ok) {
            _handleProtocolFailure(operation)
            return
        }
        const document = parsed.document

        if (operation === "health") {
            if (status === 200 && _validHealth(document)) {
                connectionState = "ready"
                state = "access"
                accessMode = "sign_in"
                statusText = "Server ready. Sign in or create an account."
                errorText = ""
            } else {
                connectionState = "protocol_error"
                statusText = "The server did not identify as OmarchyGS."
                errorText = "Check the address or try again."
            }
            return
        }

        if (operation === "register") {
            if (status === 201 && _validAccount(document)) {
                suggestedUsername = document.username
                accessMode = "sign_in"
                statusText = "Account created. Sign in to continue."
                errorText = ""
            } else {
                _handleActionError(status, document, "register")
            }
            return
        }

        if (operation === "login") {
            if (status === 201 && _validSessionCreation(document)) {
                _acceptSession(document)
            } else if (status === 202 && _validMfaChallenge(document)) {
                _mfaChallengeToken = document.challenge_token
                _mfaExpiresAt = document.expires_at
                state = "mfa"
                statusText = "Enter an authenticator or recovery code."
                errorText = ""
            } else {
                _handleActionError(status, document, "login")
            }
            return
        }

        if (operation === "mfa") {
            if (status === 201 && _validSessionCreation(document)) {
                _acceptSession(document)
            } else {
                const code = _errorCode(document)
                if (code === "invalid_mfa_challenge") {
                    _clearMfa()
                    state = "access"
                    accessMode = "sign_in"
                }
                _handleActionError(status, document, "mfa")
            }
            return
        }

        if (operation === "list_personas") {
            if (status === 200 && _validPersonaList(document)) {
                personas = document.personas
                selectedPersona = null
                state = "personas"
                statusText = personas.length === 0
                    ? "Create the first persona for this account."
                    : "Choose an owned persona or create another."
                errorText = ""
            } else if (_isInvalidSession(status, document)) {
                _returnToAccess("This device session is no longer valid.")
            } else {
                _handleAuthenticatedProtocolFailure()
            }
            return
        }

        if (operation === "create_persona") {
            if (status === 201 && _validPersona(document)) {
                personas = personas.concat([document])
                selectedPersona = document
                state = "home"
                statusText = "Persona created and selected."
                errorText = ""
            } else if (_isInvalidSession(status, document)) {
                _returnToAccess("This device session is no longer valid.")
            } else {
                _handleActionError(status, document, "persona")
            }
        }
    }

    function _acceptSession(document) {
        api.installBearer(document.token)
        _clearMfa()
        personas = []
        selectedPersona = null
        statusText = "Session created. Loading personas..."
        errorText = ""
        Qt.callLater(function() { root.loadPersonas() })
    }

    function _handleTransportFailure(operation, transportError) {
        const text = transportError === "timeout"
            ? "The request timed out. Try again."
            : transportError === "response_too_large"
              ? "The server response exceeded the client limit."
              : transportError === "unexpected_redirect"
                ? "The server redirected outside the selected endpoint."
                : "The server could not be reached."
        if (operation === "health") {
            connectionState = "offline"
            state = "connection"
            statusText = "Server offline."
            errorText = text
        } else {
            statusText = "Request not completed."
            errorText = text
        }
    }

    function _handleProtocolFailure(operation) {
        if (operation === "health") {
            state = "connection"
            connectionState = "protocol_error"
            statusText = "The server returned invalid JSON."
            errorText = "Check the address or try again."
        } else if (operation === "list_personas" || operation === "create_persona") {
            _handleAuthenticatedProtocolFailure()
        } else {
            statusText = "The server returned an unexpected response."
            errorText = "No account or session state was accepted."
        }
    }

    function _handleAuthenticatedProtocolFailure() {
        _clearAuthority()
        state = "access"
        accessMode = "sign_in"
        statusText = "The authenticated response was not accepted."
        errorText = "Sign in again after checking the server."
    }

    function _handleActionError(status, document, context) {
        const code = _errorCode(document)
        if (context === "persona" && _isInvalidSession(status, document)) {
            _returnToAccess("This device session is no longer valid.")
            return
        }
        if (code === "") {
            _handleProtocolFailure(context)
            return
        }
        statusText = "The request was not accepted."
        errorText = _safeErrorMessage(code)
    }

    function _returnToAccess(message) {
        _clearAuthority()
        state = "access"
        accessMode = "sign_in"
        statusText = message
        errorText = "Sign in again to continue."
    }

    function _clearAuthority() {
        api.cancel()
        api.clearBearer()
        _clearMfa()
        personas = []
        selectedPersona = null
        _expectedGeneration = 0
    }

    function _clearMfa() {
        _mfaChallengeToken = ""
        _mfaExpiresAt = ""
    }

    function _expireMfaIfNeeded() {
        if (_mfaChallengeToken === "")
            return
        const expiry = Date.parse(_mfaExpiresAt)
        if (!Number.isFinite(expiry) || Date.now() < expiry)
            return
        api.cancel()
        _clearMfa()
        state = "access"
        accessMode = "sign_in"
        statusText = "The MFA challenge expired."
        errorText = "Sign in again to request a new challenge."
    }

    function _parseDocument(body) {
        if (typeof body !== "string" || body.length === 0)
            return {"ok": false}
        try {
            const document = JSON.parse(body)
            if (!document || typeof document !== "object" || Array.isArray(document))
                return {"ok": false}
            return {"ok": true, "document": document}
        } catch (error) {
            return {"ok": false}
        }
    }

    function _validHealth(document) {
        return api.exactKeys(document, ["service", "version", "status", "database"])
                && document.service === "omarchy-gaming-system"
                && _boundedString(document.version, 64)
                && document.status === "ok" && document.database === "ok"
    }

    function _validAccount(document) {
        return api.exactKeys(document, ["id", "username"])
                && _validUuid(document.id) && _boundedString(document.username, 32, 3)
    }

    function _validSessionCreation(document) {
        if (!api.exactKeys(document, ["token", "session"])
                || typeof document.token !== "string"
                || !/^(ogs1_|bbs1_)[A-Za-z0-9_-]{43}$/.test(document.token))
            return false
        const session = document.session
        return api.exactKeys(session, ["id", "device_name", "created_at", "last_used_at",
                                       "expires_at", "revoked_at", "current"])
                && _validUuid(session.id) && _boundedString(session.device_name, 64, 1)
                && _validTimestamp(session.created_at) && _validTimestamp(session.last_used_at)
                && _validTimestamp(session.expires_at) && session.revoked_at === null
                && Date.parse(session.expires_at) > Date.now() && session.current === true
    }

    function _validMfaChallenge(document) {
        return api.exactKeys(document, ["mfa_required", "challenge_token", "expires_at"])
                && document.mfa_required === true
                && typeof document.challenge_token === "string"
                && /^ogm1_[A-Za-z0-9_-]{43}$/.test(document.challenge_token)
                && _validTimestamp(document.expires_at)
                && Date.parse(document.expires_at) > Date.now()
    }

    function _validPersonaList(document) {
        return api.exactKeys(document, ["personas"])
                && Array.isArray(document.personas)
                && document.personas.length <= 1000
                && document.personas.every(function(persona) { return root._validPersona(persona) })
    }

    function _validPersona(persona) {
        return api.exactKeys(persona, ["id", "handle", "display_name", "bio",
                                      "status_message", "created_at", "updated_at"])
                && _validUuid(persona.id)
                && _boundedString(persona.handle, 24, 3)
                && _boundedString(persona.display_name, 64, 1)
                && _boundedString(persona.bio, 1000)
                && _boundedString(persona.status_message, 160)
                && _validTimestamp(persona.created_at) && _validTimestamp(persona.updated_at)
    }

    function _errorCode(document) {
        if (!api.exactKeys(document, ["error"])
                || !api.exactKeys(document.error, ["code", "message"])
                || !_boundedString(document.error.code, 64, 1)
                || !_boundedString(document.error.message, 512, 1))
            return ""
        return document.error.code
    }

    function _isInvalidSession(status, document) {
        return status === 401 && _errorCode(document) === "invalid_session"
    }

    function _safeErrorMessage(code) {
        const messages = {
            "invalid_username": "Use 3–32 letters, numbers, underscores, or hyphens.",
            "invalid_password": "Use a password between 12 and 128 characters.",
            "username_taken": "That username is already registered.",
            "invalid_credentials": "The username or password was not accepted.",
            "invalid_device_name": "Use a device label between 1 and 64 characters.",
            "mfa_rate_limited": "Too many attempts. Wait before trying again.",
            "invalid_mfa_code": "That authenticator or recovery code was not accepted.",
            "invalid_mfa_challenge": "That MFA challenge expired or was already used.",
            "invalid_handle": "Use a 3–24 character persona handle.",
            "invalid_display_name": "Use a display name between 1 and 64 characters.",
            "invalid_bio": "The bio is too long or contains unsupported controls.",
            "invalid_status_message": "The status message is too long or contains unsupported controls.",
            "handle_taken": "That persona handle is already in use.",
            "internal_error": "The server could not complete the request."
        }
        return messages[code] || "The server rejected the request."
    }

    function _validUuid(value) {
        return typeof value === "string"
                && /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(value)
    }

    function _validTimestamp(value) {
        return typeof value === "string" && value.endsWith("Z")
                && Number.isFinite(Date.parse(value))
    }

    function _boundedString(value, maximum, minimum) {
        const lowerBound = minimum === undefined ? 0 : minimum
        return typeof value === "string"
                && value.length >= lowerBound && value.length <= maximum
    }
}
