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
    readonly property var serverProfiles: profileStore.profiles
    property var currentServer: null
    property string selectedProfileId: ""
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
    property string _expectedServerId: ""
    property bool _rememberServer: false

    property ServerProfiles _profileStore: ServerProfiles { id: profileStore }

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
        connectToServer(candidate || serverUrl, false)
    }

    function connectToServer(candidate, remember) {
        _clearAuthority()
        currentServer = null
        selectedProfileId = ""
        _expectedServerId = ""
        _rememberServer = remember === true
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
        const remembered = profileStore.profileForOrigin(serverUrl)
        if (remembered !== null) {
            _expectedServerId = remembered.server_id
            selectedProfileId = remembered.server_id
        }
        _expectedGeneration = api.request(
                    "discovery", "GET", "/.well-known/omarchygs", null, false)
        return _expectedGeneration !== 0
    }

    function connectSavedProfile(index) {
        if (!Number.isInteger(index) || index < 0 || index >= serverProfiles.length || busy)
            return false
        const profile = serverProfiles[index]
        const started = connectToServer(profile.origin, false)
        if (started) {
            _expectedServerId = profile.server_id
            selectedProfileId = profile.server_id
        }
        return started
    }

    function removeServerProfile(index) {
        if (!Number.isInteger(index) || index < 0 || index >= serverProfiles.length)
            return false
        const removed = serverProfiles[index]
        _clearAuthority()
        if (!profileStore.removeProfile(index))
            return false
        if (selectedProfileId === removed.server_id) {
            selectedProfileId = ""
            currentServer = null
        }
        state = "connection"
        connectionState = "idle"
        statusText = "Saved server removed."
        errorText = ""
        return true
    }

    function retryHealth() {
        return connectToServer(serverUrl, false)
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

    function registerAccount(inviteCode, username, password) {
        if (state !== "access" || accessMode !== "register" || busy)
            return false
        errorText = ""
        statusText = "Creating the account..."
        _expectedGeneration = api.request(
                    "register", "POST", "/v1/accounts",
                    {"invite_code": String(inviteCode),
                     "username": String(username), "password": String(password)}, false)
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

    function trustedCartridgeAuthority() {
        if (!hasSession || !_validPersona(selectedPersona) || currentServer === null
                || currentServer.capabilities.indexOf("games.cartridge-catalog.v1") === -1)
            return null
        const authority = {
            "origin": serverUrl,
            "server_id": currentServer.server_id,
            "device_bearer": api.trustedBearer(),
            "acquisition_supported": currentServer.capabilities.indexOf(
                                         "games.cartridge-acquisition.v1") !== -1,
            "session_acquisition_supported": currentServer.capabilities.indexOf(
                    "games.session-cartridge-acquisition.v1") !== -1
        }
        if (currentServer.operator_custom !== undefined)
            authority.operator_custom = currentServer.operator_custom
        return authority
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

        if (operation === "discovery") {
            const discovery = status === 200 ? _validatedDiscovery(document) : {"ok": false}
            if (status === 200 && discovery.ok) {
                if (discovery.incompatible) {
                    state = "connection"
                    connectionState = "incompatible"
                    statusText = "Server protocol is incompatible."
                    errorText = "This client requires OmarchyGS protocol 1 onboarding support."
                    return
                }
                if (_expectedServerId !== ""
                        && discovery.profile.server_id !== _expectedServerId) {
                    state = "connection"
                    connectionState = "identity_mismatch"
                    statusText = "Saved server identity changed."
                    errorText = "Remove the saved server before trusting this replacement."
                    return
                }
                if ((_rememberServer || _expectedServerId !== "")
                        && !profileStore.saveProfile(discovery.profile)) {
                    state = "connection"
                    connectionState = "profile_error"
                    statusText = "Server profile was not saved."
                    errorText = "Remove a conflicting or unneeded saved server and try again."
                    return
                }
                currentServer = discovery.profile
                selectedProfileId = discovery.profile.server_id
                connectionState = "ready"
                state = "access"
                accessMode = "sign_in"
                statusText = discovery.profile.server_name + " ready. Sign in or create an account."
                errorText = ""
            } else {
                state = "connection"
                connectionState = status === 503 ? "offline" : "protocol_error"
                statusText = status === 503 ? "Server discovery unavailable."
                                           : "The server did not identify as OmarchyGS."
                errorText = "Check the address or try again."
            }
            return
        }

        if (operation === "register") {
            if ((status === 200 || status === 201) && _validAccount(document)) {
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
        if (operation === "discovery") {
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
        if (operation === "discovery") {
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
        suggestedUsername = ""
        _pendingUsername = ""
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

    function _validatedDiscovery(document) {
        const keys = ["service", "server_id", "server_name",
                      "protocol_version", "capabilities"]
        if (document.operator_custom !== undefined)
            keys.push("operator_custom")
        if (document.operator_custom_modules !== undefined)
            keys.push("operator_custom_modules")
        if (!api.exactKeys(document, keys)
                || document.service !== "omarchy-gaming-system"
                || !_validUuid(document.server_id)
                || !_boundedPublicString(document.server_name, 64, 1)
                || !Number.isInteger(document.protocol_version)
                || !Array.isArray(document.capabilities)
                || document.capabilities.length > 32
                || (document.operator_custom !== undefined
                    && !_validOperatorCustom(document.operator_custom))
                || (document.operator_custom_modules !== undefined
                    && !_validOperatorCustomModules(document.operator_custom_modules,
                                                    document.server_id)))
            return {"ok": false}
        let previous = ""
        for (let index = 0; index < document.capabilities.length; index++) {
            const capability = document.capabilities[index]
            if (typeof capability !== "string"
                    || !/^[a-z0-9][a-z0-9.-]{0,63}$/.test(capability)
                    || (index > 0 && previous >= capability))
                return {"ok": false}
            previous = capability
        }
        const required = ["accounts.invite-registration.v1",
                          "auth.device-sessions.v1", "identity.personas.v1"]
        const incompatible = document.protocol_version !== 1
                || !required.every(function(capability) {
                    return document.capabilities.indexOf(capability) !== -1
                })
        const profile = {
            "origin": serverUrl,
            "server_id": document.server_id,
            "server_name": document.server_name,
            "protocol_version": document.protocol_version,
            "capabilities": document.capabilities.slice()
        }
        if (document.operator_custom !== undefined)
            profile.operator_custom = document.operator_custom
        if (document.operator_custom_modules !== undefined)
            profile.operator_custom_modules = document.operator_custom_modules
        return {
            "ok": true,
            "incompatible": incompatible,
            "profile": profile
        }
    }

    function _validOperatorCustom(value) {
        if (!value || typeof value !== "object"
                || !api.exactKeys(value, ["operator_name", "authority_id", "key_id",
                                          "key_sha256", "public_key"])
                || !_boundedPublicString(value.operator_name, 128, 1)
                || !/^[a-z][a-z0-9._-]{0,95}$/.test(value.authority_id)
                || !/^[a-z][a-z0-9._-]{0,95}$/.test(value.key_id)
                || !/^[0-9a-f]{64}$/.test(value.key_sha256))
            return false
        const key = value.public_key
        return key && typeof key === "object"
                && api.exactKeys(key, ["format_version", "algorithm", "key_id",
                                       "authority_id", "verifying_key"])
                && key.format_version === 1 && key.algorithm === "ed25519"
                && key.key_id === value.key_id && key.authority_id === value.authority_id
                && typeof key.verifying_key === "string"
                && /^[A-Za-z0-9_-]{43}$/.test(key.verifying_key)
    }

    function _validOperatorCustomModules(value, serverId) {
        if (!value || typeof value !== "object"
                || !api.exactKeys(value, ["format", "server_id", "active_count",
                                          "behavior_capabilities", "warning",
                                          "support_boundary"])
                || value.format !== "omarchygs.operator-custom-modules-disclosure/v1"
                || value.server_id !== serverId
                || !Number.isInteger(value.active_count)
                || value.active_count < 1 || value.active_count > 8
                || !Array.isArray(value.behavior_capabilities)
                || value.behavior_capabilities.length > 4
                || value.warning !== "This server runs operator-custom code not reviewed or supported by OmarchyGS."
                || value.support_boundary !== "Security, privacy, availability, and support are the server operator's responsibility.")
            return false
        let previous = ""
        for (let index = 0; index < value.behavior_capabilities.length; index++) {
            const capability = value.behavior_capabilities[index]
            if (capability !== "moderation_labels"
                    || (index > 0 && previous >= capability))
                return false
            previous = capability
        }
        return true
    }

    function _boundedPublicString(value, maximum, minimum) {
        return _boundedString(value, maximum, minimum)
                && value.trim() === value
                && !/[\u0000-\u001f\u007f-\u009f]/.test(value)
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
            "invalid_invitation": "That invitation is invalid, expired, revoked, or already used.",
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
